use crate::ast::expr::Expr;
use crate::lexer::token::Token;
use std::rc::Rc;

#[derive(Debug)]
pub enum Statement {
    Expression {
        expression: Expr,
    },
    If {
        condition: Expr,
        then_branch: Rc<Statement>,
        else_branch: Option<Rc<Statement>>,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Rc<Statement>,
    },
    Block {
        statements: Vec<Rc<Statement>>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Rc<Statement>>,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Class {
        name: Token,
        methods: Vec<Rc<Statement>>,
    },
}
