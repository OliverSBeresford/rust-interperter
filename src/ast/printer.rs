use crate::{Expr};
use crate::Token;

type Output = String;

// Pretty-printer
pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: &Expr) {
        println!("{}", self.visit(expr));
    }

    pub fn print_to_string(&self, expr: &Expr) -> String {
        self.visit(expr)
    }

    pub fn visit(&self, expr: &Expr) -> Output {
        match expr {
            Expr::Binary { left, operator, right } => self.visit_binary(left, operator, right),
            Expr::Literal { value } => self.visit_literal(value),
            Expr::Grouping { expression } => self.visit_grouping(expression),
            Expr::Unary { operator, right } => self.visit_unary(operator, right),
            Expr::Variable { name, .. } => self.visit_variable(name),
            Expr::Assign { name, value, .. } => self.visit_assign(name, value),
            Expr::LogicOr { left, right } => self.visit_logic_or(left, right),
            Expr::LogicAnd { left, right } => self.visit_logic_and(left, right),
            Expr::Call { callee, arguments , ..} => self.visit_call(callee, arguments),
            Expr::Lambda { params, .. } => self.visit_lambda(params),
            Expr::Get { object, name } => self.visit_get(object, name),

        }
    }

    fn visit_binary(&self, left: &Expr, operator: &Token, right: &Expr) -> Output {
        format!("({} {} {})", operator.lexeme, self.visit(left), self.visit(right))
    }

    fn visit_literal(&self, value: &Token) -> Output {
        format!("{}", value.literal.as_ref().unwrap())
    }

    fn visit_grouping(&self, expression: &Expr) -> Output {
        format!("(group {})", self.visit(expression))
    }

    fn visit_unary(&self, operator: &Token, right: &Expr) -> Output {
        format!("({} {})", operator.lexeme, self.visit(right))
    }

    fn visit_variable(&self, name: &Token) -> Output {
        format!("(var {})", name.lexeme)
    }

    fn visit_assign(&self, name: &Token, value: &Expr) -> Output {
        format!("(assign {} {})", name.lexeme, self.visit(value))
    }

    fn visit_logic_or(&self, left: &Expr, right: &Expr) -> Output {
        format!("(or {} {})", self.visit(left), self.visit(right))
    }

    fn visit_logic_and(&self, left: &Expr, right: &Expr) -> Output {
        format!("(and {} {})", self.visit(left), self.visit(right))
    }

    fn visit_call(&self, callee: &Expr, arguments: &Vec<Expr>) -> Output {
        let mut result = format!("(call {}", self.visit(callee));
        for argument in arguments {
            result.push_str(&format!(" {}", self.visit(argument)));
        }
        result.push(')');
        result
    }

    fn visit_lambda(&self, params: &Vec<Token>) -> Output {
        let param_list: Vec<String> = params.iter().map(|p| p.lexeme.clone()).collect();
        let mut result = format!("(lambda with ({})", param_list.join(" "));
        result.push(')');
        result
    }

    fn visit_get(&self, object: &Expr, name: &Token) -> Output {
        format!("(get {} {})", self.visit(object), name.lexeme)
    }
}