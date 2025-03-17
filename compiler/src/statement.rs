use crate::expression::Expression;
use crate::expression::ExpressionWithBlock;
use crate::token::Token;
use itertools::Itertools;

#[derive(Debug)]
pub enum MaybeStatement {
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Print(Expression),
    Expression(Expression),
    Let {
        token: Token,
        expression: Expression,
        mutable: bool,
    },
}

impl Statement {
    pub fn compile(&self) -> String {
        match self {
            Self::Print(expr) => format!("fmt.Println({})\n", expr.compile()),
            Self::Expression(expr) => format!("{}\n", expr.compile()),
            Self::Let {
                token,
                expression,
                mutable,
            } => match expression {
                Expression::WithoutBlock(expr) => {
                    format!("{} := {}\n", token.value.clone().unwrap(), expr.compile())
                }
                Expression::WithBlock(expr) => match expr {
                    ExpressionWithBlock::Block(block) => {
                        let statements_str =
                            block.statements.iter().map(|stmt| stmt.compile()).join("");
                        if let Some(ref expr) = block.expr {
                            format!(
                                "{}{} := {}\n",
                                statements_str,
                                token.value.clone().unwrap(),
                                expr.compile()
                            )
                        } else {
                            statements_str
                        }
                    }
                    ExpressionWithBlock::If { expr, then, r#else } => todo!(),
                },
            },
        }
    }
}
