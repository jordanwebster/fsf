use crate::compilers::Program;
use crate::expression::{Expression, ExpressionWithBlock, ExpressionWithoutBlock};
use crate::item::Item;
use crate::statement::Statement;
use crate::token::Literal;
use anyhow::Result;
use itertools::Itertools;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub struct JsCompiler {}

impl JsCompiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(
        &mut self,
        program: Program,
        compile_dir: &Path,
        tests: Option<Vec<String>>,
    ) -> Result<()> {
        let output_path = compile_dir.join("main.js");
        let mut output_file = File::create(&output_path)?;
        if let Some(tests) = tests {
            Self::setup_test_runner(tests, &mut output_file)?;
        }
        for module in program {
            let output = module
                .items
                .into_iter()
                .filter_map(|item| item.map(|item| self.compile_item(item)))
                .join("");

            output_file.write_all(output.as_bytes())?;
        }

        Ok(())
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
            Item::Component {
                name,
                parameters,
                body,
            } => todo!(),
            Item::Function {
                name,
                parameters,
                body,
                return_type,
            } => {
                let statements = body
                    .statements
                    .iter()
                    .map(|s| self.compile_statement(s))
                    .join("");

                let params = parameters.iter().map(|p| p.name.clone()).join(", ");

                match body.expr {
                    Some(ref expr) => format!(
                        "function {}({}) {{\n{}\nreturn {}\n}}\n",
                        name,
                        params,
                        statements,
                        self.compile_expression_without_block(expr)
                    ),
                    None => format!("function {}({}) {{\n{}\n}}\n", name, params, statements),
                }
            }
        }
    }

    fn compile_statement(&mut self, statement: &Statement) -> String {
        match statement {
            Statement::Print(expr) => todo!(),
            Statement::Expression(expr) => todo!(),
            Statement::Let {
                token,
                expression,
                mutable,
            } => todo!(),
            Statement::AssertEq(left, right) => {
                format!(
                    "if ({} != {}) {{\nthrow new Error(\"{} != {}\");\n}}\n",
                    self.compile_expression(left),
                    self.compile_expression(right),
                    // TODO: Replace with source not compiled form
                    self.compile_expression(left),
                    self.compile_expression(right),
                )
            }
        }
    }

    fn compile_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::WithBlock(expr) => self.compile_expression_with_block(expr),
            Expression::WithoutBlock(expr) => self.compile_expression_without_block(expr),
        }
    }

    fn compile_expression_with_block(&mut self, expr: &ExpressionWithBlock) -> String {
        todo!()
    }

    fn compile_expression_without_block(&mut self, expr: &ExpressionWithoutBlock) -> String {
        match expr {
            ExpressionWithoutBlock::Binary {
                left,
                operator,
                right,
            } => format!(
                "{} {} {}",
                self.compile_expression_without_block(left),
                operator.lexeme,
                self.compile_expression_without_block(right)
            ),
            ExpressionWithoutBlock::Call { callee, arguments } => todo!(),
            ExpressionWithoutBlock::Lambda { parameters, body } => todo!(),
            ExpressionWithoutBlock::Grouping(expr) => {
                format!("({})", self.compile_expression_without_block(expr))
            }
            ExpressionWithoutBlock::Literal(literal) => self.compile_literal(literal),
            ExpressionWithoutBlock::Unary { operator, right } => todo!(),
            ExpressionWithoutBlock::Variable(identifier) => {
                format!("{}", identifier.value.clone().unwrap())
            }
            ExpressionWithoutBlock::Assignment {
                name,
                value,
                operator,
            } => todo!(),
            ExpressionWithoutBlock::Html { name, inner } => todo!(),
            ExpressionWithoutBlock::FString { chunks } => todo!(),
        }
    }

    fn compile_literal(&mut self, literal: &Literal) -> String {
        match literal {
            Literal::Number(value) => format!("{}", value),
            Literal::String(value) => format!("\"{}\"", value),
            Literal::Identifier(identifier) => identifier.to_string(),
            Literal::True => "true".to_string(),
            Literal::False => "false".to_string(),
        }
    }
}
