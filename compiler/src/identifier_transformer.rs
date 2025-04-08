use crate::compilers::{Module, Program};
use crate::expression::{BlockExpression, Expression, ExpressionWithBlock, ExpressionWithoutBlock};
use crate::item::Item;
use crate::parse_module;
use crate::statement::Statement;
use crate::token::Literal;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

pub fn walk_ast(program: &mut Program, visitor: &mut impl AstVisitor) {
    visitor.visit_program(program);

    for module in program {
        walk_module(module, visitor);
    }
}

fn walk_module(module: &mut Module, visitor: &mut impl AstVisitor) {
    visitor.visit_module(module);

    for item in &mut module.items {
        walk_item(item, visitor);
    }
}

fn walk_item(item: &mut Item, visitor: &mut impl AstVisitor) {
    visitor.visit_item(item);

    match item {
        Item::Function { body, .. } => walk_block(body, visitor),
        Item::Component { body, .. } => walk_block(body, visitor),
        Item::Import { .. } => (),
    }
}

fn walk_block(block: &mut BlockExpression, visitor: &mut impl AstVisitor) {
    for statement in &mut block.statements {
        walk_statement(statement, visitor);
    }
    if let Some(expr) = &mut block.expr {
        walk_expression_without_block(expr, visitor);
    }
}

fn walk_statement(statement: &mut Statement, visitor: &mut impl AstVisitor) {
    visitor.visit_statement(statement);

    match statement {
        Statement::Print(expression) => walk_expression(expression, visitor),
        Statement::Expression(expression) => walk_expression(expression, visitor),
        Statement::Let { expression, .. } => walk_expression(expression, visitor),
        Statement::AssertEq(left, right) => {
            walk_expression(left, visitor);
            walk_expression(right, visitor);
        }
    }
}

fn walk_expression(expr: &mut Expression, visitor: &mut impl AstVisitor) {
    match expr {
        Expression::WithBlock(expr) => walk_expression_with_block(expr, visitor),
        Expression::WithoutBlock(expr) => walk_expression_without_block(expr, visitor),
    }
}

