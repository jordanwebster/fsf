use crate::expression::{Expression, ExpressionWithBlock, ExpressionWithoutBlock, FStringChunk};
use crate::item::Item;
use crate::statement::Declaration;
use crate::statement::Statement;
use crate::targets::Program;
use crate::token::{Literal, TokenType};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const MAIN_BOOTSTRAP: &str = include_str!("../bootstrap/js_bootstrap.js");
const REACT_BOOTSTRAP_HEADER: &str = include_str!("../bootstrap/react_bootstrap_header.js");
const REACT_BOOTSTRAP_FOOTER: &str = include_str!("../bootstrap/react_bootstrap_footer.js");

pub struct JsTarget {}

impl JsTarget {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile(
        &mut self,
        program: Program,
        compile_dir: &Path,
        is_exec_mode: bool,
    ) -> Result<()> {
        let output_path = match is_exec_mode {
            true => compile_dir.join("main.js"),
            false => compile_dir.join("index.jsx"),
        };
        let mut output_file = File::create(&output_path)?;
        if !is_exec_mode {
            output_file.write_all(REACT_BOOTSTRAP_HEADER.as_bytes())?;
        }

        for module in program {
            let output = module
                .items
                .into_iter()
                .map(|item| self.compile_item(item))
                .join("");

            output_file.write_all(output.as_bytes())?;
        }

        if is_exec_mode {
            output_file.write_all(MAIN_BOOTSTRAP.as_bytes())?;
        } else {
            // TODO: Write a component map so we know which component to hydrate the root
            output_file.write_all(REACT_BOOTSTRAP_FOOTER.as_bytes())?;
        }
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

    fn compile_item(&mut self, item: Item) -> String {
        match item {
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

                let params = parameters.iter().map(|p| p.name.clone()).join(", ");

                // TODO: Typechecker will ensure we are returning HTML
                let html_expr = body.expr.unwrap();
                format!(
                    "function {}({}) {{\n{}\n return ({});\n}}\n",
                    name,
                    params,
                    statements,
                    self.compile_expression(html_expr)
                )
            }
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
            Item::Struct { .. } => "".to_string(),
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
                declaration,
                expression,
                ..
            } => {
                let declaration_str = format!(
                    "let {} =",
                    match declaration {
                        Declaration::Name(name) => name.value.unwrap().to_string(),
                        Declaration::Tuple(names) | Declaration::Array(names) => format!(
                            "[{}]",
                            names
                                .into_iter()
                                .map(|t| t.value.unwrap().to_string())
                                .join(",")
                        ),
                    }
                );
                match expression {
                    Expression::WithoutBlock(expr) => format!(
                        "{} {};\n",
                        declaration_str,
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
                                    "{}{} {};\n",
                                    statements_str,
                                    declaration_str,
                                    self.compile_expression(expr)
                                )
                            } else {
                                statements_str
                            }
                        }
                        ExpressionWithBlock::If { .. } => todo!(),
                    },
                }
            }
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
            ExpressionWithoutBlock::Variable(identifier) => {
                format!("{}", identifier.value.unwrap())
            }
            ExpressionWithoutBlock::Assignment {
                name,
                value,
                operator,
            } => match operator.token_type {
                TokenType::Equal => {
                    format!("{} = {}", name.lexeme, self.compile_expression(*value))
                }
                TokenType::PlusEqual => {
                    format!("{} += {}", name.lexeme, self.compile_expression(*value))
                }
                _ => panic!("Unexpected token type in assignment: {}", operator.lexeme),
            },
            ExpressionWithoutBlock::Html {
                name,
                inner,
                attributes,
            } => {
                let react_attribute_map =
                    HashMap::from([("onclick".to_string(), "onClick".to_string())]);
                let attrs = attributes
                    .into_iter()
                    .map(|(name, expr)| {
                        format!(
                            "{}={{{}}}",
                            react_attribute_map
                                .get(&name.lexeme)
                                .unwrap_or(&name.lexeme),
                            self.compile_expression(expr)
                        )
                    })
                    .join(" ");
                let children = inner
                    .into_iter()
                    .map(|e| match e {
                        Expression::WithoutBlock(ExpressionWithoutBlock::Html { .. }) => {
                            self.compile_expression(e)
                        }
                        _ => format!("{{{}}}", self.compile_expression(e)),
                    })
                    .join("\n");
                format!("<{} {}>{}</{}>", name.lexeme, attrs, children, name.lexeme)
            }
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
            ExpressionWithoutBlock::Array { elements } => {
                let elements = elements
                    .into_iter()
                    .map(|e| self.compile_expression(e))
                    .join(", ");
                format!("[{}]", elements)
            }
            ExpressionWithoutBlock::Index { callee, index } => {
                format!(
                    "{}[{}]",
                    self.compile_expression(*callee),
                    self.compile_expression(*index)
                )
            }
            ExpressionWithoutBlock::Field { callee, field } => {
                format!("{}.{}", self.compile_expression(*callee), field.lexeme)
            }
            ExpressionWithoutBlock::Tuple { elements } => {
                let elements = elements
                    .into_iter()
                    .map(|e| self.compile_expression(e))
                    .join(", ");
                format!("[{}]", elements)
            }
            ExpressionWithoutBlock::Struct { fields, .. } => {
                format!(
                    "{{\n{}}}",
                    fields
                        .into_iter()
                        .map(|(field, value)| format!(
                            "{}: {}",
                            field.lexeme,
                            self.compile_expression(value)
                        ))
                        .join(",\n")
                )
            }
            ExpressionWithoutBlock::RawJs(code) => format!("{}\n", code),
            ExpressionWithoutBlock::RawGo(_) => "".to_string(),
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
