use crate::parser::Parser;
use crate::scanner::Scanner;
use itertools::Itertools;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

mod expression;
mod parser;
mod scanner;
mod statement;
mod token;

fn main() -> std::io::Result<()> {
    std::fs::create_dir_all("dist")?;
    compile_example()?;
    // setup_runtime()?;
    // serve()
    Ok(())
}

fn serve() -> Result<(), std::io::Error> {
    std::env::set_current_dir("dist/runtime")?;
    let output = Command::new("go").arg("run").arg(".").output()?;

    if !output.status.success() {
        eprintln!(
            "Failed to run Go command: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Command execution failed",
        ));
    }

    Ok(())
}

fn compile_example() -> Result<(), std::io::Error> {
    let mut file = File::open("../example/test.wip")?;
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

    let output_path = Path::new("dist").join("test.go");
    let mut output_file = File::create(&output_path)?;
    output_file.write_all(output.as_bytes())?;

    // std::fs::create_dir_all("dist/static")?;
    //
    // let mut file = File::open("../example/index.zhtml")?;
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;
    //
    // let output_path = Path::new("dist/static").join("index.html");
    // let mut output_file = File::create(output_path)?;
    // output_file.write_all(contents.as_bytes())?;

    Ok(())
}

fn setup_runtime() -> Result<(), std::io::Error> {
    let output = Command::new("cp")
        .arg("-R")
        .arg("../runtime")
        .arg("./dist")
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
