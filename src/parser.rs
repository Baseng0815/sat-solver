use std::{error::Error, fs, path::Path};

use chumsky::{error::Simple, pratt::{infix, prefix, right}, primitive::{choice, just}, recursive::recursive, text::{self}, Parser};

use crate::solver::{Expression, SATInstance};

// pub type ParseResult<T = ()> = Result<T, Simple<char>>;

fn parser<'a>() -> impl Parser<'a, &'a str, Expression> {
    recursive(|expr| {
        let variable = text::ascii::ident().map(|s: &str| Expression::Variable(s.to_string()));
        let constant = choice((
                just('0').to(Expression::Constant(false)),
                just('1').to(Expression::Constant(true)),
        ));

        let literal = choice((
                variable,
                constant,
        )).padded();

        let atom = literal.or(expr.delimited_by(just('('), just(')'))).padded();

        let op = |c| just(c).padded();
        atom.pratt((
                prefix(10, op('-'), |expr| Expression::Not(Box::new(expr))),
                infix(right(5), op('&'), |lhs, rhs| Expression::And(Box::new(lhs), Box::new(rhs))),
                infix(right(2), op('|'), |lhs, rhs| Expression::Or(Box::new(lhs), Box::new(rhs))),
        ))
    })
}

fn extract_variables(expression: &Expression) -> Vec<String> {
    match expression {
        Expression::Variable(var) => vec![var.clone()],
        Expression::Constant(_) => vec![],
        Expression::And(lhs, rhs) => {
            let mut variables_lhs = extract_variables(lhs);
            let mut variables_rhs = extract_variables(rhs);
            variables_lhs.append(&mut variables_rhs);
            variables_lhs
        },
        Expression::Or(lhs, rhs) => {
            let mut variables_lhs = extract_variables(lhs);
            let mut variables_rhs = extract_variables(rhs);
            variables_lhs.append(&mut variables_rhs);
            variables_lhs
        },
        Expression::Not(expr) => {
            let variables = extract_variables(expr);
            variables
        },
    }
}

pub fn parse_file(file: &Path) -> SATInstance {
    let content = fs::read_to_string(file).unwrap();

    let parser = parser();
    let expression = parser.parse(&content).into_result().unwrap();
    let variables = extract_variables(&expression);

    SATInstance {
        expression,
        variables,
    }
}
