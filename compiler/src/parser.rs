use thiserror::Error;

use crate::expression::{BlockExpression, Expression, ExpressionWithBlock, ExpressionWithoutBlock};
use crate::item::{Item};
use crate::statement::{MaybeStatement, Statement};
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

    pub fn parse(&mut self) -> Vec<Option<Item>> {
        let mut items = Vec::new();
        while !self.is_at_end() {
            items.push(self.item());
        }
        items
    }

    fn item(&mut self) -> Option<Item> {
        match self.function() {
            Ok(item) => {
                println!("Got item: {:?}", item);
                Some(item)
            },
            Err(error) => {
                // TODO: Without synchronization we can get in an infinite loop here
                println!("Got error: {:?}", error);
                None
            }
        }
    }

    fn function(&mut self) -> Result<Item, ParseError> {
        self.consume(TokenType::Fn, "Expected function declaration")?;
        let name = match self.match_token(&[TokenType::Identifier]) {
            true => Ok(self.previous().clone().lexeme),
            false => Err(ParseError::SyntaxError(self.previous().clone(), "Expected identifier".to_string())),
        }?;

        self.consume(TokenType::LeftParen, "Expected '('")?;
        let mut parameters = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                let token = self.consume(TokenType::Identifier, "Expect parameter name")?;
                parameters.push(token.clone().lexeme);

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expected ')'")?;

        self.consume(TokenType::LeftBrace, "Expected '{'")?;
        let body = self.block_expression()?;
        Ok(Item::Function {name, parameters, body})
    }

    // fn statement(&mut self) -> Option<Statement> {
    //     let statement = match self.maybe_statement() {
    //         Ok(MaybeStatement::Statement(statement)) => Ok(statement),
    //         Ok(MaybeStatement::Expression(expression)) => {
    //             match self.consume(TokenType::Semicolon, "Expect ';' after expression") {
    //                 Ok(_) => Ok(Statement::Expression(expression)),
    //                 Err(error) => Err(error),
    //             }
    //         }
    //         Err(error) => Err(error),
    //     };
    //
    //     match statement {
    //         Ok(statement) => {
    //             println!("parsed statement: {:?}", statement);
    //             Some(statement)
    //         },
    //         Err(e) => {
    //             println!("error: {}", e);
    //             self.synchronize();
    //             None
    //         }
    //     }
    // }

    fn maybe_statement(&mut self) -> Result<MaybeStatement, ParseError> {
        if self.match_token(&[TokenType::Let, TokenType::Print]) {
            match self.previous().token_type {
                TokenType::Let => {
                    let x = Ok(MaybeStatement::Statement(self.let_declaration()?));
                    println!("Parsed let statement: {:?}", x);
                    x
                }
                TokenType::Print => {
                    let x = Ok(MaybeStatement::Statement(self.print_statement()?));
                    println!("Parsed print statement: {:?}", x);
                    x
                },
                ref t => panic!("Unexpected statement type {:?}", t)
            }
        } else {
            let x = Ok(MaybeStatement::Expression(self.expression()?));
            println!("Parsed expression statement: {:?}", x);
            x
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
        if self.match_token(&[TokenType::LeftBrace, TokenType::If]) {
            Ok(Expression::WithBlock(self.expression_with_block()?))
        } else {
            Ok(Expression::WithoutBlock(self.expression_without_block()?))
        }
    }

    fn expression_with_block(&mut self) -> Result<ExpressionWithBlock, ParseError> {
        match self.previous().token_type {
            TokenType::LeftBrace => Ok(ExpressionWithBlock::Block(self.block_expression()?.into())),
            TokenType::If => self.if_expression(),
            ref t => {
                panic!("Unexpected token for expression with block: {:?}", t);
            },
        }
    }

    fn block_expression(&mut self) -> Result<BlockExpression, ParseError> {
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
                            Expression::WithBlock(e) => {
                                statements.push(Statement::Expression(Expression::WithBlock(e)));
                            }
                        }
                    }
                }
            }
        }

        Ok(BlockExpression { statements, expr })
    }

    fn if_expression(&mut self) -> Result<ExpressionWithBlock, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::LeftBrace, "Expect '{'")?;
        let then = self.block_expression()?;
        let r#else = match self.check(&TokenType::Else) {
            true => {
                self.advance();
                if !self.match_token(&[TokenType::LeftBrace, TokenType::If]) {
                    return Err(ParseError::SyntaxError(self.previous().clone(), "Expect block expression or if expression".to_string()));
                }
                Some(Box::new(self.expression_with_block()?))
            }
            false => None,
        };
        Ok(ExpressionWithBlock::If { expr: expr.into(), then: then.into(), r#else })
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

        self.call()
    }

    fn call(&mut self) -> Result<ExpressionWithoutBlock, ParseError> {
        let mut expr = self.primary()?;

        while self.match_token(&[TokenType::LeftParen]) {
            expr = self.finish_call(expr)?;
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: ExpressionWithoutBlock) -> Result<ExpressionWithoutBlock, ParseError> {
        let mut arguments: Vec<Expression> = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after arguments")?;

        Ok(ExpressionWithoutBlock::Call{ callee: callee.into(), arguments})
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
