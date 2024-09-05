use crate::token::{Literal, Token};


pub enum Expression {
    Binary { left: Box<Expression>, operator: Token, right: Box<Expression> },
    Grouping(Box<Expression>),
    Literal(Literal),
    Unary { operator: Token, right: Box<Expression> },
}