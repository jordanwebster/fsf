use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

mod scanner;
mod token;

fn main() -> std::io::Result<()> {
    compile_example()?;
    setup_runtime()?;
    serve()
}

fn serve() -> Result<(), std::io::Error> {
    std::env::set_current_dir("dist/runtime")?;
    let output = Command::new("go")
        .arg("run")
        .arg(".")
        .output()?;

    if !output.status.success() {
        eprintln!("Failed to run Go command: {}", String::from_utf8_lossy(&output.stderr));
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Command execution failed"));
    }

    Ok(())
}

fn compile_example() -> Result<(), std::io::Error> {
    std::fs::create_dir_all("dist/static")?;

    let mut file = File::open("../example/index.zhtml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let output_path = Path::new("dist/static").join("index.html");
    let mut output_file = File::create(output_path)?;
    output_file.write_all(contents.as_bytes())?;

    Ok(())
}

fn setup_runtime() -> Result<(), std::io::Error> {
    let output = Command::new("cp")
        .arg("-R")
        .arg("../runtime")
        .arg("./dist")
        .output()?;

    Ok(if !output.status.success() {
        eprintln!("Failed to copy directory: {}", String::from_utf8_lossy(&output.stderr));
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Command execution failed"));
    })
}
