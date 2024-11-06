use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Expression {
    Variable(String), // we should intern strings and assign IDs instead but whatever
    Constant(bool),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
}

pub fn solve(expr: Expression) -> Option<HashMap<String, bool>> {

    None
}

pub fn evaluate(expression: &Expression, assignment: &HashMap<String, bool>) -> bool {
    match expression {
        Expression::Variable(var) => {
            *assignment.get(var).unwrap_or_else(|| panic!("Variable {} missing assignment", var))
        }
        Expression::Constant(value) => *value,
        Expression::And(lhs, rhs) => {
            let value_lhs = evaluate(lhs, assignment);
            let value_rhs = evaluate(rhs, assignment);
            value_lhs & value_rhs
        },
        Expression::Or(lhs, rhs) => {
            let value_lhs = evaluate(lhs, assignment);
            let value_rhs = evaluate(rhs, assignment);
            value_lhs | value_rhs
        },
        Expression::Not(expr) => {
            let value = evaluate(expr, assignment);
            !value
        }
    }
}
