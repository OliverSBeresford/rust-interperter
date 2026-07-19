use std::rc::Rc;

use crate::{
    ast::{Visitor, Expr, Statement},
    lexer::Token,
};

type Output = String;

// Pretty-printer with depth-aware colored parentheses
pub struct AstPrinter {
    depth: usize,
    use_colors: bool,
}

impl AstPrinter {
    pub fn new() -> Self {
        AstPrinter { depth: 0, use_colors: false }
    }

    pub fn new_colored() -> Self {
        AstPrinter { depth: 0, use_colors: true }
    }

    pub fn print_expression(&mut self, expr: &Expr) {
        println!("{}", self.visit_expression(expr));
    }

    pub fn print_statement(&mut self, statement: Rc<Statement>) {
        println!("{}", self.visit_statement(statement));
    }

    pub fn print_to_string(&mut self, expr: &Expr) -> String {
        self.visit_expression(expr)
    }

    pub fn print_statements(&mut self, statements: Vec<Rc<Statement>>) {
        for statement in statements {
            self.print_statement(statement.clone());
        }
    }

    // Return the ANSI color code prefix for a given depth (cycles a small palette)
    fn color_code(&self, depth: usize) -> &'static str {
        if !self.use_colors {
            return "";
        }
        match depth % 6 {
            0 => "\x1b[31m", // red
            1 => "\x1b[32m", // green
            2 => "\x1b[33m", // yellow
            3 => "\x1b[34m", // blue
            4 => "\x1b[35m", // magenta
            _ => "\x1b[36m", // cyan
        }
    }

    fn colored_open(&self, depth: usize) -> String {
        if !self.use_colors {
            return "(".to_string();
        }
        format!("{}(\x1b[0m", self.color_code(depth))
    }

    fn colored_close(&self, depth: usize) -> String {
        if !self.use_colors {
            return ")".to_string();
        }
        format!("{})\x1b[0m", self.color_code(depth))
    }

    fn get_open_close(&self) -> (String, String) {
        (self.colored_open(self.depth), self.colored_close(self.depth))
    }
}

impl Visitor<Output> for AstPrinter {
    fn visit_binary(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let left_s = self.visit_expression(left);
        let right_s = self.visit_expression(right);
        self.depth -= 1;
        format!("{}{} {} {}{}", open, operator.lexeme, left_s, right_s, close)
    }

    fn visit_literal(&mut self, value: &Token) -> Output {
        format!("{}", value.literal.as_ref().unwrap())
    }

