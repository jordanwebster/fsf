use crate::compilers::go_compiler::GoCompiler;
use crate::compilers::js_compiler::JsCompiler;
use crate::compilers::{Module, Program};
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::test_collector::TestCollector;
use anyhow::anyhow;
use anyhow::Result;
use clap::Parser as _;
use itertools::Itertools;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

mod compilers;
mod expression;
mod item;
mod parser;
mod scanner;
mod statement;
mod test_collector;
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
    let mut file = File::open(path.join("index.fsf"))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let output = parser
        .parse()
        .into_iter()
        .filter_map(|stmt| stmt.map(|s| s.compile()))
        .join("");

    std::fs::create_dir_all(".dist/runtime")?;
    let output_path = Path::new(".dist/runtime").join("index.go");
    let mut output_file = File::create(&output_path)?;
    output_file.write_all("package main\n".as_bytes())?;
    // TODO: Propagate this information up via the parser
    if output.contains("fmt.Println") || output.contains("fmt.Sprintf") {
        output_file.write_all("import \"fmt\"\n".as_bytes())?;
    }
    output_file.write_all(output.as_bytes())?;

    setup_runtime()?;

    std::env::set_current_dir(".dist/runtime")?;
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

fn parse_module(path: PathBuf) -> Result<Module> {
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut scanner = Scanner::new(contents);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
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
        .map(parse_module)
        .collect::<Result<Program>>()?;

    match target {
        Target::Go => {
            let mut compiler = GoCompiler::new();
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
            todo!();
            // let mut compiler = JsCompiler::new();
            // compiler.compile(program)?;
            // Ok(())
        }
    }
}

fn test(path: &Path, target: &Target) -> Result<()> {
    let program = std::fs::read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().is_file() => Some(entry.path()),
            _ => None,
        })
        .map(parse_module)
        .collect::<Result<Program>>()?;
    let tests = TestCollector::all_tests(&program);

    match target {
        Target::Go => {
            let mut compiler = GoCompiler::new();
            let compile_dir = PathBuf::from(".dist/runtime");
            std::fs::create_dir_all(&compile_dir)?;
            compiler.compile(program, &compile_dir)?;

            // TODO: Make part of compiler
            setup_go_test_runner(tests)?;
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
            let mut compiler = JsCompiler::new();
            let compile_dir = PathBuf::from(".dist/js");
            std::fs::create_dir_all(&compile_dir)?;
            compiler.compile(program, &compile_dir, Some(tests))?;

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

    Ok(if !output.status.success() {
        eprintln!(
            "Failed to copy directory: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Command execution failed",
        ));
    })
}

fn setup_go_test_runner(tests: Vec<String>) -> Result<()> {
    let input_file_path = Path::new("../test_runner/test_runner.go");
    let output_file_path = Path::new(".dist/runtime/main.go");

    let mut content = String::new();
    let mut file = File::open(input_file_path)?;
    file.read_to_string(&mut content)?;

    let replacement = tests
        .iter()
        .map(|t| format!("runner.runTest({t}, \"{t}\")"))
        .join("\n");

    let new_content = content.replace("/* replace: tests */", &replacement);

    let mut output_file = File::create(output_file_path)?;
    output_file.write_all(new_content.as_bytes())?;

    Ok(())
}
