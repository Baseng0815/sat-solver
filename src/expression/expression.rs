// This file contains data structures and functions for expressions, assignments and evaluation.

use std::{collections::HashMap, fmt::Display};
use rand::seq::SliceRandom;

use colored::{Color, Colorize};

pub type VariableId = u16;

// arbitrary expressions
#[derive(Debug, Clone)]
pub enum Expression {
    Variable(VariableId),
    Constant(bool),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
}

#[derive(Debug, Default, Clone)]
pub struct Assignment {
    pub values: HashMap<VariableId, bool>
}

impl Expression {
    /// (Partially) evaluate `self` using the given [Assignment].
    pub fn evaluate(self, assignment: &Assignment) -> Expression {
        match self {
            Expression::Variable(var) => {
                if let Some(val) = assignment.values.get(&var) {
                    Expression::Constant(*val)
                } else {
                    Expression::Variable(var)
                }
            }
            Expression::And(lhs, rhs) => {
                let value_lhs = lhs.evaluate(assignment);
                let value_rhs = rhs.evaluate(assignment);
                if let Expression::Constant(val) = value_lhs {
                    if val {
                        value_rhs
                    } else {
                        Expression::Constant(false)
                    }
                } else if let Expression::Constant(val) = value_rhs {
                    if val {
                        value_lhs
                    } else {
                        Expression::Constant(false)
                    }
                } else {
                    Expression::And(Box::new(value_lhs), Box::new(value_rhs))
                }
            }
            Expression::Or(lhs, rhs) => {
                let value_lhs = lhs.evaluate(assignment);
                let value_rhs = rhs.evaluate(assignment);
                if let Expression::Constant(val) = value_lhs {
                    if val {
                        Expression::Constant(true)
                    } else {
                        value_rhs
                    }
                } else if let Expression::Constant(val) = value_rhs {
                    if val {
                        Expression::Constant(true)
                    } else {
                        value_lhs
                    }
                } else {
                    Expression::Or(Box::new(value_lhs), Box::new(value_rhs))
                }
            }
            Expression::Not(expr) => {
                let value = expr.evaluate(assignment);
                if let Expression::Constant(val) = value {
                    Expression::Constant(!val)
                } else {
                    Expression::Not(Box::new(value))
                }
            }
            _ => self
        }
    }

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
            Expression::Variable(var) => write!(f, "v{}", var),
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

impl Assignment {
    pub fn new(values: HashMap<VariableId, bool>) -> Self {
        Self { values }
    }
}

impl<const N: usize> From<[(VariableId, bool); N]> for Assignment {
    fn from(value: [(VariableId, bool); N]) -> Self {
        Self::new(HashMap::from(value))
    }
}
