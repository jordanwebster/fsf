use crate::compilers::Program;
use crate::expression::{Expression, ExpressionWithBlock, ExpressionWithoutBlock, FStringChunk};
use crate::item::Item;
use crate::statement::Statement;
use crate::token::{Literal, TokenType};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const MAIN_BOOTSTRAP: &str = include_str!("../bootstrap/js_bootstrap.js");

pub struct JsCompiler {
    name_map: HashMap<String, String>,
}

impl JsCompiler {
    pub fn new() -> Self {
        Self {
            name_map: HashMap::new(),
        }
    }

    pub fn compile(
        &mut self,
        root: &Path,
        program: Program,
        compile_dir: &Path,
        tests: Option<Vec<String>>,
    ) -> Result<()> {
        let output_path = compile_dir.join("main.js");
        let mut output_file = File::create(&output_path)?;

        let name_map = Self::construct_name_map(root, &program);
        if let Some(tests) = tests {
            let tests = tests.into_iter().map(|t| name_map[&t].clone()).collect();
            Self::setup_test_runner(tests, &mut output_file)?;
        }
        self.name_map = name_map;

        for module in program {
            let output = module
                .items
                .into_iter()
                .map(|item| self.compile_item(item))
                .join("");

            output_file.write_all(output.as_bytes())?;
        }

        output_file.write_all(MAIN_BOOTSTRAP.as_bytes())?;
        Ok(())
    }

    fn construct_name_map(root: &Path, program: &Program) -> HashMap<String, String> {
        program
            .iter()
            .flat_map(|module| {
                module.items.iter().filter_map(|item| match item {
                    // TODO: Add resolving relative imports (not from project root)
                    Item::Import { path } => Some((path.last().unwrap().clone(), path.join("_"))),

                    // TODO: Add transformer step to find main function and add bootstrapping
                    Item::Function { name, .. } => {
                        if name == "main" {
                            Some((name.clone(), name.clone()))
                        } else {
                            Some((
                                name.clone(),
                                format!(
                                    "{}_{}",
                                    Path::new(
                                        module
                                            .path
                                            .strip_prefix(root)
                                            .unwrap()
                                            .file_stem()
                                            .unwrap()
                                    )
                                    .iter()
                                    .map(|p| p.to_string_lossy())
                                    .join("_"),
                                    name
                                ),
                            ))
                        }
                    }
                    _ => None,
                })
            })
            .collect()
    }

    fn setup_test_runner(tests: Vec<String>, output_file: &mut File) -> Result<()> {
        let input_file_path = Path::new("../test_runner/test_runner.js");

        let mut content = String::new();
        let mut file = File::open(input_file_path)?;
        file.read_to_string(&mut content)?;

        let replacement = tests
            .iter()
            .map(|t| format!("runner.runTest({t}, \"{t}\");"))
            .join("\n");

        let new_content = content.replace("/* replace: tests */", &replacement);

        output_file.write_all(new_content.as_bytes())?;

        Ok(())
    }

    fn compile_item(&mut self, item: Item) -> String {
        match item {
            Item::Component { .. } => todo!(),
            Item::Function {
                name,
                parameters,
                body,
                ..
            } => {
                let statements = body
                    .statements
                    .into_iter()
                    .map(|s| self.compile_statement(s))
                    .join("");

                let params = parameters.iter().map(|p| p.name.clone()).join(", ");

                let name = self.name_map.get(&name).unwrap().to_string();
                match body.expr {
                    Some(expr) => format!(
                        "function {}({}) {{\n{}\nreturn {}\n}}\n",
                        name,
                        params,
                        statements,
                        self.compile_expression(expr)
                    ),
                    None => format!("function {}({}) {{\n{}\n}}\n", name, params, statements),
                }
            }
            Item::Import { .. } => "".to_string(),
            Item::TestRunner => r#"
                function runTest(test, name) {
                    process.stdout.write(`${name}...`);

                    try {
                        test();
                        process.stdout.write(" pass\n");
                    } catch (err) {
                        process.stdout.write(" fail\n");
                    }
                }"#
            .to_string(),
        }
    }

