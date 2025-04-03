use crate::compilers::Program;
use crate::expression::{Expression, ExpressionWithBlock, ExpressionWithoutBlock, FStringChunk};
use crate::item::Item;
use crate::statement::Statement;
use crate::token::{Literal, TokenType};
use anyhow::Result;
use itertools::Itertools;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct GoCompiler {}

impl GoCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(&mut self, program: Program, compile_dir: &Path) -> Result<()> {
        for module in program {
            let output = module
                .items
                .into_iter()
                .filter_map(|item| item.map(|item| (self.compile_item(item))))
                .join("");
            let mut output_path = compile_dir.join(module.path.file_stem().unwrap());
            output_path.set_extension("go");
            let mut output_file = File::create(&output_path)?;
            output_file.write_all("package main\n".as_bytes())?;
            // TODO: Propagate this information up via the parser
            if output.contains("fmt.Println") || output.contains("fmt.Sprintf") {
                output_file.write_all("import \"fmt\"\n".as_bytes())?;
            }
            output_file.write_all(output.as_bytes())?;
        }

        Ok(())
    }

    fn compile_item(&mut self, item: Item) -> String {
        match item {
            Item::Function {
                name,
                parameters,
                body,
                return_type,
            } => {
                let statements = body
                    .statements
                    .into_iter()
                    .map(|s| self.compile_statement(s))
                    .join("");
                let params = parameters
                    .iter()
                    .map(|p| {
                        format!(
                            "{} {}",
                            p.name,
                            match p.type_annotation.as_str() {
                                "int" => "int",
                                "str" => "string",
                                other => other,
                            }
                        )
                    })
                    .join(", ");
                match body.expr {
                    Some(expr) => format!(
                        "func {}({}) {} {{\n{}\nreturn {}\n}}\n",
                        name,
                        params,
                        return_type,
                        statements,
                        self.compile_expression(expr)
                    ),
                    None => format!("func {}({}) {{\n{}\n}}\n", name, params, statements),
                }
            }
            Item::Component {
                name,
                parameters,
                body,
            } => {
                let statements = body
                    .statements
                    .into_iter()
                    .map(|s| self.compile_statement(s))
                    .join("");
                let params = parameters
                    .iter()
                    .map(|p| {
                        format!(
                            "{} {}",
                            p.name,
                            match p.type_annotation.as_str() {
                                "int" => "int",
                                "str" => "string",
                                other => other,
                            }
                        )
                    })
                    .join(", ");
                match body.expr {
                    Some(expr) => format!(
                        "func {}({}) string {{\n{}\nreturn `{}`\n}}\n",
                        name,
                        params,
                        statements,
                        self.compile_expression(expr)
                    ),
                    None => format!("func {}({}) {{\n{}\n}}\n", name, params, statements),
                }
            }
            Item::Import { path } => todo!(),
        }
    }

    fn compile_statement(&mut self, statement: Statement) -> String {
        match statement {
            Statement::Print(expr) => format!("fmt.Println({})\n", self.compile_expression(expr)),
            Statement::Expression(expr) => format!("{}\n", self.compile_expression(expr)),
            Statement::Let {
                token,
                expression,
                mutable,
            } => match expression {
                Expression::WithoutBlock(expr) => {
                    format!(
                        "{} := {}\n",
                        token.value.unwrap(),
                        self.compile_expression(Expression::WithoutBlock(expr))
                    )
                }
                Expression::WithBlock(expr) => match expr {
                    ExpressionWithBlock::Block(block) => {
                        let statements_str = block
                            .statements
                            .into_iter()
                            .map(|stmt| self.compile_statement(stmt))
                            .join("");
                        if let Some(expr) = block.expr {
                            format!(
                                "{}{} := {}\n",
                                statements_str,
                                token.value.unwrap(),
                                self.compile_expression(expr)
                            )
                        } else {
                            statements_str
                        }
                    }
                    ExpressionWithBlock::If { expr, then, r#else } => todo!(),
                },
            },
            Statement::AssertEq(left, right) => {
                format!(
                    "if ({} != {}) {{\npanic(\"{} != {}\")}}\n",
                    self.compile_expression(left.clone()),
                    self.compile_expression(right.clone()),
                    // TODO: Replace with source not compiled form
                    self.compile_expression(left),
                    self.compile_expression(right),
                )
            }
        }
    }

    fn compile_expression<E>(&mut self, expr: E) -> String  where E: Into<Expression> {
        match expr.into() {
            Expression::WithBlock(expr) => self.compile_expression_with_block(expr),
            Expression::WithoutBlock(expr) => self.compile_expression_without_block(expr),
        }
    }

    fn compile_expression_with_block(&mut self, expr: ExpressionWithBlock) -> String {
        match expr {
            ExpressionWithBlock::Block(block) => block
                .statements
                .into_iter()
                .map(|s| self.compile_statement(s))
                .join(""),
            ExpressionWithBlock::If { expr, then, r#else } => {
                // TODO: Handle the case there is a dangling expression
                let mut s = format!(
                    "if ({}) {{\n{}}}",
                    self.compile_expression(*expr),
                    then.statements
                        .into_iter()
                        .map(|s| self.compile_statement(s))
                        .join("")
                );
                if let Some(r#else) = r#else {
                    s = format!(
                        "{} else {{\n{}}}\n",
                        s,
                        self.compile_expression(Expression::WithBlock(*r#else))
                    );
                }
                s
            }
        }
    }

    fn compile_expression_without_block(&mut self, expr: ExpressionWithoutBlock) -> String {
        match expr {
            ExpressionWithoutBlock::Binary {
                left,
                operator,
                right,
            } => format!(
                "{} {} {}",
                self.compile_expression(*left),
                operator.lexeme,
                self.compile_expression(*right)
            ),
            ExpressionWithoutBlock::Call { callee, arguments } => format!(
                "{}({})",
                self.compile_expression(*callee),
                arguments
                    .into_iter()
                    .map(|e| self.compile_expression(e))
                    .join(", ")
            ),
            ExpressionWithoutBlock::Grouping(expression) => {
                format!("({})", self.compile_expression(*expression))
            }
            ExpressionWithoutBlock::Literal(Literal::Number(number)) => format!("{}", number),
            ExpressionWithoutBlock::Literal(Literal::String(string)) => format!("\"{}\"", string),
            ExpressionWithoutBlock::Literal(Literal::Identifier(identifier)) => {
                identifier.to_string()
            }
            ExpressionWithoutBlock::Literal(Literal::True) => "true".to_string(),
            ExpressionWithoutBlock::Literal(Literal::False) => "false".to_string(),
            ExpressionWithoutBlock::Unary { operator, right } => todo!(),
            ExpressionWithoutBlock::Variable(identifier) => {
                format!("{}", identifier.value.unwrap())
            }
            ExpressionWithoutBlock::Assignment {
                name,
                value,
                operator,
            } => match operator.token_type {
                TokenType::Equal => format!(
                    "{} = {}",
                    name.lexeme,
                    self.compile_expression(*value)
                ),
                TokenType::PlusEqual => format!(
                    "{} += {}",
                    name.lexeme,
                    self.compile_expression(*value)
                ),
                _ => panic!("Unexpected token type in assignment: {}", operator.lexeme),
            },
            ExpressionWithoutBlock::FString { chunks } => {
                let format_string = chunks
                    .iter()
                    .map(|chunk| match chunk {
                        FStringChunk::Literal(string) => string,
                        FStringChunk::Identifier(string) => "%v", // TODO: Use correct specifier based on type
                    })
                    .join("");
                let arguments = chunks
                    .iter()
                    .filter_map(|chunk| match chunk {
                        FStringChunk::Literal(_) => None,
                        FStringChunk::Identifier(string) => Some(string),
                    })
                    .join(", ");
                format!("fmt.Sprintf(\"{}\", {})", format_string, arguments)
            }
            ExpressionWithoutBlock::Html { name, inner } => {
                format!(
                    "<{}>\n{}\n</{}>",
                    name.lexeme,
                    self.compile_expression(*inner),
                    name.lexeme
                )
            }
            ExpressionWithoutBlock::Lambda { parameters, body } => {
                let params = parameters
                    .iter()
                    .map(|p| {
                        format!(
                            "{} {}",
                            p.name,
                            match p.type_annotation.as_deref() {
                                Some("int") => "int",
                                Some("str") => "string",
                                Some(other) => other,
                                // TODO: Add proper type inference
                                None => "int",
                            }
                        )
                    })
                    .join(", ");
                match *body {
                    Expression::WithoutBlock(expression) => {
                        // TODO: Add proper type inference
                        format!(
                            "func({}) int {{\nreturn {};\n}}\n",
                            params,
                            self.compile_expression_without_block(expression)
                        )
                    }
                    Expression::WithBlock(expression) => todo!(),
                }
            }
        }
    }
}