use crate::statement::Statement;
use crate::token::{Literal, Token};

#[derive(Debug)]
pub enum Expression {
    WithBlock(ExpressionWithBlock),
    WithoutBlock(ExpressionWithoutBlock),
}

impl Expression {
    pub fn compile(&self) -> String {
        match self {
            Self::WithBlock(e) => e.compile(),
            Self::WithoutBlock(e) => e.compile(),
        }
    }
}

#[derive(Debug)]
pub enum ExpressionWithoutBlock {
    Binary { left: Box<ExpressionWithoutBlock>, operator: Token, right: Box<ExpressionWithoutBlock> },
    Grouping(Box<ExpressionWithoutBlock>),
    Literal(Literal),
    Unary { operator: Token, right: Box<ExpressionWithoutBlock> },
    Variable(Token),
    Assignment { name: Token, value: Box<ExpressionWithoutBlock> },
}

impl ExpressionWithoutBlock {
    pub fn compile(&self) -> String {
        match self {
            Self::Binary { left, operator, right } => format!("{} {} {}", left.compile(), operator.lexeme, right.compile()),
            Self::Grouping(expression) => todo!(),
            Self::Literal(Literal::Number(number)) => format!("{}", number),
            Self::Literal(Literal::String(string)) => format!("\"{}\"", string),
            Self::Literal(Literal::Identifier(identifier)) => identifier.to_string(),
            Self::Literal(Literal::True) => "true".to_string(),
            Self::Literal(Literal::False) => "false".to_string(),
            Self::Unary { operator, right} => todo!(),
            Self::Variable(identifier) => format!("{}", identifier.value.clone().unwrap()),
            Self::Assignment { name, value } => format!("{} = {}", name.lexeme, value.clone().compile()),
        }
    }
}

#[derive(Debug)]
pub enum ExpressionWithBlock {
    Block { statements: Vec<Statement>, expr: Option<ExpressionWithoutBlock>}
}

impl ExpressionWithBlock {
    pub fn compile(&self) -> String {
        match self {
            Self::Block {statements, expr} => { todo!() }
        }
    }
}
