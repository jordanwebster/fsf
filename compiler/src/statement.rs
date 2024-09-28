use itertools::Itertools;
use crate::expression::Expression;
use crate::expression::ExpressionWithBlock;
use crate::token::Token;

#[derive(Debug)]
pub enum MaybeStatement {
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug)]
pub enum Statement {
    Print(Expression),
    Expression(Expression),
    Let(Token, Expression),
}

impl Statement {
    pub fn compile(&self) -> String {
        match self {
            Self::Print(expr) => format!("fmt.Println({})\n", expr.compile()),
            Self::Expression(expr) => format!("{}\n", expr.compile()),
            Self::Let(token, expr) => match expr {
                Expression::WithoutBlock(expr) => format!("{} := {}\n", token.value.clone().unwrap(), expr.compile()),
                Expression::WithBlock(expr) => match expr {
                    ExpressionWithBlock::Block {statements, expr} => {
                        let statements_str = statements.iter().map(|stmt| stmt.compile()).join("");
                        if let Some(expr) = expr {
                            format!("{}{} := {}\n", statements_str, token.value.clone().unwrap(), expr.compile())
                        } else {
                            statements_str
                        }
                    }
                }
            },
        }
    }
}
