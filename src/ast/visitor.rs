use crate::lexer::token::Token;
use crate::ast::expr::Expr;
use crate::ast::statement::Statement;
use crate::ast::expr::Depth;

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
    fn visit_lambda(&mut self, params: &Vec<Token>, body: &Vec<Statement>) -> T;
    fn visit_get(&mut self, object: &Expr, name: &Token) -> T;

    // Statement visitor methods
    fn visit_expression_statement(&mut self, expression: &Expr) -> T;
    fn visit_print_statement(&mut self, expression: &Expr) -> T;
    fn visit_var_statement(&mut self, name: &Token, initializer: &Option<Expr>) -> T;
    fn visit_block_statement(&mut self, statements: &[Statement]) -> T;
    fn visit_if_statement(&mut self, condition: &Expr, then_branch: &Statement, else_branch: &Option<Box<Statement>>) -> T;
    fn visit_while_statement(&mut self, condition: &Expr, body: &Statement) -> T;
    fn visit_function_statement(&mut self, statement: &Statement) -> T;
    fn visit_return_statement(&mut self, keyword: &Token, value: &Option<Expr>) -> T;
    fn visit_class_statement(&mut self, name: &Token, methods: &[Statement]) -> T;

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
            Expr::Lambda { params, body } => self.visit_lambda(params, body),
            Expr::Get { object, name } => self.visit_get(object, name),
        }
    }

    fn visit_statement(&mut self, statement: &Statement) -> T {
        match statement {
            Statement::Expression { expression } => self.visit_expression_statement(expression),
            Statement::Print { expression } => self.visit_print_statement(expression),
            Statement::Var { name, initializer } => self.visit_var_statement(name, initializer),
            Statement::Block { statements } => self.visit_block_statement(statements),
            Statement::If { condition, then_branch, else_branch } => self.visit_if_statement(condition, then_branch, else_branch),
            Statement::While { condition, body } => self.visit_while_statement(condition, body),
            Statement::Function { .. } => self.visit_function_statement(statement),
            Statement::Return { keyword, value } => self.visit_return_statement(keyword, value),
            Statement::Class { name, methods } => self.visit_class_statement(name, methods),
        }
    }
}