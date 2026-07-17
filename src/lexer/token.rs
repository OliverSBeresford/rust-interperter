use phf::phf_map;
use std::fmt;
use heck::ToShoutySnakeCase;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Static, // Added Static keyword for static methods
}

// static perfect-hash map from string -> Keyword
static KEYWORDS: phf::Map<&'static str, Keyword> = phf_map! {
    "and" => Keyword::And,
    "class" => Keyword::Class,
    "else" => Keyword::Else,
    "false" => Keyword::False,
    "for" => Keyword::For,
    "fun" => Keyword::Fun,
    "if" => Keyword::If,
    "nil" => Keyword::Nil,
    "or" => Keyword::Or,
    "print" => Keyword::Print,
    "return" => Keyword::Return,
    "super" => Keyword::Super,
    "this" => Keyword::This,
    "true" => Keyword::True,
    "var" => Keyword::Var,
    "while" => Keyword::While,
    "static" => Keyword::Static,
};

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        KEYWORDS.get(s).copied()
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
