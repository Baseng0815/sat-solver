use std::{collections::{BTreeSet, HashMap, HashSet}, fmt::{Debug, Display}};

use chumsky::container::Container;
use colored::{Color, Colorize};
use rand::seq::SliceRandom;

pub type VariableId = u16;

// cnf, dnf
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Literal {
    Variable(VariableId),
    VariableNot(VariableId),
}

impl Literal {
    pub fn new(id: VariableId, value: bool) -> Self {
        if value == true {
            Literal::Variable(id)
        } else {
            Literal::VariableNot(id)
        }
    }

    pub fn id(&self) -> VariableId {
        match self {
            Literal::Variable(id) => *id,
            Literal::VariableNot(id) => *id,
        }
    }

    pub fn not(&self) -> Self {
        match self {
            Literal::Variable(id) => Literal::VariableNot(*id),
            Literal::VariableNot(id) => Literal::Variable(*id),
        }
    }

    pub fn value(&self) -> bool {
        match self {
            Literal::Variable(_) => true,
            Literal::VariableNot(_) => false,
        }
    }
}

// arbitrary expressions
#[derive(Debug, Clone)]
pub enum Expression {
    Variable(VariableId),
    Constant(bool),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
}

#[derive(Debug, Clone)]
pub struct SATInstance {
    pub expression: Expression,
    pub interned_variables: HashMap<VariableId, String>,
}

impl SATInstance {
    pub fn new(expression: Expression, interned_variables: HashMap<VariableId, String>) -> Self {
        Self { expression, interned_variables }
    }
}

#[derive(Debug, Default)]
pub struct Clause {
    pub literals: HashSet<Literal>,
}

impl Clause {
    pub fn new(literals: HashSet<Literal>) -> Self {
        Self { literals }
    }


}

#[derive(Debug, Default)]
pub struct Assignment {
    pub values: HashMap<VariableId, bool>
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

#[derive(Debug)]
pub struct DNF {
    pub clauses: Vec<Clause>
}

#[derive(Debug, Default)]
pub struct CNF {
    pub clauses: Vec<Clause>
}

impl DNF {
    pub fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }
}

impl CNF {
    pub fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }
}

impl From<Expression> for DNF {
    fn from(value: Expression) -> Self {
        // convert to dnf
        let dnf_expr = value.to_dnf_expr();

        // extract clauses
        let mut clauses = Vec::new();
        let mut remaining = vec![dnf_expr];
        while !remaining.is_empty() {
            let top = remaining.pop().unwrap();
            if let Expression::Or(lhs, rhs) = top {
                remaining.push(*lhs);
                remaining.push(*rhs);
            } else {
                clauses.push(Clause::new(top.collect_literals()));
            }
        }

        Self::new(clauses)
    }
}

impl From<Expression> for CNF {
    fn from(value: Expression) -> Self {
        let cnf_expr = value.to_cnf_expr();
        eprintln!("CNF expression = {}", cnf_expr);

        // extract clauses
        let mut clauses = Vec::new();
        let mut remaining = vec![cnf_expr];
        while !remaining.is_empty() {
            let top = remaining.pop().unwrap();
            if let Expression::And(lhs, rhs) = top {
                remaining.push(*lhs);
                remaining.push(*rhs);
            } else {
                clauses.push(Clause::new(top.collect_literals()));
            }
        }

        Self::new(clauses)
    }
}

impl Display for SATInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Instance containing {} variables", self.interned_variables.len());

        write!(f, "Expression: {}", self.expression)
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

    /// Distribute 'And' expressions over 'Or' expressions
    ///
    /// # Example
    ///
    /// `(v0 | v1) & v2 => (v0 & v2) | (v1 & v2)`
    fn distribute_and_over_or(self) -> Expression {
        match self {
            Expression::And(lhs, rhs) => {
                if let Expression::Or(inner_lhs, inner_rhs) = *rhs {
                    // (lhs & (inner_lhs | inner_rhs) => (lhs & inner_lhs) | (lhs & inner_rhs))
                    Expression::Or(
                        Box::new(Expression::And(lhs.clone(), inner_lhs.clone())),
                        Box::new(Expression::And(lhs.clone(), inner_rhs.clone())),
                    ).distribute_and_over_or()
                } else if let Expression::Or(inner_lhs, inner_rhs) = *lhs {
                    // ((inner_lhs | inner_rhs) & rhs) => (inner_lhs & rhs) | (inner_rhs & rhs)
                    Expression::Or(
                        Box::new(Expression::And(inner_lhs.clone(), rhs.clone())),
                        Box::new(Expression::And(inner_rhs.clone(), rhs.clone())),
                    ).distribute_and_over_or()
                } else {
                    Expression::And(Box::new(lhs.distribute_and_over_or()), Box::new(rhs.distribute_and_over_or()))
                }
            },
            Expression::Or(lhs, rhs) => Expression::Or(Box::new(lhs.distribute_and_over_or()), Box::new(rhs.distribute_and_over_or())),
            Expression::Not(expr) => Expression::Not(Box::new(expr.distribute_and_over_or())),
            _ => self,
        }
    }

    /// Convert `self` into an expression in disjunctive normal form, i.e. a disjunction of
    /// conjunctions.
    ///
    /// # Example
    ///
    /// (v0 | v1) & v2 => (v0 & v2) | (v1 | v2)
    fn to_dnf_expr(self) -> Expression {
        let reduced = self.evaluate(&Assignment::default());
        let nnf = reduced.recursive_demorgan();
        let distributed = nnf.distribute_and_over_or();
        distributed
    }

    /// Move 'Not' expressions inside
    ///
    /// # Example
    ///
    /// `-((v0 | v1) & -v2) => (-v0 & -v1) | v2`
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

    /// Convert `self` into an expression in conjunctive normal form, i.e. a conjunction of
    /// disjunctions.
    ///
    /// # Example
    ///
    /// (v0 & v1) | v2 => (v0 | v2) & (v1 | v2)
    fn to_cnf_expr(self) -> Expression {
        // 1. negate and convert to dnf
        let negated_dnf = Expression::Not(Box::new(self)).to_dnf_expr();
        eprintln!("-DNF expression = {}", negated_dnf);

        // 2. negate again and move 'not' inwards using DeMorgan
        Expression::Not(Box::new(negated_dnf)).recursive_demorgan()
    }

    fn collect_literals(&self) -> HashSet<Literal> {
        match self {
            Expression::Variable(var) => HashSet::from([Literal::Variable(var.clone())]),
            Expression::Constant(_) => HashSet::from([]),
            Expression::And(lhs, rhs) => {
                let mut literals_lhs = lhs.collect_literals();
                literals_lhs.extend(rhs.collect_literals());
                literals_lhs
            },
            Expression::Or(lhs, rhs) => {
                let mut literals_lhs = lhs.collect_literals();
                literals_lhs.extend(rhs.collect_literals());
                literals_lhs
            },
            Expression::Not(expr) => {
                match expr.as_ref() {
                    Expression::Variable(var) => HashSet::from([Literal::VariableNot(var.clone())]),
                    _ => expr.collect_literals()
                }
            },
        }
    }
}
