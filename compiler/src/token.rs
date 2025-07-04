#[derive(Clone, Debug)]
pub enum Literal {
    True,
    False,
    Identifier(String),
    String(String),
    Number(f64),
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Identifier(identifier) => write!(f, "{}", identifier),
            Self::String(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftSquareBracket,
    RightSquareBracket,
    Comma,
    Dot,
    Minus,
    Pipe,
    Plus,
    Colon,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Less,
    LessEqual,
    LessSlash,
    Greater,
    GreaterEqual,
    PlusEqual,
    SlashGreater,
    MinusGreater,
    ColonColon,

    // Literals.
    Identifier,
    String,
    Number,
    FString,

    // Keywords.
    Let,
    Mut,
    False,
    True,
    Print,
    Fn,
    Cmpnt,
    If,
    Else,
    AssertEq,
    Import,
    Struct,

    // Builtins.
    RunTest,
    TestRunner,

    // Escape hatches.
    RawJs,
    RawGo,

    EOF,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub value: Option<Literal>,
    line: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        value: Option<Literal>,
        line: usize,
    ) -> Token {
        Token {
            token_type,
            lexeme,
            value,
            line,
        }
    }
}
