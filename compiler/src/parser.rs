use thiserror::Error;

use crate::expression::Expression;
use crate::statement::Statement;
use crate::token::{Literal, Token, TokenType};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("syntax error: {1}")]
    SyntaxError(Token, String),
}
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Vec<Option<Statement>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if self.match_token(&[TokenType::NewLine]) {
                self.advance();
                continue;
            }

            statements.push(self.declaration());
        }
        statements
    }

    fn declaration(&mut self) -> Option<Statement> {
        let statement = if self.match_token(&[TokenType::Let]) {
            self.let_declaration()
        } else {
            self.statement()
        };

        match statement {
            Ok(statement) => Some(statement),
            Err(e) => {
                println!("error: {}", e);
                self.synchronize();
                None
            }
        }
    }

    fn let_declaration(&mut self) -> Result<Statement, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name")?.clone();

        let initializer = match self.match_token(&[TokenType::Equal]) {
            true => Some(self.expression()?),
            false => None,
        };

        self.consume(TokenType::NewLine, "Expect new line after variable declaration")?;
        Ok(Statement::Let(name, initializer))
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        match self.match_token(&[TokenType::Print]) {
            true => self.print_statement(),
            false => self.expression_statement(),
        }
    }

    fn synchronize(&mut self) {
        todo!();
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '('")?;
        let value = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')'")?;
        self.consume(TokenType::NewLine, "Expect newline")?;
        Ok(Statement::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let value = self.expression()?;
        self.consume(TokenType::NewLine, "Expect new line")?;
        Ok(Statement::Expression(value))
    }

    fn expression(&mut self) -> Result<Expression, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.expression()?;
            expr = Expression::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expression::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expression::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expression::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expression::Unary {
                operator,
                right: right.into(),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expression::Literal(Literal::False));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expression::Literal(Literal::True));
        }
        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expression::Literal(self.previous().value.clone().unwrap()));
        }
        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expression::Variable(self.previous().clone()));
        }
        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expression::Grouping(expr.into()));
        }

        Err(ParseError::SyntaxError(self.peek().clone(), "Expect expression.".to_string()))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, ParseError> {
        match self.check(&token_type) {
            true => Ok(self.advance()),
            false => Err(ParseError::SyntaxError(self.peek().clone(), message.to_string())),
        }
    }

    fn match_token(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&mut self, token_type: &TokenType) -> bool {
        match self.is_at_end() {
            true => false,
            false => self.peek().token_type == *token_type,
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }
}
