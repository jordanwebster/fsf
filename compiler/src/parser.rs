use thiserror::Error;

use crate::expression::{Expression, ExpressionWithBlock, ExpressionWithoutBlock};
use crate::statement::{MaybeStatement, Statement};
use crate::token::{Literal, Token, TokenType};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("syntax error: {1}")]
    SyntaxError(Token, String),

    #[error("grammar error")]
    GrammarError,
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
            statements.push(self.statement());
        }
        statements
    }

    fn statement(&mut self) -> Option<Statement> {
        let statement = match self.maybe_statement() {
            Ok(MaybeStatement::Statement(statement)) => Ok(statement),
            Ok(MaybeStatement::Expression(expression)) => {
                match self.consume(TokenType::Semicolon, "Expect ';' after expression") {
                    Ok(_) => Ok(Statement::Expression(expression)),
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
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

    fn maybe_statement(&mut self) -> Result<MaybeStatement, ParseError> {
        if self.match_token(&[TokenType::Let, TokenType::Print]) {
            match self.previous().token_type {
                TokenType::Let => {
                    Ok(MaybeStatement::Statement(self.let_declaration()?))
                }
                TokenType::Print => Ok(MaybeStatement::Statement(self.print_statement()?)),
                ref t => panic!("Unexpected statement type {:?}", t)
            }
        } else {
            Ok(MaybeStatement::Expression(self.expression()?))
        }
    }

    fn synchronize(&mut self) {
        todo!();
    }

    fn let_declaration(&mut self) -> Result<Statement, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name")?.clone();

        self.consume(TokenType::Equal, "All variables must be initialized")?;
        let initializer = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ; after variable declaration")?;

        Ok(Statement::Let(name, initializer))
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '('")?;
        let value = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')'")?;
        self.consume(TokenType::Semicolon, "Expect ;")?;
        Ok(Statement::Print(value))
    }

    fn expression(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(&[TokenType::LeftBrace]) {
            Ok(Expression::WithBlock(self.expression_with_block()?))
        } else {
            Ok(Expression::WithoutBlock(self.expression_without_block()?))
        }
    }

    fn expression_with_block(&mut self) -> Result<ExpressionWithBlock, ParseError> {
        match self.previous().token_type {
            TokenType::LeftBrace => self.block_expression(),
            _ => todo!(), // this is a compiler error
        }
    }

    fn block_expression(&mut self) -> Result<ExpressionWithBlock, ParseError> {
        let mut expr: Option<ExpressionWithoutBlock> = None;
        let mut statements: Vec<Statement> = Vec::new();

        while !self.match_token(&[TokenType::RightBrace]) {
            match self.maybe_statement()? {
                MaybeStatement::Statement(statement) => statements.push(statement),
                MaybeStatement::Expression(expression) => {
                    if self.check(&TokenType::Semicolon) {
                        self.advance();
                        statements.push(Statement::Expression(expression));
                    } else {
                        match expression {
                            Expression::WithoutBlock(expression) => {
                                expr = Some(expression);
                                if !self.check(&TokenType::RightBrace) {
                                    return Err(ParseError::SyntaxError(self.peek().clone(), "Expect ';' after expression".to_string()));
                                }
                            }
                            Expression::WithBlock(_) => {
                                return Err(ParseError::GrammarError)
                            }
                        }
                    }
                }
            }
        }

        Ok(ExpressionWithBlock::Block { statements, expr })
    }

    fn expression_without_block(&mut self) -> Result<ExpressionWithoutBlock, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<ExpressionWithoutBlock, ParseError> {
        let expr = self.equality()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let ExpressionWithoutBlock::Variable(name) = expr {
                return Ok(ExpressionWithoutBlock::Assignment { name, value: value.into() });
            }

            // TODO: Report (but don't throw error)
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<ExpressionWithoutBlock, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.expression_without_block()?;
            expr = ExpressionWithoutBlock::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<ExpressionWithoutBlock, ParseError> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = ExpressionWithoutBlock::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<ExpressionWithoutBlock, ParseError> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = ExpressionWithoutBlock::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<ExpressionWithoutBlock, ParseError> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = ExpressionWithoutBlock::Binary {
                left: expr.into(),
                operator,
                right: right.into(),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<ExpressionWithoutBlock, ParseError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(ExpressionWithoutBlock::Unary {
                operator,
                right: right.into(),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<ExpressionWithoutBlock, ParseError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(ExpressionWithoutBlock::Literal(Literal::False));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(ExpressionWithoutBlock::Literal(Literal::True));
        }
        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(ExpressionWithoutBlock::Literal(self.previous().value.clone().unwrap()));
        }
        if self.match_token(&[TokenType::Identifier]) {
            return Ok(ExpressionWithoutBlock::Variable(self.previous().clone()));
        }
        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression_without_block()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(ExpressionWithoutBlock::Grouping(expr.into()));
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
