use crate::expression::{BlockExpression, Expression, ExpressionWithBlock, ExpressionWithoutBlock};
use crate::item::Item;
use crate::statement::Statement;
use crate::targets::{Module, Program};

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
        Item::Struct { .. } => (),
        Item::Import { .. } => (),
        Item::TestRunner => (),
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
        Statement::RunTest { function_name, .. } => walk_expression(function_name, visitor),
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
        ExpressionWithoutBlock::Index { callee, index } => {
            walk_expression_without_block(callee, visitor);
            walk_expression(index, visitor)
        }
        ExpressionWithoutBlock::Field { callee, .. } => {
            walk_expression_without_block(callee, visitor);
        }
        ExpressionWithoutBlock::Lambda { body, .. } => walk_expression(body, visitor),
        ExpressionWithoutBlock::Grouping(expr) => walk_expression_without_block(expr, visitor),
        ExpressionWithoutBlock::Unary { right, .. } => {
            walk_expression_without_block(right, visitor)
        }
        ExpressionWithoutBlock::Assignment { value, .. } => {
            walk_expression_without_block(value, visitor)
        }
        ExpressionWithoutBlock::Html { inner, .. } => {
            for expression in inner {
                walk_expression(expression, visitor)
            }
        }
        ExpressionWithoutBlock::Array { elements, .. } => {
            for expression in elements {
                walk_expression(expression, visitor)
            }
        }
        ExpressionWithoutBlock::Tuple { elements, .. } => {
            for expression in elements {
                walk_expression(expression, visitor)
            }
        }
        ExpressionWithoutBlock::Struct { fields, .. } => {
            for (_, expression) in fields {
                walk_expression(expression, visitor)
            }
        }

        // NO OPS
        ExpressionWithoutBlock::Literal(_) => (),
        ExpressionWithoutBlock::Variable(_) => (),
        ExpressionWithoutBlock::FString { .. } => (),
        ExpressionWithoutBlock::RawJs(_) => (),
        ExpressionWithoutBlock::RawGo(_) => (),
    }
}

pub trait AstVisitor {
    fn visit_program(&mut self, _program: &mut Program) {}

    fn visit_module(&mut self, _module: &mut Module) {}

    fn visit_item(&mut self, _item: &mut Item) {}

    fn visit_statement(&mut self, _stmt: &mut Statement) {}

    fn visit_expression_with_block(&mut self, _expr: &mut ExpressionWithBlock) {}

    fn visit_expression_without_block(&mut self, _expr: &mut ExpressionWithoutBlock) {}
}
