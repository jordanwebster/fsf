use crate::expression::Expression;

pub fn print(expression: Expression) -> String {
    match expression {
        Expression::Binary { left, operator, right } => format!("({} {} {})", operator.lexeme, print(*left), print(*right)),
        Expression::Grouping(expression) => format!("(group {})", print(*expression)),
        Expression::Unary { operator, right } => format!("({} {})", operator.lexeme, print(*right)),
        Expression::Literal(literal) => literal.to_string(),
    }
}