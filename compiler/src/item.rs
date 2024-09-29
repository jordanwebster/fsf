use itertools::Itertools;

use crate::expression::{BlockExpression, ExpressionWithBlock};

pub enum Item {
    Function { name: String, body: BlockExpression }
}

impl Item {
    pub fn compile(&self) -> String {
        match self {
            Self::Function { name, body } => {
                let statements = body.statements
                    .iter()
                    .map(|s| s.compile())
                    .join("");
                let body = match body.expr {
                    Some(ref expr) => format!("{}\nreturn {}\n", statements, expr.compile()),
                    None => statements,
                };
                format!("func {}() {{\n{}}}", name, body)
            }
        }
    }
}