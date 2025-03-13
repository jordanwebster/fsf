use itertools::Itertools;

use crate::expression::BlockExpression;

#[derive(Debug, Clone)]
pub enum Item {
    Function {
        name: String,
        parameters: Vec<String>,
        body: BlockExpression,
    },
    // TODO: Merge into function once we have typing
    Component {
        name: String,
        parameters: Vec<String>,
        body: BlockExpression,
    },
}

impl Item {
    pub fn compile(&self) -> String {
        match self {
            Self::Function {
                name,
                parameters,
                body,
            } => {
                let statements = body.statements.iter().map(|s| s.compile()).join("");
                let params = parameters.iter().map(|p| format!("{p} int")).join(", ");
                // TODO: Add proper typing and don't assume all parameters are ints
                match body.expr {
                    Some(ref expr) => format!(
                        "func {}({}) int {{\n{}\nreturn {}\n}}\n",
                        name,
                        params,
                        statements,
                        expr.compile()
                    ),
                    None => format!("func {}({}) {{\n{}\n}}\n", name, params, statements),
                }
            }
            Self::Component {
                name,
                parameters,
                body,
            } => {
                let statements = body.statements.iter().map(|s| s.compile()).join("");
                let params = parameters.iter().map(|p| format!("{p} string")).join(", ");
                // TODO: Add proper typing and don't assume all parameters are ints
                match body.expr {
                    Some(ref expr) => format!(
                        "func {}({}) string {{\n{}\nreturn `{}`\n}}\n",
                        name,
                        params,
                        statements,
                        expr.compile()
                    ),
                    None => format!("func {}({}) {{\n{}\n}}\n", name, params, statements),
                }
            }
        }
    }
}
