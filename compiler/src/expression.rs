use crate::statement::Statement;
use crate::token::{Literal, Token, TokenType};
use itertools::Itertools;

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

impl Expression {
    pub fn compile(&self) -> String {
        match self {
            Self::WithBlock(e) => e.compile(),
            Self::WithoutBlock(e) => e.compile(),
        }
    }
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

impl ExpressionWithoutBlock {
    pub fn compile(&self) -> String {
        match self {
            Self::Binary {
                left,
                operator,
                right,
            } => format!("{} {} {}", left.compile(), operator.lexeme, right.compile()),
            Self::Call { callee, arguments } => format!(
                "{}({})",
                callee.compile(),
                arguments.iter().map(|e| e.compile()).join(", ")
            ),
            Self::Grouping(expression) => format!("({})", expression.compile()),
            Self::Literal(Literal::Number(number)) => format!("{}", number),
            Self::Literal(Literal::String(string)) => format!("\"{}\"", string),
            Self::Literal(Literal::Identifier(identifier)) => identifier.to_string(),
            Self::Literal(Literal::True) => "true".to_string(),
            Self::Literal(Literal::False) => "false".to_string(),
            Self::Unary { operator, right } => todo!(),
            Self::Variable(identifier) => format!("{}", identifier.value.clone().unwrap()),
            Self::Assignment {
                name,
                value,
                operator,
            } => match operator.token_type {
                TokenType::Equal => format!("{} = {}", name.lexeme, value.compile()),
                TokenType::PlusEqual => format!("{} += {}", name.lexeme, value.compile()),
                _ => panic!("Unexpected token type in assignment: {}", operator.lexeme),
            },
            Self::FString { chunks } => {
                let format_string = chunks
                    .iter()
                    .map(|chunk| match chunk {
                        FStringChunk::Literal(string) => string,
                        FStringChunk::Identifier(string) => "%v", // TODO: Use correct specifier based on type
                    })
                    .join("");
                let arguments = chunks
                    .iter()
                    .filter_map(|chunk| match chunk {
                        FStringChunk::Literal(_) => None,
                        FStringChunk::Identifier(string) => Some(string),
                    })
                    .join(", ");
                format!("fmt.Sprintf(\"{}\", {})", format_string, arguments)
            }
            Self::Html { name, inner } => {
                format!("<{}>\n{}\n</{}>", name.lexeme, inner.compile(), name.lexeme)
            }
            Self::Lambda { parameters, body } => {
                let params = parameters
                    .iter()
                    .map(|p| {
                        format!(
                            "{} {}",
                            p.name,
                            match p.type_annotation.as_deref() {
                                Some("int") => "int",
                                Some("str") => "string",
                                Some(other) => other,
                                // TODO: Add proper type inference
                                None => "int",
                            }
                        )
                    })
                    .join(", ");
                match &**body {
                    Expression::WithoutBlock(expression) => {
                        // TODO: Add proper type inference
                        format!(
                            "func({}) int {{\nreturn {};\n}}\n",
                            params,
                            expression.compile()
                        )
                    }
                    Expression::WithBlock(expression) => todo!(),
                }
            }
        }
    }
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

impl ExpressionWithBlock {
    pub fn compile(&self) -> String {
        match self {
            Self::Block(block) => block.statements.iter().map(|s| s.compile()).join(""),
            Self::If { expr, then, r#else } => {
                // TODO: Handle the case there is a dangling expression
                let mut s = format!(
                    "if ({}) {{\n{}}}",
                    expr.compile(),
                    then.statements.iter().map(|s| s.compile()).join("")
                );
                if let Some(r#else) = r#else {
                    s = format!("{} else {{\n{}}}\n", s, r#else.compile());
                }
                s
            }
        }
    }
}
