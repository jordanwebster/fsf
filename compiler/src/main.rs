use crate::identifier_transformer::{
    GoIdentifierTransformer, JsIdentifierTransformer, StandardLibraryTransformer,
    TestRunnerTransformer,
};
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::targets::go_target::GoTarget;
use crate::targets::js_target::JsTarget;
use crate::targets::{Module, Program};
use anyhow::anyhow;
use anyhow::Result;
use clap::Parser as _;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

mod expression;
mod identifier_transformer;
mod item;
mod parser;
mod scanner;
mod statement;
mod targets;
mod token;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Target {
    Go,
    Js,
}

#[derive(clap::Parser)]
#[command(name = "fsf")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Start serving from the specified path
    Serve {
        /// The path to serve from
        path: PathBuf,
    },
    /// Run from the specified path
    Run {
        /// The path to run from
        path: PathBuf,

        #[arg(long, value_enum, default_value_t = Target::Go)]
        target: Target,
    },
    /// Run all the tests in the specified path
    Test {
        path: PathBuf,

        #[arg(long, value_enum, default_value_t = Target::Go)]
        target: Target,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let _ = std::fs::remove_dir_all(".dist");
    match &cli.command {
        Commands::Serve { path } => serve(path),
        Commands::Run { path, target } => run(path, target),
        Commands::Test { path, target } => test(path, target),
    }
}

fn serve(path: &Path) -> Result<()> {
    // TODO: Handle multiple routes
    let mut program = std::fs::read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().is_file() => Some(entry.path()),
            _ => None,
        })
        .map(parse_module_from_file)
        .collect::<Result<Program>>()?;

    let mut std_lib_transformer = StandardLibraryTransformer::new(path.into());
    std_lib_transformer.transform(&mut program)?;

    let js_program = program.clone();

    let compile_dir = PathBuf::from(".dist/runtime");
    std::fs::create_dir_all(&compile_dir)?;
    let mut compiler = GoTarget::new();
    compiler.compile(program, &compile_dir)?;

    setup_runtime()?;

    // Compile Javascript
    let js_dir = compile_dir.join("javascript");
    std::fs::create_dir_all(&js_dir)?;
    Command::new("rsync")
        .arg("-a")
        .arg("../javascript/")
        .arg(&js_dir)
        .status()?;

    // // TODO: Filter out modules to only include app/ directory
    // for module in js_program {
    //     let mut js_compiler = JsCompiler::new();
    //     js_compiler.compile(path, vec![module], &js_dir, false)?;
    // }
    let mut js_compiler = JsTarget::new();
    js_compiler.compile(path, js_program, &js_dir, false)?;

    let cwd = std::env::current_dir()?;
    std::env::set_current_dir(&js_dir)?;
    Command::new("npm").arg("install").status()?;
    Command::new("npm").arg("run").arg("build").status()?;
    std::env::set_current_dir(cwd)?;

    std::env::set_current_dir(compile_dir)?;
    let _ = Command::new("go")
        .arg("mod")
        .arg("init")
        .arg("fsf")
        .output()?;
    let output = Command::new("go").arg("run").arg(".").output()?;

    match output.status.success() {
        true => Ok(()),
        false => {
            eprintln!(
                "Failed to run Go command: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            Err(anyhow!("Failed to run Go command"))
        }
    }
}

fn parse_module_from_file(path: PathBuf) -> Result<Module> {
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    parse_module(contents, path)
}

fn parse_module(contents: String, path: PathBuf) -> Result<Module> {
    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens, path.clone());
    Ok(Module {
        path,
        items: parser.parse(),
    })
}

fn run(path: &Path, target: &Target) -> Result<()> {
    let program = std::fs::read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().is_file() => Some(entry.path()),
            _ => None,
        })
        .map(parse_module_from_file)
        .collect::<Result<Program>>()?;

    match target {
        Target::Go => {
            let mut compiler = GoTarget::new();
            std::fs::create_dir_all(".dist/runtime")?;
            let compile_dir = PathBuf::from(".dist/runtime");
            compiler.compile(program, &compile_dir)?;

            // TODO: Make this part of the compiler
            std::env::set_current_dir(compile_dir)?;
            let _ = Command::new("go")
                .arg("mod")
                .arg("init")
                .arg("fsf")
                .output()?;
            let output = Command::new("go").arg("run").arg(".").output()?;

            match output.status.success() {
                true => {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                    Ok(())
                }
                false => {
                    eprintln!(
                        "Failed to run Go command: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    Err(anyhow!("Failed to run Go command"))
                }
            }
        }
        Target::Js => {
            let mut compiler = JsTarget::new();
            let compile_dir = PathBuf::from("./dist/js");
            std::fs::create_dir_all(&compile_dir)?;
            compiler.compile(path, program, &compile_dir, true)?;

            std::env::set_current_dir(compile_dir)?;
            match Command::new("node").arg("main.js").status()?.success() {
                true => Ok(()),
                false => Err(anyhow!("Failed to run node")),
            }
        }
    }
}

fn test(path: &Path, target: &Target) -> Result<()> {
    let mut program = std::fs::read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().is_file() => Some(entry.path()),
            _ => None,
        })
        .map(parse_module_from_file)
        .collect::<Result<Program>>()?;

    let mut test_runner_transformer = TestRunnerTransformer::new(path.into());
    test_runner_transformer.transform(&mut program);

    let mut std_lib_transformer = StandardLibraryTransformer::new(path.into());
    std_lib_transformer.transform(&mut program)?;

    match target {
        Target::Go => {
            let mut identifier_transformer = GoIdentifierTransformer::new(path.into());
            identifier_transformer.transform(&mut program);

            let mut compiler = GoTarget::new();
            let compile_dir = PathBuf::from(".dist/runtime");
            std::fs::create_dir_all(&compile_dir)?;
            compiler.compile(program, &compile_dir)?;

            std::env::set_current_dir(compile_dir)?;
            let _ = Command::new("go")
                .arg("mod")
                .arg("init")
                .arg("fsf")
                .output()?;

            match Command::new("go").arg("run").arg(".").status()?.success() {
                true => Ok(()),
                false => Err(anyhow!("Tests failed")),
            }
        }
        Target::Js => {
            let mut identifier_transformer = JsIdentifierTransformer::new(path.into());
            identifier_transformer.transform(&mut program);

            let mut compiler = JsTarget::new();
            let compile_dir = PathBuf::from(".dist/js");
            std::fs::create_dir_all(&compile_dir)?;
            compiler.compile(path, program, &compile_dir, true)?;

            std::env::set_current_dir(compile_dir)?;
            match Command::new("node").arg("main.js").status()?.success() {
                true => Ok(()),
                false => Err(anyhow!("Tests failed")),
            }
        }
    }
}

fn setup_runtime() -> Result<(), std::io::Error> {
    let output = Command::new("cp")
        .arg("-R")
        .arg("../runtime")
        .arg("./.dist")
        .output()?;

    if !output.status.success() {
        eprintln!(
            "Failed to copy directory: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Command execution failed",
        ));
    };
    Ok(())
}
