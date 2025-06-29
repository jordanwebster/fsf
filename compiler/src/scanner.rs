use crate::token::{Literal, Token, TokenType};

pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token(&mut tokens);
        }

        tokens.push(Token::new(TokenType::EOF, "".to_string(), None, self.line));

        tokens
    }

    pub fn scan_token(&mut self, tokens: &mut Vec<Token>) {
        let c = self.advance();
        match c {
            '(' => tokens.push(Token::new(
                TokenType::LeftParen,
                c.to_string(),
                None,
                self.line,
            )),
            ')' => tokens.push(Token::new(
                TokenType::RightParen,
                c.to_string(),
                None,
                self.line,
            )),
            '{' => tokens.push(Token::new(
                TokenType::LeftBrace,
                c.to_string(),
                None,
                self.line,
            )),
            '}' => tokens.push(Token::new(
                TokenType::RightBrace,
                c.to_string(),
                None,
                self.line,
            )),
            '|' => tokens.push(Token::new(TokenType::Pipe, c.to_string(), None, self.line)),
            ',' => tokens.push(Token::new(TokenType::Comma, c.to_string(), None, self.line)),
            '.' => tokens.push(Token::new(TokenType::Dot, c.to_string(), None, self.line)),
            '-' => {
                if self.match_char('>') {
                    tokens.push(Token::new(
                        TokenType::MinusGreater,
                        "->".to_string(),
                        None,
                        self.line,
                    ));
                } else {
                    tokens.push(Token::new(TokenType::Minus, c.to_string(), None, self.line))
                }
            }
            '+' => {
                if self.match_char('=') {
                    tokens.push(Token::new(
                        TokenType::PlusEqual,
                        "+=".to_string(),
                        None,
                        self.line,
                    ));
                } else {
                    tokens.push(Token::new(TokenType::Plus, c.to_string(), None, self.line))
                }
            }
            ':' => {
                if self.match_char(':') {
                    tokens.push(Token::new(
                        TokenType::ColonColon,
                        c.to_string(),
                        None,
                        self.line,
                    ));
                } else {
                    tokens.push(Token::new(TokenType::Colon, c.to_string(), None, self.line))
                }
            }
            ';' => tokens.push(Token::new(
                TokenType::Semicolon,
                c.to_string(),
                None,
                self.line,
            )),
            '*' => tokens.push(Token::new(TokenType::Star, c.to_string(), None, self.line)),
            '!' => {
                if self.match_char('=') {
                    tokens.push(Token::new(
                        TokenType::BangEqual,
                        "!=".to_string(),
                        None,
                        self.line,
                    ));
                } else {
                    tokens.push(Token::new(
                        TokenType::Bang,
                        "!".to_string(),
                        None,
                        self.line,
                    ));
                }
            }
            '=' => {
                if self.match_char('=') {
                    tokens.push(Token::new(
                        TokenType::EqualEqual,
                        "==".to_string(),
                        None,
                        self.line,
                    ));
                } else {
                    tokens.push(Token::new(
                        TokenType::Equal,
                        "=".to_string(),
                        None,
                        self.line,
                    ));
                }
            }
            '<' => {
                if self.match_char('=') {
                    tokens.push(Token::new(
                        TokenType::LessEqual,
                        "<=".to_string(),
                        None,
                        self.line,
                    ));
                } else if self.match_char('/') {
                    tokens.push(Token::new(
                        TokenType::LessSlash,
                        "</".to_string(),
                        None,
                        self.line,
                    ))
                } else {
                    tokens.push(Token::new(
                        TokenType::Less,
                        "<".to_string(),
                        None,
                        self.line,
                    ));
                }
            }
            '>' => {
                if self.match_char('=') {
                    tokens.push(Token::new(
                        TokenType::GreaterEqual,
                        ">=".to_string(),
                        None,
                        self.line,
                    ));
                } else {
                    tokens.push(Token::new(
                        TokenType::Greater,
                        ">".to_string(),
                        None,
                        self.line,
                    ));
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('>') {
                    tokens.push(Token::new(
                        TokenType::SlashGreater,
                        "/>".to_string(),
                        None,
                        self.line,
                    ))
                } else {
                    tokens.push(Token::new(
                        TokenType::Slash,
                        "/".to_string(),
                        None,
                        self.line,
                    ));
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(tokens, false),
            '`' => self.string(tokens, true),
            '0'..='9' => self.number(tokens),
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(tokens),
            '[' => tokens.push(Token::new(
                TokenType::LeftSquareBracket,
                c.to_string(),
                None,
                self.line,
            )),
            ']' => tokens.push(Token::new(
                TokenType::RightSquareBracket,
                c.to_string(),
                None,
                self.line,
            )),
            '@' => self.raw_code(tokens),
            _ => todo!("Handle unexpected tokens"),
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn string(&mut self, tokens: &mut Vec<Token>, is_fstring: bool) {
        let terminator = match is_fstring {
            true => '`',
            false => '"',
        };
        while self.peek() != terminator && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            println!("Unterminated string.");
            return;
        }

        // Consume the closing "
        self.advance();

        let value = self.source[self.start + 1..self.current - 1].to_string();
        tokens.push(Token::new(
            match is_fstring {
                true => TokenType::FString,
                false => TokenType::String,
            },
            self.source[self.start..self.current].to_string(),
            Some(Literal::String(value)),
            self.line,
        ));
    }

    fn number(&mut self, tokens: &mut Vec<Token>) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        tokens.push(Token::new(
            TokenType::Number,
            self.source[self.start..self.current].to_string(),
            Some(Literal::Number(
                self.source[self.start..self.current]
                    .parse::<f64>()
                    .unwrap(),
            )),
            self.line,
        ));
    }

    fn identifier(&mut self, tokens: &mut Vec<Token>) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();
        let token_type = match text.as_str() {
            // Match keywords
            "let" => TokenType::Let,
            "mut" => TokenType::Mut,
            "print" => TokenType::Print,
            "fn" => TokenType::Fn,
            "cmpnt" => TokenType::Cmpnt,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "assert_eq" => TokenType::AssertEq,
            "import" => TokenType::Import,
            "struct" => TokenType::Struct,

            // Builtins.
            "__RUN_TEST" => TokenType::RunTest,
            "__TEST_RUNNER" => TokenType::TestRunner,
            _ => TokenType::Identifier,
        };

        tokens.push(Token::new(
            token_type,
            text.clone(),
            Some(Literal::Identifier(text)),
            self.line,
        ));
    }

    fn raw_code(&mut self, tokens: &mut Vec<Token>) {
        // Consume the @
        self.advance();

        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();
        match text.as_str() {
            "@js" => tokens.push(Token::new(TokenType::RawJs, text.clone(), None, self.line)),
            "@go" => tokens.push(Token::new(TokenType::RawGo, text.clone(), None, self.line)),
            _ => panic!("Unexpected raw code: {}", text),
        }
    }
}
