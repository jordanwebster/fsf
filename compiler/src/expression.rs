use crate::statement::Statement;
use crate::token::{Literal, Token};

#[derive(Debug, Clone)]
pub enum FStringChunk {
    Literal(String),
    Identifier(String),
}

#[derive(Debug, Clone)]
pub enum Expression {
    WithBlock(ExpressionWithBlock),
    WithoutBlock(ExpressionWithoutBlock),
}

#[derive(Debug, Clone)]
pub struct LambdaParameter {
    pub name: String,
    pub type_annotation: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ExpressionWithoutBlock {
    Binary {
        left: Box<ExpressionWithoutBlock>,
        operator: Token,
        right: Box<ExpressionWithoutBlock>,
    },
    Call {
        callee: Box<ExpressionWithoutBlock>,
        arguments: Vec<Expression>,
    },
    Lambda {
        parameters: Vec<LambdaParameter>,
        body: Box<Expression>,
    },
    Grouping(Box<ExpressionWithoutBlock>),
    Literal(Literal),
    Unary {
        operator: Token,
        right: Box<ExpressionWithoutBlock>,
    },
    Variable(Token),
    Assignment {
        name: Token,
        value: Box<ExpressionWithoutBlock>,
        operator: Token,
    },
    Html {
        name: Token,
        inner: Box<Expression>,
    },
    FString {
        chunks: Vec<FStringChunk>,
    },
}

#[derive(Debug, Clone)]
pub enum ExpressionWithBlock {
    Block(Box<BlockExpression>),
    If {
        expr: Box<Expression>,
        then: Box<BlockExpression>,
        r#else: Option<Box<ExpressionWithBlock>>,
    },
}

#[derive(Debug, Clone)]
pub struct BlockExpression {
    pub statements: Vec<Statement>,
    pub expr: Option<ExpressionWithoutBlock>,
}

impl From<ExpressionWithoutBlock> for Expression {
    fn from(expr: ExpressionWithoutBlock) -> Self {
        Expression::WithoutBlock(expr)
    }
}

impl From<ExpressionWithBlock> for Expression {
    fn from(expr: ExpressionWithBlock) -> Self {
        Expression::WithBlock(expr)
    }
}
