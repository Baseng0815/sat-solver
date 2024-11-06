use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub enum Expression {
    Variable(String), // we should intern strings and assign IDs instead but whatever
    Constant(bool),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Variable(var) => write!(f, "{}", var),
            Expression::Constant(val) => write!(f, "{}", val),
            Expression::And(lhs, rhs) => {
                write!(f, "({} & {})", lhs, rhs)
            },
            Expression::Or(lhs, rhs) => {
                write!(f, "({} | {})", lhs, rhs)
            },
            Expression::Not(expr) => {
                write!(f, "-{}", expr)
            }
        }
    }
}

impl Expression {
    pub fn solve(&self) -> Option<HashMap<String, bool>> {
        None
    }

    pub fn evaluate(&self, assignment: &HashMap<String, bool>) -> bool {
        match self {
            Expression::Variable(var) => *assignment
                .get(var)
                .unwrap_or_else(|| panic!("Variable {} missing assignment", var)),
            Expression::Constant(value) => *value,
            Expression::And(lhs, rhs) => {
                let value_lhs = lhs.evaluate(assignment);
                let value_rhs = rhs.evaluate(assignment);
                value_lhs & value_rhs
            }
            Expression::Or(lhs, rhs) => {
                let value_lhs = rhs.evaluate(assignment);
                let value_rhs = lhs.evaluate(assignment);
                value_lhs | value_rhs
            }
            Expression::Not(expr) => {
                let value = expr.evaluate(assignment);
                !value
            }
        }
    }

    pub fn to_dnf(self) -> Expression {
        // we apply the following rules until a fix point is found:
        // 1. combine multiple not: --x => x
        // 2.1 De Morgan: -(x | y) => -x & -x
        // 2.2 De Morgan: -(x & y) => -x | -x
        // 3.1 distribute: x & (y | z) => x & y | x & z
        // 3.2 distribute: (x | y) & z => x & z | y & z

        let mut expression = self;
        let mut modified = true;

        while modified {
            modified = false;

            expression = match expression {
                Expression::And(lhs, rhs) => {
                    if let Expression::Or(inner_lhs, inner_rhs) = *rhs {
                        // (lhs & (inner_lhs | inner_rhs) => (lhs & inner_lhs) | (lhs & inner_rhs))
                        modified = true;
                        Expression::Or(
                            Box::new(Expression::And(Box::new(lhs.clone().to_dnf()), Box::new(inner_lhs.to_dnf()))),
                            Box::new(Expression::And(Box::new(lhs.to_dnf()), Box::new(inner_rhs.to_dnf()))),
                        )
                    } else if let Expression::Or(inner_lhs, inner_rhs) = *lhs {
                        // ((inner_lhs | inner_rhs) & rhs) => (inner_lhs & rhs) | (inner_rhs & rhs)
                        modified = true;
                        Expression::Or(
                            Box::new(Expression::And(Box::new(inner_lhs.to_dnf()), Box::new(rhs.clone().to_dnf()))),
                            Box::new(Expression::And(Box::new(inner_rhs.to_dnf()), Box::new(rhs.to_dnf()))),
                        )
                    } else {
                        Expression::And(Box::new(lhs.to_dnf()), Box::new(rhs.to_dnf()))
                    }
                },
                Expression::Or(lhs, rhs) => Expression::Or(Box::new(lhs.to_dnf()), Box::new(rhs.to_dnf())),
                Expression::Not(expr) => match *expr {
                    Expression::And(lhs, rhs) => {
                        // -(lhs & rhs) => -lhs | -rhs
                        let inner_lhs = Expression::Not(Box::new(lhs.to_dnf()));
                        let inner_rhs = Expression::Not(Box::new(rhs.to_dnf()));
                        modified = true;
                        Expression::Or(Box::new(inner_lhs), Box::new(inner_rhs))
                    }
                    Expression::Or(lhs, rhs) => {
                        // -(lhs | rhs) => -lhs & -rhs
                        let inner_lhs = Expression::Not(Box::new(lhs.to_dnf()));
                        let inner_rhs = Expression::Not(Box::new(rhs.to_dnf()));
                        modified = true;
                        Expression::And(Box::new(inner_lhs), Box::new(inner_rhs))
                    }
                    Expression::Not(inner_expr) => *inner_expr,
                    _ => Expression::Not(expr),
                },
                _ => expression,
            };
        }

        expression
    }
}

#[derive(Debug, Clone)]
pub struct SATInstance {
    pub expression: Expression,
    pub variables: Vec<String>,
}
