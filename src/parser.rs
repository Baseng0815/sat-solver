use std::{error::Error, fs, path::Path};

use chumsky::{error::Simple, pratt::{infix, prefix, right}, primitive::{choice, just}, recursive::recursive, text::{self}, Parser};

use crate::solver::Expression;

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
                prefix(10, op('-'), |expr| {
                    // fold multiple nots into one
                    if let Expression::Not(inner) = expr {
                        *inner
                    } else {
                        Expression::Not(Box::new(expr))
                    }
                }),
                infix(right(5), op('&'), |lhs, rhs| Expression::And(Box::new(lhs), Box::new(rhs))),
                infix(right(2), op('|'), |lhs, rhs| Expression::Or(Box::new(lhs), Box::new(rhs))),
        ))
    })
}

pub fn parse_file(file: &Path) -> Result<Expression, Box<dyn Error>> {
    let content = fs::read_to_string(file)?;

    let parser = parser();
    match parser.parse(&content).into_result() {
        Ok(expr) => Ok(expr),
        Err(errors) => {
            panic!("Encountered errors while parsing: {:#?}", errors);
        }
    }
}
