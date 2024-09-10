use crate::token::{Literal, Token};


#[derive(Debug)]
pub enum Expression {
    Binary { left: Box<Expression>, operator: Token, right: Box<Expression> },
    Grouping(Box<Expression>),
    Literal(Literal),
    Unary { operator: Token, right: Box<Expression> },
    Variable(Token),
}

impl Expression {
    pub fn compile(&self) -> String {
        match self {
            Self::Literal(Literal::Number(number)) => format!("{}", number),
            Self::Literal(Literal::String(string)) => format!("\"{}\"", string),
            Self::Literal(Literal::Identifier(identifier)) => identifier.to_string(),
            Self::Literal(Literal::True) => "true".to_string(),
            Self::Literal(Literal::False) => "false".to_string(),
            Self::Variable(identifier) => format!("{}", identifier.value.clone().unwrap()),
            _ => todo!(),
        }
    }
}