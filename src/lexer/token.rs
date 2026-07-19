use std::fmt;
use heck::ToShoutySnakeCase;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    And,
    Break,
    Class,
    Continue,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Static,
    Super,
    This,
    ThisClass,
    True,
    Var,
    While,
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        match s {
            "and" => Some(Keyword::And),
            "break" => Some(Keyword::Break),
            "class" => Some(Keyword::Class),
            "continue" => Some(Keyword::Continue),
            "else" => Some(Keyword::Else),
            "false" => Some(Keyword::False),
            "for" => Some(Keyword::For),
            "fun" => Some(Keyword::Fun),
            "if" => Some(Keyword::If),
            "nil" => Some(Keyword::Nil),
            "or" => Some(Keyword::Or),
            "print" => Some(Keyword::Print),
            "return" => Some(Keyword::Return),
            "super" => Some(Keyword::Super),
            "this" => Some(Keyword::This),
            "This" => Some(Keyword::ThisClass),
            "true" => Some(Keyword::True),
            "var" => Some(Keyword::Var),
            "while" => Some(Keyword::While),
            "static" => Some(Keyword::Static),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Eof,
    // Literals
    String,
    Number,
    // One or two character tokens.
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    // Identifiers
    Identifier,
    // Keywords
    Keyword(Keyword),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For Keyword(kw) we want "AND"/"CLASS"/etc.  For other variants rely on Debug name.
        // format!("{:?}", ...) produces "LeftBrace", "Number", etc. Converting that to
        // shouty snake yields "LEFT_BRACE", "NUMBER", ...
        let out = match self {
            TokenType::Keyword(kw) => format!("{:?}", kw).to_shouty_snake_case(),
            _ => format!("{:?}", self).to_shouty_snake_case(),
        };
        f.write_str(&out)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::String(s) => f.write_str(s),
            Literal::Number(n) => {
                // If the value is an integer (no fractional part) print one decimal place
                // Otherwise print the float normally.
                if n.fract() == 0.0 {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

// implement Display for Token so format!("{}", token) or token.to_string() works
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.literal {
            None | Some(Literal::Nil) | Some(Literal::Boolean(false)) | Some(Literal::Boolean(true)) => write!(f, "{} {} null", self.token_type, self.lexeme),
            Some(lit) => write!(f, "{} {} {}", self.token_type, self.lexeme, lit),
        }
    }
}
