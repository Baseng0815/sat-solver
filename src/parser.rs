use std::{collections::HashMap, error::Error, fs, path::Path};

use chumsky::{container::Seq, error::Simple, pratt::{infix, prefix, right}, primitive::{choice, just}, recursive::recursive, text, Parser};

use crate::{expression::expression::{Expression, VariableId}, solver::instance::SATInstance};

// pub type ParseResult<T = ()> = Result<T, Simple<char>>;

// arbitrary expressions
#[derive(Debug, Clone)]
pub enum ParsedExpression {
    Variable(String),
    Constant(bool),
    And(Box<ParsedExpression>, Box<ParsedExpression>),
    Or(Box<ParsedExpression>, Box<ParsedExpression>),
    Not(Box<ParsedExpression>),
}

impl ParsedExpression {
    fn intern_to_expression(self, interned_variables: &mut HashMap<VariableId, String>) -> Expression {
        match self {
            ParsedExpression::Variable(var) => {
                let id = match interned_variables.iter().find(|(_, s)| **s == var) {
                    Some((id, _)) => *id,
                    None => {
                        let id = VariableId::try_from(interned_variables.len()).expect("Couldn't convert to variable id");
                        interned_variables.insert(id, var);
                        id
                    },
                };
                Expression::Variable(id)
            },
            ParsedExpression::Constant(val) => Expression::Constant(val),
            ParsedExpression::And(lhs, rhs) => {
                let expr_lhs = lhs.intern_to_expression(interned_variables);
                let expr_rhs = rhs.intern_to_expression(interned_variables);
                Expression::And(Box::new(expr_lhs), Box::new(expr_rhs))
            },
            ParsedExpression::Or(lhs, rhs) => {
                let expr_lhs = lhs.intern_to_expression(interned_variables);
                let expr_rhs = rhs.intern_to_expression(interned_variables);
                Expression::Or(Box::new(expr_lhs), Box::new(expr_rhs))
            },
            ParsedExpression::Not(expr) => {
                let interned = expr.intern_to_expression(interned_variables);
                Expression::Not(Box::new(interned))
            },
        }
    }
}

impl From<ParsedExpression> for SATInstance {
    fn from(value: ParsedExpression) -> Self {
        let mut interned_variables = HashMap::new();
        let interned_expression = value.intern_to_expression(&mut interned_variables);
        Self::new(interned_expression, interned_variables)
    }
}

fn parser<'a>() -> impl Parser<'a, &'a str, ParsedExpression> {
    recursive(|expr| {
        let variable = text::ascii::ident().map(|s: &str| ParsedExpression::Variable(s.to_string()));
        let constant = choice((
                just('0').to(ParsedExpression::Constant(false)),
                just('1').to(ParsedExpression::Constant(true)),
        ));

        let literal = choice((
                variable,
                constant,
        )).padded();

        let atom = literal.or(expr.delimited_by(just('('), just(')'))).padded();

        let op = |c| just(c).padded();
        atom.pratt((
                prefix(10, op('-'), |expr| ParsedExpression::Not(Box::new(expr))),
                infix(right(5), op('&'), |lhs, rhs| ParsedExpression::And(Box::new(lhs), Box::new(rhs))),
                infix(right(2), op('|'), |lhs, rhs| ParsedExpression::Or(Box::new(lhs), Box::new(rhs))),
        ))
    })
}

pub fn parse_file(file: &Path) -> SATInstance {
    let content = fs::read_to_string(file).unwrap();

    let parser = parser();
    let parsed_expression = parser.parse(&content).into_result().unwrap();

    let mut interned_variables = HashMap::new();
    let expression = parsed_expression.intern_to_expression(&mut interned_variables);
    SATInstance::new(expression, interned_variables)
}
