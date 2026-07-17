use std::rc::Rc;

use crate::{
    ast::Statement,
    lexer::Token
};

#[derive(Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    LogicOr {
        left: Box<Expr>,
        // operator: Token, Right now we don't use the operator token, but it's here for completeness
        right: Box<Expr>,
    },
    LogicAnd {
        left: Box<Expr>,
        // operator: Token, Right now we don't use the operator token, but it's here for completeness
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Literal {
        value: Token,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    This {
        keyword: Token,
    },
    THIS {
        keyword: Token,
    },
    Lambda {
        params: Vec<Token>,
        body: Vec<Rc<Statement>>,
    },
}
