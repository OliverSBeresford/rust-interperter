use crate::ast::Visitor;
use crate::{Expr};
use crate::Token;
use crate::ast::Depth;
use crate::ast::Statement;

type Output = String;

// Pretty-printer
pub struct AstPrinter;

impl AstPrinter {
    pub fn print_expression(&mut self, expr: &Expr) {
        println!("{}", self.visit_expression(expr));
    }

    pub fn print_statement(&mut self, statement: &Statement) {
        println!("{}", self.visit_statement(statement));
    }

    pub fn print_to_string(&mut self, expr: &Expr) -> String {
        self.visit_expression(expr)
    }
}

impl Visitor<Output> for AstPrinter {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Output {
        format!("({} {} {})", operator.lexeme, self.visit_expression(left), self.visit_expression(right))
    }

    fn visit_literal(&mut self, value: &Token) -> Output {
        format!("{}", value.literal.as_ref().unwrap())
    }

    fn visit_grouping(&mut self, expression: &Expr) -> Output {
        format!("(group {})", self.visit_expression(expression))
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> Output {
        format!("({} {})", operator.lexeme, self.visit_expression(right))
    }

    fn visit_variable(&mut self, name: &Token, _depth: &Depth) -> Output {
        format!("(var {})", name.lexeme)
    }

    fn visit_assign(&mut self, name: &Token, value: &Expr, _depth: &Depth) -> Output {
        format!("(assign {} {})", name.lexeme, self.visit_expression(value))
    }

    fn visit_logical_or(&mut self, left: &Expr, right: &Expr) -> Output {
        format!("(or {} {})", self.visit_expression(left), self.visit_expression(right))
    }

    fn visit_logical_and(&mut self, left: &Expr, right: &Expr) -> Output {
        format!("(and {} {})", self.visit_expression(left), self.visit_expression(right))
    }

    fn visit_call(&mut self, callee: &Expr, _paren: &Token, arguments: &Vec<Expr>) -> Output {
        let mut result = format!("(call {}", self.visit_expression(callee));
        for argument in arguments {
            result.push_str(&format!(" {}", self.visit_expression(argument)));
        }
        result.push(')');
        result
    }

    fn visit_lambda(&mut self, params: &Vec<Token>, body: &Vec<Statement>) -> Output {
        let param_list: Vec<String> = params.iter().map(|p| p.lexeme.clone()).collect();
        let body_list: Vec<String> = body.iter().map(|s| self.visit_statement(s)).collect();
        let mut result = format!("(lambda ({})", param_list.join(" "));
        for statement in body_list {
            result.push_str(&format!(" {}", statement));
        }
        result.push(')');
        result
    }

    fn visit_get(&mut self, object: &Expr, name: &Token) -> Output {
        format!("(get {} {})", self.visit_expression(object), name.lexeme)
    }

    fn visit_expression_statement(&mut self, expression: &Expr) -> Output {
        format!("(expr {})", self.visit_expression(expression))
    }

    fn visit_print_statement(&mut self, expression: &Expr) -> Output {
        format!("(print {})", self.visit_expression(expression))
    }

    fn visit_var_statement(&mut self, name: &Token, initializer: &Option<Expr>) -> Output {
        if let Some(init) = initializer {
            format!("(var {} {})", name.lexeme, self.visit_expression(init))
        } else {
            format!("(var {} nil)", name.lexeme)
        }
    }

    fn visit_block_statement(&mut self, statements: &[Statement]) -> Output {
        let mut result = String::from("(block");
        for statement in statements {
            result.push_str(&format!(" {}", self.visit_statement(statement)));
        }
        result.push(')');
        result
    }

    fn visit_if_statement(&mut self, condition: &Expr, then_branch: &Statement, else_branch: &Option<Box<Statement>>) -> Output {
        let mut result = format!(
            "(if {} {}",
            self.visit_expression(condition),
            self.visit_statement(then_branch)
        );

        if let Some(else_stmt) = else_branch {
            result.push_str(&format!(" {}", self.visit_statement(else_stmt)));
        }

        result.push(')');
        result
    }

    fn visit_while_statement(&mut self, condition: &Expr, body: &Statement) -> Output {
        format!("(while {} {})", self.visit_expression(condition), self.visit_statement(body))
    }

    fn visit_function_statement(&mut self, statement: &Statement) -> Output {
        match statement {
            Statement::Function { name, params, body } => {
                let param_list: Vec<String> = params.iter().map(|p| p.lexeme.clone()).collect();
                let mut result = format!("(fun {} ({})", name.lexeme, param_list.join(" "));

                for stmt in body {
                    result.push_str(&format!(" {}", self.visit_statement(stmt)));
                }

                result.push(')');
                result
            }
            _ => String::from("(fun <invalid>)"),
        }
    }

    fn visit_return_statement(&mut self, _keyword: &Token, value: &Option<Expr>) -> Output {
        if let Some(return_value) = value {
            format!("(return {})", self.visit_expression(return_value))
        } else {
            String::from("(return)")
        }
    }

    fn visit_class_statement(&mut self, name: &Token, methods: &[Statement]) -> Output {
        let mut result = format!("(class {}", name.lexeme);
        for method in methods {
            result.push_str(&format!(" {}", self.visit_statement(method)));
        }
        result.push(')');
        result
    }
}