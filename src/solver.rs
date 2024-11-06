use std::{collections::HashMap, fmt::Display};

use chumsky::container::Container;
use colored::{Color, Colorize};
use rand::seq::SliceRandom;

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
        let colors = [
            Color::Red,
            Color::Green,
            Color::Blue,
            Color::BrightRed,
            Color::BrightGreen,
            Color::BrightBlue,
            Color::Yellow,
            Color::BrightYellow,
            Color::Cyan,
            Color::BrightCyan
        ];

        match self {
            Expression::Variable(var) => write!(f, "{}", var),
            Expression::Constant(val) => write!(f, "{}", val),
            Expression::And(lhs, rhs) => {
                let color = *colors.choose(&mut rand::thread_rng()).unwrap();
                write!(f, "{}{} & {}{}", "(".color(color), lhs, rhs, ")".color(color))
            },
            Expression::Or(lhs, rhs) => {
                let color = *colors.choose(&mut rand::thread_rng()).unwrap();
                write!(f, "{}{} | {}{}", "(".color(color), lhs, rhs, ")".color(color))
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

    fn to_dnf_recursive(self, modified_prev: &mut bool) -> Expression {
        // we apply the following rules until a fix point is found:
        // 1. combine multiple not: --x => x
        // 2.1 DeMorgan: -(x | y) => -x & -x
        // 2.2 DeMorgan: -(x & y) => -x | -x
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
                            Box::new(Expression::And(Box::new(lhs.clone().to_dnf_recursive(&mut modified)), Box::new(inner_lhs.to_dnf_recursive(modified_prev)))),
                            Box::new(Expression::And(Box::new(lhs.to_dnf_recursive(&mut modified)), Box::new(inner_rhs.to_dnf_recursive(modified_prev)))),
                        )
                    } else if let Expression::Or(inner_lhs, inner_rhs) = *lhs {
                        // ((inner_lhs | inner_rhs) & rhs) => (inner_lhs & rhs) | (inner_rhs & rhs)
                        modified = true;
                        Expression::Or(
                            Box::new(Expression::And(Box::new(inner_lhs.to_dnf_recursive(&mut modified)), Box::new(rhs.clone().to_dnf_recursive(modified_prev)))),
                            Box::new(Expression::And(Box::new(inner_rhs.to_dnf_recursive(&mut modified)), Box::new(rhs.to_dnf_recursive(modified_prev)))),
                        )
                    } else {
                        Expression::And(Box::new(lhs.to_dnf_recursive(&mut modified)), Box::new(rhs.to_dnf_recursive(modified_prev)))
                    }
                },
                Expression::Or(lhs, rhs) => Expression::Or(Box::new(lhs.to_dnf_recursive(&mut modified)), Box::new(rhs.to_dnf_recursive(modified_prev))),
                Expression::Not(expr) => match *expr {
                    Expression::And(lhs, rhs) => {
                        // -(lhs & rhs) => -lhs | -rhs
                        let inner_lhs = Expression::Not(Box::new(lhs.to_dnf_recursive(&mut modified)));
                        let inner_rhs = Expression::Not(Box::new(rhs.to_dnf_recursive(&mut modified)));
                        modified = true;
                        Expression::Or(Box::new(inner_lhs), Box::new(inner_rhs))
                    }
                    Expression::Or(lhs, rhs) => {
                        // -(lhs | rhs) => -lhs & -rhs
                        let inner_lhs = Expression::Not(Box::new(lhs.to_dnf_recursive(&mut modified)));
                        let inner_rhs = Expression::Not(Box::new(rhs.to_dnf_recursive(&mut modified)));
                        modified = true;
                        Expression::And(Box::new(inner_lhs), Box::new(inner_rhs))
                    }
                    Expression::Not(inner_expr) => inner_expr.to_dnf_recursive(&mut modified),
                    _ => Expression::Not(expr),
                },
                _ => expression,
            };

            if modified {
                *modified_prev = true;
            }
        }

        expression
    }

    pub fn to_dnf(self) -> Expression {
        let mut modified = false;
        self.to_dnf_recursive(&mut modified)
    }

    fn recursive_demorgan(self) -> Expression {
        let mut expression = self;
        let mut modified = true;

        while modified {
            modified = false;

            expression = match expression {
                Expression::And(lhs, rhs) => {
                    let inner_lhs = Box::new(lhs.recursive_demorgan());
                    let inner_rhs = Box::new(rhs.recursive_demorgan());
                    Expression::And(inner_lhs, inner_rhs)
                }
                Expression::Or(lhs, rhs) => {
                    let inner_lhs = Box::new(lhs.recursive_demorgan());
                    let inner_rhs = Box::new(rhs.recursive_demorgan());
                    Expression::Or(inner_lhs, inner_rhs)
                }
                Expression::Not(expr) => match *expr {
                    Expression::And(lhs, rhs) => {
                        // -(lhs & rhs) => -lhs | -rhs
                        let inner_lhs = Expression::Not(Box::new(lhs.recursive_demorgan()));
                        let inner_rhs = Expression::Not(Box::new(rhs.recursive_demorgan()));
                        modified = true;
                        Expression::Or(Box::new(inner_lhs), Box::new(inner_rhs))
                    }
                    Expression::Or(lhs, rhs) => {
                        // -(lhs | rhs) => -lhs & -rhs
                        let inner_lhs = Expression::Not(Box::new(lhs.recursive_demorgan()));
                        let inner_rhs = Expression::Not(Box::new(rhs.recursive_demorgan()));
                        modified = true;
                        Expression::And(Box::new(inner_lhs), Box::new(inner_rhs))
                    }
                    Expression::Not(inner_expr) => inner_expr.recursive_demorgan(),
                    _ => Expression::Not(expr)
                },
                _ => expression,
            }
        }

        expression
    }

    pub fn to_cnf(self) -> Expression {
        // 1. negate and convert to dnf
        let negated_dnf = Expression::Not(Box::new(self)).to_dnf();
        eprintln!("negated_dnf = {}", negated_dnf);

        // 2. negate again and move not inwards using DeMorgan
        Expression::Not(Box::new(negated_dnf)).recursive_demorgan()
    }

    pub fn extract_cnf_clauses(self) -> Vec<Expression> {
        let mut clauses = Vec::new();
        let mut remaining = vec![self];
        while !remaining.is_empty() {
            let top = remaining.pop().unwrap();
            if let Expression::And(lhs, rhs) = top {
                remaining.push(*lhs);
                remaining.push(*rhs);
            } else {
                clauses.push(top);
            }
        }

        clauses
    }
}

#[derive(Debug, Clone)]
pub struct SATInstance {
    pub expression: Expression,
    pub variables: Vec<String>,
}
