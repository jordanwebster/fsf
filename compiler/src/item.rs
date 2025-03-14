use itertools::Itertools;

use crate::expression::BlockExpression;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: String,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function {
        name: String,
        parameters: Vec<Parameter>,
        body: BlockExpression,
        return_type: String,
    },
    // TODO: Merge into function once we have typing
    Component {
        name: String,
        parameters: Vec<Parameter>,
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
                return_type,
            } => {
                let statements = body.statements.iter().map(|s| s.compile()).join("");
                let params = parameters
                    .iter()
                    .map(|p| {
                        format!(
                            "{} {}",
                            p.name,
                            match p.type_annotation.as_str() {
                                "int" => "int",
                                "str" => "string",
                                other => other,
                            }
                        )
                    })
                    .join(", ");
                match body.expr {
                    Some(ref expr) => format!(
                        "func {}({}) {} {{\n{}\nreturn {}\n}}\n",
                        name,
                        params,
                        return_type,
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
                let params = parameters
                    .iter()
                    .map(|p| {
                        format!(
                            "{} {}",
                            p.name,
                            match p.type_annotation.as_str() {
                                "int" => "int",
                                "str" => "string",
                                other => other,
                            }
                        )
                    })
                    .join(", ");
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