fn walk_expression_with_block(expr: &mut ExpressionWithBlock, visitor: &mut impl AstVisitor) {
    visitor.visit_expression_with_block(expr);

    match expr {
        ExpressionWithBlock::Block(expr) => walk_block(expr, visitor),
        ExpressionWithBlock::If { expr, then, r#else } => {
            walk_expression(expr, visitor);
            walk_block(then, visitor);
            if let Some(r#else) = r#else {
                walk_expression_with_block(r#else, visitor);
            }
        }
    }
}

fn walk_expression_without_block(expr: &mut ExpressionWithoutBlock, visitor: &mut impl AstVisitor) {
    visitor.visit_expression_without_block(expr);

    match expr {
        ExpressionWithoutBlock::Binary { left, right, .. } => {
            walk_expression_without_block(left, visitor);
            walk_expression_without_block(right, visitor);
        }
        ExpressionWithoutBlock::Call { callee, arguments } => {
            walk_expression_without_block(callee, visitor);
            for argument in arguments {
                walk_expression(argument, visitor);
            }
        }
        ExpressionWithoutBlock::Lambda { body, .. } => walk_expression(body, visitor),
        ExpressionWithoutBlock::Grouping(expr) => walk_expression_without_block(expr, visitor),
        ExpressionWithoutBlock::Unary { right, .. } => {
            walk_expression_without_block(right, visitor)
        }
        ExpressionWithoutBlock::Assignment { value, .. } => {
            walk_expression_without_block(value, visitor)
        }
        ExpressionWithoutBlock::Html { inner, .. } => walk_expression(inner, visitor),

        // NO OPS
        ExpressionWithoutBlock::Literal(_) => (),
        ExpressionWithoutBlock::Variable(_) => (),
        ExpressionWithoutBlock::FString { .. } => (),
    }
}

trait AstVisitor {
    fn visit_program(&mut self, _program: &mut Program) {}

    fn visit_module(&mut self, _module: &mut Module) {}

    fn visit_item(&mut self, _item: &mut Item) {}

    fn visit_statement(&mut self, _stmt: &mut Statement) {}

    fn visit_expression_with_block(&mut self, _expr: &mut ExpressionWithBlock) {}

    fn visit_expression_without_block(&mut self, _expr: &mut ExpressionWithoutBlock) {}
}

#[derive(Error, Debug)]
enum IdentifierTransformError {}

pub struct GoIdentifierTransformer {
    current_module: Option<PathBuf>,
    name_map: HashMap<String, String>,
    root: PathBuf,
}

impl GoIdentifierTransformer {
    pub fn new(root: PathBuf) -> Self {
        Self {
            current_module: None,
            name_map: HashMap::new(),
            root,
        }
    }

    pub fn transform(&mut self, program: &mut Program) -> Vec<IdentifierTransformError> {
        walk_ast(program, self);
        vec![]
    }
}

impl AstVisitor for GoIdentifierTransformer {
    fn visit_module(&mut self, module: &mut Module) {
        self.current_module = Some(module.path.clone());
        self.name_map.clear();
    }

    fn visit_item(&mut self, item: &mut Item) {
        match item {
            Item::Function { name, .. } => {
                let new_name = format!(
                    "{}_{}",
                    self.current_module
                        .as_ref()
                        .unwrap()
                        .strip_prefix(&self.root)
                        .unwrap()
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .join("_")
                        .to_uppercase(),
                    name
                );
                name.clear();
                name.push_str(&new_name);
            }
            Item::Import { path } => {
                let name = path.last().unwrap().to_string();
                let module_path = self
                    .current_module
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|p| p.to_string_lossy());
                let mut full_path = module_path.chain(path.into_iter());
                self.name_map
                    .insert(name, full_path.join("_").to_uppercase());
            }
            _ => (),
        }
    }

    fn visit_expression_without_block(&mut self, expr: &mut ExpressionWithoutBlock) {
        match expr {
            ExpressionWithoutBlock::Literal(Literal::Identifier(name)) => {
                if let Some(new_name) = self.name_map.get(name) {
                    name.clear();
                    name.push_str(new_name);
                }
            }
            _ => (),
        }
    }
}

const TEST_RUNNER_TEMPLATE: &str = include_str!("templates/test_runner.fsf");

pub struct TestRunnerTransformer {
    tests: Vec<(String, String)>,
    current_module: Option<PathBuf>,
    root: PathBuf,
}

impl TestRunnerTransformer {
    pub fn new(root: PathBuf) -> Self {
        Self {
            tests: Vec::new(),
            current_module: None,
            root,
        }
    }

    pub fn visit_module(&mut self, module: &mut Module) {
        self.current_module = Some(module.path.clone());
    }

    pub fn transform(&mut self, program: &mut Program) {
        walk_ast(program, self);

        let imports = self
            .tests
            .iter()
            .map(|(module, test)| format!("import {}::{};", module, test))
            .join("\n");

        let tests = self
            .tests
            .iter()
            .map(|(module, test)| format!("__RUN_TEST({}::{}, {});", module, test, test))
            .join("\n");

        let contents = TEST_RUNNER_TEMPLATE
            .replace("/* replace_imports */", &imports)
            .replace("/* replace_tests */", &tests);

        let test_runner = parse_module(contents, PathBuf::from("main.fsf")).unwrap();

        // TODO: Don't lose ownership of Program so that I can append this module
        // TODO: Set up pipelining this transformer with the name transformer before compilation
        // TODO: Implement __RUN_TEST as a special token that handles catching panics and reporting
        // test errors
    }
}

impl AstVisitor for TestRunnerTransformer {
    fn visit_item(&mut self, item: &mut Item) {
        match item {
            Item::Function { name, .. } if name.starts_with("test_") => {
                self.tests.push((
                    self.current_module
                        .as_ref()
                        .unwrap()
                        .strip_prefix(&self.root)
                        .map(|p| p.to_string_lossy())
                        .iter()
                        .join("::"),
                    name.clone(),
                ));
            }
            _ => (),
        }
    }
}