    fn compile_statement(&mut self, statement: Statement) -> String {
        match statement {
            Statement::Print(expr) => format!("console.log({});\n", self.compile_expression(expr)),
            Statement::Expression(expr) => format!("{}\n", self.compile_expression(expr)),
            Statement::Let {
                token, expression, ..
            } => match expression {
                Expression::WithoutBlock(expr) => format!(
                    "let {} = {};\n",
                    token.value.unwrap(),
                    self.compile_expression(Expression::WithoutBlock(expr))
                ),
                Expression::WithBlock(expr) => match expr {
                    ExpressionWithBlock::Block(block) => {
                        let statements_str = block
                            .statements
                            .into_iter()
                            .map(|stmt| self.compile_statement(stmt))
                            .join("");
                        if let Some(expr) = block.expr {
                            format!(
                                "{}let {} = {};\n",
                                statements_str,
                                token.value.unwrap(),
                                self.compile_expression(expr)
                            )
                        } else {
                            statements_str
                        }
                    }
                    ExpressionWithBlock::If { .. } => todo!(),
                },
            },
            Statement::AssertEq(left, right) => {
                format!(
                    "if ({} != {}) {{\nthrow new Error(`{} != {}`);\n}}\n",
                    self.compile_expression(left.clone()),
                    self.compile_expression(right.clone()),
                    // TODO: Replace with source not compiled form
                    self.compile_expression(left),
                    self.compile_expression(right),
                )
            }
            Statement::RunTest {
                test_name,
                function_name,
            } => {
                if let Some(Literal::String(test_name)) = test_name.value {
                    format!(
                        "runTest({}, \"{}\")\n",
                        self.compile_expression(*function_name),
                        test_name
                    )
                } else {
                    panic!("Test name must be a string");
                }
            }
        }
    }

    fn compile_expression<E>(&mut self, expr: E) -> String
    where
        E: Into<Expression>,
    {
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
            ExpressionWithoutBlock::Call { callee, arguments } => {
                format!(
                    "{}({})",
                    self.compile_expression(*callee),
                    arguments
                        .into_iter()
                        .map(|e| self.compile_expression(e))
                        .join(", ")
                )
            }
            ExpressionWithoutBlock::Lambda { parameters, body } => {
                let params = parameters.iter().map(|p| format!("{}", p.name,)).join(", ");
                match *body {
                    Expression::WithoutBlock(expression) => {
                        // TODO: Add proper type inference
                        format!(
                            "({}) => {{\nreturn {};\n}}\n",
                            params,
                            self.compile_expression_without_block(expression)
                        )
                    }
                    Expression::WithBlock(_expression) => todo!(),
                }
            }
            ExpressionWithoutBlock::Grouping(expr) => {
                format!("({})", self.compile_expression(*expr))
            }
            ExpressionWithoutBlock::Literal(literal) => self.compile_literal(&literal),
            ExpressionWithoutBlock::Unary { .. } => todo!(),
            ExpressionWithoutBlock::Variable(identifier) => self
                .name_map
                .get(&identifier.lexeme)
                .unwrap_or(&identifier.lexeme)
                .clone()
                .to_string(),
            ExpressionWithoutBlock::Assignment {
                name,
                value,
                operator,
            } => match operator.token_type {
                TokenType::Equal => {
                    format!("{} = {};", name.lexeme, self.compile_expression(*value))
                }
                TokenType::PlusEqual => {
                    format!("{} += {};", name.lexeme, self.compile_expression(*value))
                }
                _ => panic!("Unexpected token type in assignment: {}", operator.lexeme),
            },
            ExpressionWithoutBlock::Html { .. } => todo!(),
            ExpressionWithoutBlock::FString { chunks } => {
                let format_string = chunks
                    .iter()
                    .map(|chunk| match chunk {
                        FStringChunk::Literal(string) => string.to_string(),
                        FStringChunk::Identifier(string) => format!("${{{string}}}"),
                    })
                    .join("");
                format!("`{}`", format_string)
            }
        }
    }

    fn compile_literal(&mut self, literal: &Literal) -> String {
        match literal {
            Literal::Number(value) => format!("{}", value),
            Literal::String(value) => format!("\"{}\"", value),
            Literal::Identifier(identifier) => self.name_map[identifier].to_string(),
            Literal::True => "true".to_string(),
            Literal::False => "false".to_string(),
        }
    }
}
