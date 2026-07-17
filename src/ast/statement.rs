use std::rc::Rc;

use crate::{
    ast::Expr,
    lexer::Token
};

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
        static_fields: Vec<Rc<Statement>>,
        static_methods: Vec<Rc<Statement>>,
    },
}
