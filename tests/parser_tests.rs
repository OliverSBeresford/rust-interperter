use rust_interpreter::{Parser, scan, Expr, TokenType, AstPrinter};

#[test]
fn parse_simple_addition_expression() {
    let input = "1 + 2;";
    let tokens = scan(input);
    let mut parser = Parser::new(tokens.tokens);
    let expr = parser.expression().unwrap_or_else(|e| panic!("parse error: {}", e));
    match expr {
        Expr::Binary { operator, .. } => {
            assert!(matches!(operator.token_type, TokenType::Plus));
        }
        _ => panic!("expected binary expression"),
    }
}

#[test]
fn parse_error_on_invalid_expression() {
    let input = "1 + ;";
    let tokens = scan(input);
    let mut parser = Parser::new(tokens.tokens);

    // This should result in a parse error (Result::Err)
    let result = parser.expression();
    assert!(result.is_err());
}

#[test]
fn parse_math_expression() {
    let input = "1 + 2 * 4 - 8 + 9 / 2.99 + (3 - (4 / 2));";
    let tokens = scan(input);
    let mut parser = Parser::new(tokens.tokens);
    let expr = parser.expression().unwrap_or_else(|e| panic!("parse error: {}", e));
    
    // Use AstPrinter to get the string representation of the AST
    assert!(matches!(AstPrinter::new().print_to_string(&expr).as_str(), "(+ (+ (- (+ 1.0 (* 2.0 4.0)) 8.0) (/ 9.0 2.99)) (group (- 3.0 (group (/ 4.0 2.0)))))"));
}
