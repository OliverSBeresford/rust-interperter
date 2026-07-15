use crate::lexer::token::Token;
use crate::ast::expr::Expr;
use crate::ast::statement::Statement;
use crate::ast::expr::Depth;
use std::rc::Rc;

pub trait Visitor<T> {
    // Expression visitor methods
    fn visit_literal(&mut self, value: &Token) -> T;
    fn visit_grouping(&mut self, expression: &Expr) -> T;
    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> T;
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_variable(&mut self, name: &Token, depth: &Depth) -> T;
    fn visit_assign(&mut self, name: &Token, value: &Expr, depth: &Depth) -> T;
    fn visit_logical_or(&mut self, left: &Expr, right: &Expr) -> T;
    fn visit_logical_and(&mut self, left: &Expr, right: &Expr) -> T;
    fn visit_call(&mut self, callee: &Expr, paren: &Token, arguments: &Vec<Expr>) -> T;
    fn visit_lambda(&mut self, params: &Vec<Token>, body: Vec<Rc<Statement>>) -> T;
    fn visit_get(&mut self, object: &Expr, name: &Token) -> T;
    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> T;
    fn visit_this(&mut self, keyword: &Token, depth: &Depth) -> T;

    // Statement visitor methods
    fn visit_expression_statement(&mut self, expression: &Expr) -> T;
    fn visit_print_statement(&mut self, expression: &Expr) -> T;
    fn visit_var_statement(&mut self, name: &Token, initializer: &Option<Expr>) -> T;
    fn visit_block_statement(&mut self, statements: Vec<Rc<Statement>>) -> T;
    fn visit_if_statement(&mut self, condition: &Expr, then_branch: Rc<Statement>, else_branch: Option<Rc<Statement>>) -> T;
    fn visit_while_statement(&mut self, condition: &Expr, body: Rc<Statement>) -> T;
    fn visit_function_statement(&mut self, statement: Rc<Statement>) -> T;
    fn visit_return_statement(&mut self, keyword: &Token, value: &Option<Expr>) -> T;
    fn visit_class_statement(&mut self, name: &Token, methods: Vec<Rc<Statement>>) -> T;

    fn visit_expression(&mut self, expr: &Expr) -> T {
        match expr {
            Expr::Literal { value } => self.visit_literal(value),
            Expr::Grouping { expression } => self.visit_grouping(expression),
            Expr::Unary { operator, right } => self.visit_unary(operator, right),
            Expr::Binary { left, operator, right } => self.visit_binary(left, operator, right),
            Expr::Variable { name, depth } => self.visit_variable(name, depth),
            Expr::Assign { name, value, depth } => self.visit_assign(name, value, depth),
            Expr::LogicOr { left, right } => self.visit_logical_or(left, right),
            Expr::LogicAnd { left, right } => self.visit_logical_and(left, right),
            Expr::Call { callee, arguments , paren} => self.visit_call(callee, paren, arguments),
            Expr::Lambda { params, body } => self.visit_lambda(params, body.clone()),
            Expr::Get { object, name } => self.visit_get(object, name),
            Expr::Set { object, name, value } => self.visit_set(object, name, value),
            Expr::This { keyword, depth } => self.visit_this(keyword, depth),
        }
    }

    fn visit_statement(&mut self, statement: Rc<Statement>) -> T {
        match &*statement.clone() {
            Statement::Expression { expression } => self.visit_expression_statement(expression),
            Statement::Print { expression } => self.visit_print_statement(expression),
            Statement::Var { name, initializer } => self.visit_var_statement(name, initializer),
            Statement::Block { statements } => self.visit_block_statement(statements.clone()),
            Statement::If { condition, then_branch, else_branch } => self.visit_if_statement(condition, then_branch.clone(), else_branch.clone()),
            Statement::While { condition, body } => self.visit_while_statement(condition, body.clone()),
            Statement::Function { .. } => self.visit_function_statement(statement),
            Statement::Return { keyword, value } => self.visit_return_statement(keyword, value),
            Statement::Class { name, methods } => self.visit_class_statement(name, methods.clone()),
        }
    }
}
