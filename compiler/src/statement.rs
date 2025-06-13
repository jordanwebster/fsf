use crate::expression::Expression;
use crate::token::Token;

#[derive(Debug)]
pub enum MaybeStatement {
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum Declaration {
    Name(Token),
    Array(Vec<Token>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Print(Expression),
    Expression(Expression),
    Let {
        declaration: Declaration,
        expression: Expression,
        mutable: bool,
    },
    AssertEq(Expression, Expression),
    RunTest {
        test_name: Token,
        function_name: Box<Expression>,
    },
}
