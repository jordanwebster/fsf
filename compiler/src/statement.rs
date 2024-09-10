use crate::expression::Expression;
use crate::token::Token;

#[derive(Debug)]
pub enum Statement {
    Print(Expression),
    Expression(Expression),
    Let(Token, Option<Expression>),
}

impl Statement {
    pub fn compile(&self) -> String {
        match self {
            Self::Print(expr) => format!("fmt.Println({})", expr.compile()),
            Self::Expression(expr) => expr.compile(),
            Self::Let(token, expr) => match expr {
                Some(expr) => format!("{} := {}", token.value.clone().unwrap(), expr.compile()),
                None => todo!(),
            },
        }
    }
}
