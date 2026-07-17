use std::fmt;
use std::iter::Peekable;
use std::str::CharIndices;

use crate::{Keyword, Literal, Token, TokenType};

pub struct TokenArray {
    pub tokens: Vec<Token>,
}

impl TokenArray {
    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }
}

impl fmt::Display for TokenArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for token in &self.tokens {
            writeln!(f, "{}", token)?;
        }
        Ok(())
    }
}

pub fn scan(input: &str) -> TokenArray {
    let mut scanner = Scanner::new(input);
    scanner.scan_tokens();

    // Check for lexical errors, then return tokens
    if scanner.had_error() {
        println!("{}", scanner.tokens);
        std::process::exit(65);
    }
    scanner.tokens
}

struct Scanner<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
    start: usize,
    current: usize,
    lexical_error: bool,
    pub tokens: TokenArray,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            line: 1,
            start: 0,
            current: 0,
            lexical_error: false,
            tokens: TokenArray { tokens: Vec::new() },
        }
    }

    // Start a token
    fn begin_token(&mut self) {
        self.start = self.current;
    }

    // Advance the scanner by one character and return it
    fn advance(&mut self) -> Option<char> {
        if let Some((byte_index, ch)) = self.chars.next() {
            self.current = byte_index + ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    // Get the current lexeme being scanned
    fn get_lexeme(&self) -> &str {
        &self.input[self.start..self.current]
    }

    // Create a new token and add it to the tokens vector
    fn make_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let lexeme = self.get_lexeme();
        let token = Token::new(token_type, lexeme.to_string(), literal, self.line);
        self.tokens.push(token);
    }

    fn scan_tokens(&mut self) {
        while self.peek().is_some() {
            self.scan_token();
        }
        // Add EOF token at the end
        self.begin_token();
        self.make_token(TokenType::Eof, None);
    }

    fn scan_token(&mut self) {
        self.begin_token();

        // Make sure there's a character to process
        let c = self.advance();
        if c.is_none() {
            return;
        }
        let c = c.unwrap();

        match c {
            // Multi-char tokens
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(TokenType::EqualEqual, None);
                } else {
                    self.make_token(TokenType::Equal, None);
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(TokenType::BangEqual, None);
                } else {
                    self.make_token(TokenType::Bang, None);
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(TokenType::LessEqual, None);
                } else {
                    self.make_token(TokenType::Less, None);
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(TokenType::GreaterEqual, None);
                } else {
                    self.make_token(TokenType::Greater, None);
                }
            }

            // Single-char tokens
            '(' => self.make_token(TokenType::LeftParen, None),
            ')' => self.make_token(TokenType::RightParen, None),
            '{' => self.make_token(TokenType::LeftBrace, None),
            '}' => self.make_token(TokenType::RightBrace, None),
            ',' => self.make_token(TokenType::Comma, None),
            '.' => self.make_token(TokenType::Dot, None),
            '-' => self.make_token(TokenType::Minus, None),
            '+' => self.make_token(TokenType::Plus, None),
            ';' => self.make_token(TokenType::Semicolon, None),
            '*' => self.make_token(TokenType::Star, None),

            // whitespace & newlines
            '\n' => {
                self.line += 1;
            }
            c if c.is_whitespace() => { /* skip other whitespace */ }

            // Comments and division
            '/' => {
                if self.peek() == Some('/') {
                    // consume rest of line
                    while let Some(&(_, next_char)) = self.chars.peek() {
                        if next_char == '\n' {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.make_token(TokenType::Slash, None);
                }
            }
            // Literals
            '"' => {
                self.scan_string();
            }
            c if c.is_digit(10) => {
                self.scan_number();
            }

            // Identifiers
            c if c.is_alphabetic() || c == '_' => {
                self.scan_word();
            }

            // unexpected characters
            other => {
                eprintln!("[line {}] Error: Unexpected character: {}", self.line, other);
                self.lexical_error = true;
            }
        };
    }

    // Method to scan words (identifiers and keywords)
    fn scan_word(&mut self) {
        // Look ahead to consume all alphanumeric characters
        while let Some(next_char) = self.peek() {
            if next_char.is_alphanumeric() || next_char == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let lexeme = self.get_lexeme();
        let token_type = if let Some(keyword) = Keyword::from_str(lexeme) {
            TokenType::Keyword(keyword)
        } else {
            TokenType::Identifier
        };
        let literal = match token_type {
            TokenType::Keyword(Keyword::True) => Some(Literal::Boolean(true)),
            TokenType::Keyword(Keyword::False) => Some(Literal::Boolean(false)),
            TokenType::Keyword(Keyword::Nil) => Some(Literal::Nil),
            _ => None,
        };
        self.make_token(token_type, literal);
    }

    // Method to scan number literals
    fn scan_number(&mut self) {
        // Look ahead to consume all digits
        while let Some(next_char) = self.peek() {
            if next_char.is_digit(10) || next_char == '.' {
                self.advance();
            } else {
                break;
            }
        }
        let number_literal: f64 = self
            .get_lexeme()
            .parse()
            .expect("Failed to parse number literal");
        self.make_token(TokenType::Number, Some(Literal::Number(number_literal)));
    }

    // Method to scan string literals
    fn scan_string(&mut self) {
        while let Some(c) = self.advance() {
            if c == '"' {
                // Consume the closing quote
                let string_literal = &self.input[self.start + 1..self.current - 1];
                self.make_token(
                    TokenType::String,
                    Some(Literal::String(string_literal.to_string())),
                );
                return;
            }
        }

        // If we reach the end of the input without finding a closing quote, it's an error
        eprintln!("[line {}] Scanning Error: Unterminated string.", self.line);
        self.lexical_error = true;
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, ch)| ch)
    }

    fn had_error(&self) -> bool {
        self.lexical_error
    }
}
