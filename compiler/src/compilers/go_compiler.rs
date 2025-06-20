use crate::compilers::Program;
use crate::expression::{Expression, ExpressionWithBlock, ExpressionWithoutBlock, FStringChunk};
use crate::item::Item;
use crate::statement::{Declaration, Statement};
use crate::token::{Literal, TokenType};
use anyhow::Result;
use itertools::Itertools;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct GoCompiler {
    building_html: bool,
}

impl GoCompiler {
    pub fn new() -> Self {
        Self {
            building_html: false,
        }
    }

    pub fn compile(&mut self, program: Program, compile_dir: &Path) -> Result<()> {
        for module in program {
            let output = module
                .items
                .into_iter()
                .map(|item| self.compile_item(item))
                .join("");
            let mut output_path = compile_dir.join(module.path.file_stem().unwrap());
            output_path.set_extension("go");
            let mut output_file = File::create(&output_path)?;
            output_file.write_all("package main\n".as_bytes())?;
            // TODO: Propagate this information up via the parser
            if output.contains("fmt.") {
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
                let return_type = match return_type.as_deref() {
                    Some("int") => "int",
                    Some("str") => "string",
                    Some("void") => "",
                    Some(other) => other,
                    None => "",
                };

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
                    None => format!(
                        "func {}({}) {} {{\n{}\n}}\n",
                        name,
                        params,
                        // This is a HACK, because we have "string" types and injecting raw code.
                        // We need to be able to conditionally compile JS or Go to ensure typing
                        // is correct.
                        return_type,
                        statements
                    ),
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
                        "func {}({}) string {{\n{}\nreturn {}\n}}\n",
                        name,
                        params,
                        statements,
                        self.compile_expression(expr)
                    ),
                    None => format!("func {}({}) {{\n{}\n}}\n", name, params, statements),
                }
            }
            Item::Import { .. } => "".to_string(),
            Item::TestRunner => r#"
                func runTest(test func(), name string) {
                    defer func() {
                        if err := recover(); err != nil {
                            fmt.Print(" fail\n")
                        }
                    }()

                    fmt.Printf("%s...", name)
                    test()
                    fmt.Print(" pass\n")
                }
                "#
            .to_string(),
        }
    }

    fn compile_statement(&mut self, statement: Statement) -> String {
        match statement {
            Statement::Print(expr) => format!("fmt.Println({})\n", self.compile_expression(expr)),
            Statement::Expression(expr) => format!("{}\n", self.compile_expression(expr)),
            Statement::Let {
                declaration,
                expression,
                ..
            } => {
                // TODO: Create a unique temporary variable name generator
                let tmp_var = "x_tmp";
                let declaration_str = match declaration {
                    Declaration::Name(ref name) => format!("{} := ", name.clone().value.unwrap()),
                    Declaration::Array(_) => format!("{} := ", tmp_var),
                    Declaration::Tuple(ref names) => {
                        format!(
                            "{} := ",
                            names.iter().cloned().map(|t| t.value.unwrap()).join(", ")
                        )
                    }
                };

                let destructuring_str = match declaration {
                    Declaration::Name(_) => "".to_string(),
                    Declaration::Array(names) => format!(
                        "{}\n",
                        names
                            .into_iter()
                            .enumerate()
                            .map(|(i, name)| format!(
                                "{} := {}[{}]",
                                name.value.unwrap(),
                                tmp_var,
                                i
                            ))
                            .join("\n")
                    ),
                    // Note that for now we are only interested in the case
                    // of destructing tuples returned from functions so there is
                    // nothing to do. When we add more general tuples as structs
                    // there will be something to do here.
                    Declaration::Tuple(_) => "".to_string(),
                };
                match expression {
                    Expression::WithoutBlock(expr) => {
                        format!(
                            "{} {}\n{}",
                            declaration_str,
                            self.compile_expression(Expression::WithoutBlock(expr)),
                            destructuring_str
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
                                    "{}{} {}\n{}",
                                    statements_str,
                                    declaration_str,
                                    self.compile_expression(expr),
                                    destructuring_str
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
                    "if ({} != {}) {{\npanic(`{} != {}`)}}\n",
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
            ExpressionWithoutBlock::FString { chunks } => {
                let format_string = chunks
                    .iter()
                    .map(|chunk| match chunk {
                        FStringChunk::Literal(string) => string,
                        FStringChunk::Identifier(_string) => "%v", // TODO: Use correct specifier based on type
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
            ExpressionWithoutBlock::Html {
                name,
                inner,
                attributes,
            } => {
                // TODO: Preserve newlines for HTML. Newlines between inline elements get
                // converted into spaces.
                let mut output = String::new();

                let require_cleanup = if self.building_html {
                    false
                } else {
                    output.push_str("func() string {\nbuilder := NewHTMLBuilder()\n");
                    self.building_html = true;
                    true
                };

                output.push_str(&format!("builder.beginElement(\"{}\")\n", name.lexeme));
                for (name, value_expr) in attributes {
                    output.push_str(&format!(
                        "builder.addAttribute(\"{}\", {})\n",
                        name.lexeme,
                        self.compile_expression(value_expr).replace("\n", ""),
                    ));
                }
                for expression in inner {
                    match expression {
                        Expression::WithoutBlock(ExpressionWithoutBlock::FString { .. })
                        | Expression::WithoutBlock(ExpressionWithoutBlock::Literal(
                            Literal::String(_),
                        )) => {
                            let compiled_expression = self.compile_expression(expression);
                            output
                                .push_str(&format!("builder.addString({})\n", compiled_expression));
                        }
                        _ => {
                            output.push_str(&self.compile_expression(expression));
                        }
                    }
                }
                output.push_str("builder.endElement()\n");

                if require_cleanup {
                    output.push_str("return builder.build()\n}()");
                    self.building_html = false;
                }

                output
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
                            "func({}) int {{\nreturn {}\n}}\n",
                            params,
                            self.compile_expression_without_block(expression)
                        )
                    }
                    Expression::WithBlock(_expression) => todo!(),
                }
            }
            ExpressionWithoutBlock::Array { elements } => {
                let count = elements.len();
                // TODO: Add proper type information
                let type_ = "int";
                let elements = elements
                    .into_iter()
                    .map(|e| self.compile_expression(e))
                    .join(", ");
                format!("[{}]{}{{{}}}", count, type_, elements)
            }
            ExpressionWithoutBlock::Index { callee, index } => {
                format!(
                    "{}[{}]",
                    self.compile_expression(*callee),
                    self.compile_expression(*index)
                )
            }
            // Note that because we only care about tuples returned from functions right now
            // we can just use Go's multiple returns
            ExpressionWithoutBlock::Tuple { elements } => elements
                .into_iter()
                .map(|e| self.compile_expression(e))
                .join(", "),
            ExpressionWithoutBlock::RawJs(_) => "".to_string(),
            ExpressionWithoutBlock::RawGo(code) => format!("{}\n", code),
        }
    }
}
