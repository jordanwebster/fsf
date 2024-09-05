use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

mod expression;
mod scanner;
mod token;
mod parser;

fn main() -> std::io::Result<()> {
    // let expression = Expression::Binary {
    //     left: Expression::Unary {
    //         operator: Token::new(TokenType::Minus, "-".to_string(), 1),
    //         right: Expression::Literal(Literal::Number(123.0)).into(),
    //     }
    //         .into(),
    //     operator: Token::new(TokenType::Star, "*".to_string(), 1),
    //     right: Expression::Grouping(Expression::Literal(Literal::Number(45.67)).into()).into(),
    // };
    //
    // println!("{}", printer::print(expression));
    //
    // Ok(())
    compile_example()?;
    setup_runtime()?;
    serve()
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
