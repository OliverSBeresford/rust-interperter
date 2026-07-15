use crate::ast::statement::Statement;
use crate::lexer::token::Token;
use std::rc::Rc;

#[derive(Debug, Copy, Clone)]
pub enum Depth {
    Unresolved,
    Resolved(usize),
}

#[derive(Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
        depth: Depth,
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
        depth: Depth,
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
        depth: Depth,
    },
    Lambda {
        params: Vec<Token>,
        body: Vec<Rc<Statement>>,
    },
}