    fn visit_grouping(&mut self, expression: &Expr) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let inner = format!("group {}", self.visit_expression(expression));
        self.depth -= 1;
        format!("{}{}{}", open, inner, close)
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expr) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let inner = format!("{} {}", operator.lexeme, self.visit_expression(right));
        self.depth -= 1;
        format!("{}{}{}", open, inner, close)
    }

    fn visit_variable(&mut self, name: &Token) -> Output {
        let (open, close) = self.get_open_close();
        format!("{}var {}{}", open, name.lexeme, close)
    }

    fn visit_assign(&mut self, name: &Token, value: &Expr) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let val = self.visit_expression(value);
        self.depth -= 1;
        format!("{}assign {} {}{}", open, name.lexeme, val, close)
    }

    fn visit_logical_or(&mut self, left: &Expr, right: &Expr) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let l = self.visit_expression(left);
        let r = self.visit_expression(right);
        self.depth -= 1;
        format!("{}or {} {}{}", open, l, r, close)
    }

    fn visit_logical_and(&mut self, left: &Expr, right: &Expr) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let l = self.visit_expression(left);
        let r = self.visit_expression(right);
        self.depth -= 1;
        format!("{}and {} {}{}", open, l, r, close)
    }

    fn visit_call(&mut self, callee: &Expr, _paren: &Token, arguments: &Vec<Expr>) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let mut parts = vec![self.visit_expression(callee)];
        for argument in arguments {
            parts.push(self.visit_expression(argument));
        }
        self.depth -= 1;
        format!("{}call {}{}", open, parts.join(" "), close)
    }

    fn visit_lambda(&mut self, params: &Vec<Token>, body: Vec<Rc<Statement>>) -> Output {
        let (open, close) = self.get_open_close();
        let param_list: Vec<String> = params.iter().map(|p| p.lexeme.clone()).collect();
        self.depth += 1;
        let body_list: Vec<String> = body.iter().map(|s| self.visit_statement(s.clone())).collect();
        self.depth -= 1;
        let mut inner = format!("lambda ({})", param_list.join(" "));
        for statement in body_list {
            inner.push_str(&format!(" {}", statement));
        }
        format!("{}{}{}", open, inner, close)
    }

    fn visit_get(&mut self, object: &Expr, name: &Token) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let obj = self.visit_expression(object);
        self.depth -= 1;
        format!("{}get {} .{}{}", open, obj, name.lexeme, close)
    }

    fn visit_set(&mut self, object: &Expr, name: &Token, value: &Expr) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let obj = self.visit_expression(object);
        let val = self.visit_expression(value);
        self.depth -= 1;
        format!("{}set {} .{} {}{}", open, obj, name.lexeme, val, close)
    }

    fn visit_this(&mut self, keyword: &Token) -> Output {
        let (open, close) = self.get_open_close();
        format!("{}this {}{}", open, keyword.lexeme, close)
    }

    fn visit_this_class(&mut self, keyword: &Token) -> Output {
        let (open, close) = self.get_open_close();
        format!("{}This {}{}", open, keyword.lexeme, close)
    }

    fn visit_super(&mut self, keyword: &Token, property: &Token) -> Output {
        let (open, close) = self.get_open_close();
        format!("{}super {} .{}{}", open, keyword.lexeme, property.lexeme, close)
    }

    fn visit_expression_statement(&mut self, expression: &Expr) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let inner = format!("expr {}", self.visit_expression(expression));
        self.depth -= 1;
        format!("{}{}{}", open, inner, close)
    }

    fn visit_print_statement(&mut self, expression: &Expr) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let inner = format!("print {}", self.visit_expression(expression));
        self.depth -= 1;
        format!("{}{}{}", open, inner, close)
    }

    fn visit_var_statement(&mut self, name: &Token, initializer: &Option<Expr>) -> Output {
        let (open, close) = self.get_open_close();
        if let Some(init) = initializer {
            self.depth += 1;
            let val = self.visit_expression(init);
            self.depth -= 1;
            format!("{}var {} = {}{}", open, name.lexeme, val, close)
        } else {
            format!("{}var {} = nil{}", open, name.lexeme, close)
        }
    }

    fn visit_block_statement(&mut self, statements: Vec<Rc<Statement>>) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let mut inner = String::from("block");
        for statement in statements {
            inner.push_str(&format!(" {}", self.visit_statement(statement)));
        }
        self.depth -= 1;
        format!("{}{}{}", open, inner, close)
    }

    fn visit_if_statement(&mut self, condition: &Expr, then_branch: Rc<Statement>, else_branch: Option<Rc<Statement>>) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let cond = self.visit_expression(condition);
        let then_s = self.visit_statement(then_branch);
        let mut inner = format!("if {} {}", cond, then_s);
        if let Some(else_stmt) = else_branch {
            inner.push_str(&format!(" {}", self.visit_statement(else_stmt)));
        }
        self.depth -= 1;
        format!("{}{}{}", open, inner, close)
    }

    fn visit_while_statement(&mut self, condition: &Expr, body: Rc<Statement>) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        let inner = format!("while {} {}", self.visit_expression(condition), self.visit_statement(body));
        self.depth -= 1;
        format!("{}{}{}", open, inner, close)
    }

    fn visit_function_statement(&mut self, statement: Rc<Statement>) -> Output {
        match &*statement {
            Statement::Function { name, params, body, .. } => {
                let param_list: Vec<String> = params.iter().map(|p| p.lexeme.clone()).collect();
                let (open, close) = self.get_open_close();
                self.depth += 1;
                let mut inner = format!("fun {} ({})", name.lexeme, param_list.join(" "));
                for stmt in body {
                    inner.push_str(&format!(" {}", self.visit_statement(stmt.clone())));
                }
                self.depth -= 1;
                format!("{}{}{}", open, inner, close)
            }
            _ => String::from("(fun <invalid>)"),
        }
    }

    fn visit_return_statement(&mut self, _keyword: &Token, value: &Option<Expr>) -> Output {
        let (open, close) = self.get_open_close();
        if let Some(return_value) = value {
            self.depth += 1;
            let inner = format!("return {}", self.visit_expression(return_value));
            self.depth -= 1;
            format!("{}{}{}", open, inner, close)
        } else {
            format!("{}return{}", open, close)
        }
    }

    fn visit_class_statement(&mut self, name: &Token, superclass: &Option<Expr>, methods: Vec<Rc<Statement>>, static_fields: Vec<Rc<Statement>>, static_methods: Vec<Rc<Statement>>) -> Output {
        let (open, close) = self.get_open_close();
        self.depth += 1;
        // If there is a superclass, visit it and include it in the output
        let superclass_str = if let Some(superclass_expr) = superclass {
            format!("< {}", self.visit_expression(superclass_expr))
        } else {
            String::new()
        };
        let mut inner = format!("class {} {}", name.lexeme, superclass_str);

        // Iterate over methods, static fields, and static methods and append their string representations
        for method in methods {
            inner.push_str(&format!(" {}", self.visit_statement(method)));
        }
        for static_field in static_fields {
            inner.push_str(&format!(" {}", self.visit_statement(static_field)));
        }
        for static_method in static_methods {
            inner.push_str(&format!(" {}", self.visit_statement(static_method)));
        }
        self.depth -= 1;
        
        format!("{}{}{}", open, inner, close)
    }
}