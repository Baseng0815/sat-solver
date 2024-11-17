// This file contains data structures and functions for transforming expressions into normal forms.

use std::collections::HashSet;

use super::expression::{Assignment, Expression, VariableId};

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct Clause {
    pub literals: Vec<Literal>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Literal {
    pub var_id: VariableId,
    pub value: bool,
}

#[derive(Debug)]
pub struct DNF {
    pub clauses: Vec<Clause>
}

#[derive(Debug, Default, Clone)]
pub struct CNF {
    pub clauses: Vec<Clause>
}

impl Literal {
    pub fn new(var_id: VariableId, value: bool) -> Self {
        Self { var_id, value }
    }

    pub fn not(&self) -> Self {
        Self::new(self.var_id, !self.value)
    }
}

impl Clause {
    pub fn new(literals: Vec<Literal>) -> Self {
        Self { literals }
    }


}

impl From<Literal> for Expression {
    fn from(value: Literal) -> Self {
        if value.value == true {
            Expression::Variable(value.var_id)
        } else {
            Expression::Not(Box::new(Expression::Variable(value.var_id)))
        }
    }
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

impl From<CNF> for Expression {
    fn from(value: CNF) -> Self {
        let clause_expressions = value.clauses.into_iter().map(|clause| {
            clause.literals.into_iter().fold(Expression::Constant(false), |acc, literal| {
                Expression::Or(Box::new(acc), Box::new(literal.into()))
            })
        });

        clause_expressions.into_iter().fold(Expression::Constant(true), |acc, clause| {
                Expression::And(Box::new(acc), Box::new(clause.into()))
                })
    }
}

impl From<Expression> for DNF {
    fn from(value: Expression) -> Self {
        // convert to dnf
        let dnf_expr = value.to_dnf_expr();

        // extract clauses
        let mut clauses = HashSet::new();
        let mut remaining = vec![dnf_expr];
        while !remaining.is_empty() {
            let top = remaining.pop().unwrap();
            if let Expression::Or(lhs, rhs) = top {
                remaining.push(*lhs);
                remaining.push(*rhs);
            } else {
                let literals = top.collect_literals().into_iter().collect::<Vec<_>>();
                clauses.insert(Clause::new(literals));
            }
        }

        Self::new(clauses.into_iter().collect::<Vec<_>>())
    }
}

impl From<Expression> for CNF {
    fn from(value: Expression) -> Self {
        let cnf_expr = value.to_cnf_expr();
        // eprintln!("CNF expression = {}", cnf_expr);

        // extract clauses
        let mut clauses = Vec::new();
        let mut remaining = vec![cnf_expr];
        while !remaining.is_empty() {
            let top = remaining.pop().unwrap();
            if let Expression::And(lhs, rhs) = top {
                remaining.push(*lhs);
                remaining.push(*rhs);
            } else {
                let literals = top.collect_literals().into_iter().collect::<Vec<_>>();
                clauses.push(Clause::new(literals));
            }
        }

        Self::new(clauses)
    }
}

impl Expression {
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

        // 2. negate again and move 'not' inwards using DeMorgan
        Expression::Not(Box::new(negated_dnf)).recursive_demorgan()
    }

    /// Collect all literals of `self`
    ///
    /// # Example
    ///
    /// collect_literals(-(a | b | b) & c & -d) -> {a, b, c, -d}
    fn collect_literals(&self) -> HashSet<Literal> {
        match self {
            Expression::Variable(var) => HashSet::from([Literal::new(*var, true)]),
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
                    Expression::Variable(var) => HashSet::from([Literal::new(*var, false)]),
                    _ => expr.collect_literals()
                }
            },
        }
    }
}
