use crate::parser::Parser;
use crate::scanner::Scanner;
use anyhow::anyhow;
use anyhow::Result;
use clap::Parser as _;
use itertools::Itertools;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

mod expression;
mod item;
mod parser;
mod scanner;
mod statement;
mod token;

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
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let _ = std::fs::remove_dir_all(".dist");
    std::fs::create_dir_all(".dist/runtime")?;
    match &cli.command {
        Commands::Serve { path } => serve(path),
        Commands::Run { path } => run(path),
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

fn run(path: &Path) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let mut file = File::open(&path)?;
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

            let mut output_path = Path::new(".dist/runtime").join(path.file_stem().unwrap());
            output_path.set_extension("go");
            let mut output_file = File::create(&output_path)?;
            output_file.write_all("package main\n".as_bytes())?;
            // TODO: Propagate this information up via the parser
            if output.contains("fmt.Println") || output.contains("fmt.Sprintf") {
                output_file.write_all("import \"fmt\"\n".as_bytes())?;
            }
            output_file.write_all(output.as_bytes())?;
        }
    }

    std::env::set_current_dir(".dist/runtime")?;
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
