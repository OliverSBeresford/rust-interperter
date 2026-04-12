use crate::ast::{Expr, Statement, Depth};
use crate::lexer::token::Keyword::{False, Nil, True};
use crate::lexer::token::{Keyword, Literal, Token, TokenType};
use crate::parser::error::ParseError;
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    // Report a parse error
    fn error<T>(token: &Token, message: &str) -> Result<T, ParseError> {
        if token.token_type == TokenType::Eof {
            Err(ParseError::new(
                token.line,
                format!("Error at end: {}", message),
            ))
        } else {
            Err(ParseError::new(
                token.line,
                format!("Error at '{}': {}", token.lexeme, message),
            ))
        }
    }

    // A synchronization method to recover from errors
    fn synchronize(&mut self) {
        self.consume_any();

        while let Some(token) = self.current_token() {
            if token.token_type == TokenType::Semicolon {
                self.consume_any();
                return;
            }

            match token.token_type {
                TokenType::Keyword(kw) => match kw {
                    Keyword::Class
                    | Keyword::Fun
                    | Keyword::Var
                    | Keyword::For
                    | Keyword::If
                    | Keyword::While
                    | Keyword::Print
                    | Keyword::Return => {
                        return;
                    }
                    _ => {}
                },
                _ => {}
            }

            self.consume_any();
        }
    }

    // Return the current token and advance the parser
    fn advance(&mut self) -> Result<Token, ParseError> {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            Ok(token)
        } else {
            Self::error(&self.tokens[self.tokens.len() - 1], "Unexpected end of input")
        }
    }

    // Get the current token without advancing the parser
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    // Check if the current token is of one of the expected types
    fn check(&self, expected: &[TokenType]) -> bool {
        if let Some(token) = self.current_token() {
            return expected.contains(&token.token_type);
        }
        false
    }

    // Consume a token of the expected type, or return an error
    fn consume(&mut self, expected: TokenType, error_message: &str) -> Result<Token, ParseError> {
        let current_token = self.advance()?;

        // If the current token is not of the expected type or doesn't exist, return an error
        if current_token.token_type != expected {
            return Self::error(&current_token, error_message);
        }

        Ok(current_token)
    }

    fn consume_any(&mut self) {
        let _ = self.advance();
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();

        // Parse statements until the end of the token stream (-1 for EOF)
        while self.current < self.tokens.len() - 1 {
            let statement = self.declaration();
            if let Err(e) = &statement {
                eprintln!("{}", e);
            } else if let Ok(statement) = statement {
                statements.push(statement);
            }
        }

        statements
    }

    fn declaration(&mut self) -> Result<Statement, ParseError> {
        // For now, only parse variable declarations and statements
        if self.check(&[TokenType::Keyword(Keyword::Var)]) {
            return self.var_declaration().or_else(|err: ParseError| {
                self.synchronize(); // Synchronize on error
                Err(err)
            });
        } else if self.check(&[TokenType::Keyword(Keyword::Fun)]) {
            // Consume the 'fun' keyword
            self.advance()?;

            // Function declaration
            return self
                .function_declaration("function")
                .or_else(|err: ParseError| {
                    self.synchronize(); // Synchronize on error
                    Err(err)
                });
        } else if self.check(&[TokenType::Keyword(Keyword::Class)]) {
            // Class declaration
            return self
                .class_declaration()
                .or_else(|err: ParseError| {
                    self.synchronize(); // Synchronize on error
                    Err(err)
                });
        }
        self.statement().or_else(|err: ParseError| {
            self.synchronize(); // Synchronize on error
            Err(err)
        })
    }

    fn var_declaration(&mut self) -> Result<Statement, ParseError> {
        // Consume the 'var' keyword
        let _var_token = self.advance();

        // Consume the variable name
        let name_token = self.consume(TokenType::Identifier, "Expect variable name.")?;

        // Optional initializer
        let initializer = if self.check(&[TokenType::Equal]) {
            // Consume the '=' token
            let _equal_token = self.advance();

            // Parse the initializer expression
            Some(self.expression()?)
        } else {
            None
        };

        // Consume the semicolon
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Statement::Var {
            name: name_token,
            initializer,
        })
    }

    fn function_declaration(&mut self, kind: &str) -> Result<Statement, ParseError> {
        // Consume the function name
        let name_token = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;

        // Consume the '(' token
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        // Parse the parameters
        let mut params: Vec<Token> = Vec::new();
        if !self.check(&[TokenType::RightParen]) {
            loop {
                // Consume the parameter name
                let param_token = self.consume(TokenType::Identifier, "Expect parameter name.")?;
                params.push(param_token);

                if !self.check(&[TokenType::Comma]) {
                    break;
                }
                // Consume the ',' token
                let _comma_token = self.advance();
            }
        }

        // Consume the ')' token
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        // Consume the '{' token
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;

        // Parse the function body
        let Statement::Block { statements: body } = self.block_statement()? else {
            return Self::error(&name_token, "Expect function body.");
        };

        Ok(Statement::Function { name: name_token, params, body })
    }

    fn class_declaration(&mut self) -> Result<Statement, ParseError> {
        // Consume the 'class' keyword
        let _class_token = self.advance();

        // Consume the class name
        let name_token = self.consume(TokenType::Identifier, "Expect class name.")?;

        // Consume the '{' token
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

        // Parse the methods in the class
        let mut methods: Vec<Statement> = Vec::new();
        while !self.check(&[TokenType::RightBrace]) && self.current < self.tokens.len() - 1 {
            methods.push(self.function_declaration("method")?);
        }

        // Consume the '}' token
        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Statement::Class { name: name_token, methods })
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        // Parse different kinds of statements based on the current token
        if self.check(&[TokenType::Keyword(Keyword::Print)]) {
            return self.print_statement();
        } else if self.check(&[TokenType::LeftBrace]) {
            return self.block_statement();
        } else if self.check(&[TokenType::Keyword(Keyword::If)]) {
            return self.if_statement();
        } else if self.check(&[TokenType::Keyword(Keyword::While)]) {
            return self.while_statement();
        } else if self.check(&[TokenType::Keyword(Keyword::For)]) {
            return self.for_statement();
        } else if self.check(&[TokenType::Keyword(Keyword::Return)]) {
            return self.return_statement();
        } else {
            return self.expression_statement();
        }
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        // Consume the 'print' keyword
        let _print_token = self.advance();

        // Parse the expression to be printed
        let expression = self.expression()?;

        // Consume the semicolon at the end of the print statement
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;

        Ok(Statement::Print { expression })
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expression = self.expression()?;

        // Consume the semicolon at the end of the expression statement
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;

        Ok(Statement::Expression { expression })
    }

    fn block_statement(&mut self) -> Result<Statement, ParseError> {
        // Consume the '{' token if it's there
        if self.check(&[TokenType::LeftBrace]) {
            let _left_brace_token = self.advance()?;
        }

        // Create a vector to hold the statements in the block
        let mut statements: Vec<Statement> = Vec::new();

        // Parse statements until we find a '}'
        while !self.check(&[TokenType::RightBrace]) && self.current < self.tokens.len() - 1 {
            let declaration = self.declaration()?;
            statements.push(declaration);
        }

        // Consume the '}' token
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(Statement::Block { statements })
    }

    fn if_statement(&mut self) -> Result<Statement, ParseError> {
        // Consume the 'if' keyword
        let _if_token = self.advance();

        // Parse the condition expression and consume the parentheses
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        // Parse the then branch statement
        let then_branch = self.statement()?;

        // Optional else branch
        let else_branch: Option<Box<Statement>> = if self.check(&[TokenType::Keyword(Keyword::Else)]) {
            // Consume the 'else' keyword
            let _else_token = self.advance();

            // Parse the else branch statement
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Statement, ParseError> {
        // Consume the 'while' keyword
        let _while_token = self.advance();

        // Parse the condition expression (decides whether to run the loop) and consume the parentheses
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;

        // Parse the body statement (the thing that gets repeated)
        let body: Statement = self.statement()?;

        Ok(Statement::While { condition, body: Box::new(body) })
    }

    // This is not a new kind of statement, we are just desugaring a for loop into a while loop and some extra statements
    fn for_statement(&mut self) -> Result<Statement, ParseError> {
        // Consume the 'for' keyword
        let _for_token = self.advance();

        // Consume the '(' token
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        // Parse the initializer (can be a variable declaration, expression statement, or empty)
        let initializer = if self.check(&[TokenType::Semicolon]) {
            self.consume_any();
            None
        } else if self.check(&[TokenType::Keyword(Keyword::Var)]) {
            // Initializer is a variable declaration
            Some(self.var_declaration()?)
        } else {
            // Initializer is an expression statement
            Some(self.expression_statement()?)
        };

        // Parse the condition (can be empty, which defaults to 'true')
        let condition = if !self.check(&[TokenType::Semicolon]) {
            self.expression()?
        } else {
            // Consume the ';' token
            Expr::Literal {
                value: Token {
                    token_type: TokenType::Keyword(Keyword::True),
                    lexeme: "true".to_string(),
                    literal: Some(Literal::Boolean(true)),
                    line: 0,
                },
            }
        };
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        // Parse the increment (can be empty)
        let increment = if !self.check(&[TokenType::RightParen]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        // Parse the body statement
        let mut body: Statement = self.statement()?;

        if increment.is_some() {
            // Combine what's in the body with the increment expression
            body = Statement::Block {
                statements: vec![body.into(), Statement::Expression {
                    expression: increment.unwrap(),
                }.into()],
            };
        }

        // Create a while statement with the condition specified and the body we made (with the increment)
        body = Statement::While {
            condition,
            body: body.into(),
        };

        // If there is an initializer, add it as a statement before the while loop
        if initializer.is_some() {
            body = Statement::Block {
                statements: vec![initializer.unwrap(), body],
            };
        }

        Ok(body)
    }

    fn return_statement(&mut self) -> Result<Statement, ParseError> {
        // Consume the 'return' keyword
        let keyword = self.advance()?;

        // Optional return value
        let value = if !self.check(&[TokenType::Semicolon]) {
            Some(self.expression()?)
        } else {
            None
        };

        // Consume the semicolon at the end of the return statement
        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;

        Ok(Statement::Return { keyword, value })
    }

    pub fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logic_or()?;

        if self.check(&[TokenType::Equal]) {
            let equals = self.advance()?;
            let value = self.assignment()?;

            // If the left-hand side is a variable, create an assignment expression
            if let Expr::Variable { name, .. } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                    depth: Depth::Unresolved, // Depth will be resolved later
                });
            }

            return Self::error(&equals, "Invalid assignment target.");
        }

        Ok(expr)
    }

    fn logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logic_and()?;

        while self.check(&[TokenType::Keyword(Keyword::Or)]) {
            let _operator = self.advance()?;
            let right = self.logic_and()?;

            expr = Expr::LogicOr {
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.check(&[TokenType::Keyword(Keyword::And)]) {
            let _operator = self.advance()?;
            let right = self.equality()?;

            expr = Expr::LogicAnd {
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // Lowest precedence, going up from here
    fn equality(&mut self) -> Result<Expr, ParseError> {
        // Create the left-hand side expression
        let mut expr = self.comparison()?;

        while self.check(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            // Consume the operator and store it
            let operator = self.advance()?;
            let right = self.comparison()?;

            // Create a new binary expression with the left and right expressions
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // A comparison is a term followed by zero or more <, >, <=, >=, each followed by a term, like 1 < 2 >= 3
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        // Create the left-hand side expression (can be a term or above)
        let mut expr = self.term()?;

        while self.check(&[TokenType::Less, TokenType::Greater, TokenType::LessEqual, TokenType::GreaterEqual]) {
            // Consume the operator and store it
            let operator = self.advance()?;
            let right = self.term()?;

            // Create a new binary expression with the left and right expressions
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // A term is a factor followed by zero or more + or -, each followed by a factor, like 1 + 2 - 3
    fn term(&mut self) -> Result<Expr, ParseError> {
        // Create the left-hand side expression (can be a factor or above)
        let mut expr = self.factor()?;

        while self.check(&[TokenType::Minus, TokenType::Plus]) {
            // Consume the operator and store it
            let operator = self.advance()?;
            let right = self.factor()?;

            // Create a new binary expression with the left and right expressions
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // A factor is a unary expression followed by zero or more * or /, each followed by a unary expression, like -4 / 2 * 3
    fn factor(&mut self) -> Result<Expr, ParseError> {
        // Create the left-hand side expression (can be a unary or above)
        let mut expr = self.unary()?;

        while self.check(&[TokenType::Slash, TokenType::Star]) {
            // Consume the operator and store it
            let operator = self.advance()?;
            let right = self.unary()?;

            // Create a new binary expression with the left and right expressions
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // A unary expression is either a primary expression or a unary operator followed by another unary expression, like -!!5
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.check(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.advance()?;
            let right = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.check(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        // Consume the '(' token
        self.advance()?;

        // Scan the arguments
        let mut arguments: Vec<Expr> = Vec::new();

        if !self.check(&[TokenType::RightParen]) {
            loop {
                // Add one argument expression to the list of arguments
                arguments.push(self.expression()?);
                if !self.check(&[TokenType::Comma]) {
                    // If there isn't a comma, there are no more arguments
                    break;
                }
                self.advance()?; // consume the comma (yummy)
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    // A primary expression is either a literal value or a parenthesized expression
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let current_token = self.advance()?;

        match current_token.token_type {
            TokenType::Number | TokenType::String => {
                Ok(Expr::Literal { value: current_token })
            }
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect expression.")?;
                Ok(Expr::Grouping {
                    expression: Box::new(expr),
                })
            }
            TokenType::Keyword(Nil) | TokenType::Keyword(False) | TokenType::Keyword(True) => {
                Ok(Expr::Literal { value: current_token })
            }
            TokenType::Keyword(Keyword::Fun) => self.lambda_expression(),
            TokenType::Identifier => Ok(Expr::Variable { name: current_token, depth: Depth::Unresolved }),
            _ => Self::error(&current_token, "Expect expression."),
        }
    }

    fn lambda_expression(&mut self) -> Result<Expr, ParseError> {
        // Parse the parameters
        self.consume(TokenType::LeftParen, "Expect '(' after 'fun'.")?;

        // Parse the parameters to the lambda
        let mut params: Vec<Token> = Vec::new();
        if !self.check(&[TokenType::RightParen]) {
            loop {
                // Consume the parameter name
                let param_token = self.consume(TokenType::Identifier, "Expect parameter name.")?;
                params.push(param_token);

                if !self.check(&[TokenType::Comma]) {
                    break;
                }
                // Consume the ',' token
                let _comma_token = self.advance()?;
            }
        }

        // Consume the ')' token
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        // Consume the '{' token
        self.consume(TokenType::LeftBrace, "Expect '{' before lambda body.")?;

        // Parse the function body
        let Statement::Block { statements: body } = self.block_statement()? else {
            return Self::error(&params[0], "Expect lambda body.");
        };

        Ok(Expr::Lambda { params, body })
    }
}
