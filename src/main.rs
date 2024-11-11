use std::{collections::HashMap, error::Error, path::{Path, PathBuf}};

use chumsky::container::Container;
use expression::{Clause, Expression, Literal, VariableId, CNF};
use parser::parse_file;

use crate::{expression::Assignment, solver::{solve_dpll, SolverResult}};

mod parser;
mod solver;
mod expression;

// sudoku as an example
fn encode_sudoku() -> CNF {
    let var_string = |y: u32, x: u32, k: u32| format!("D_{}_{}_{}", x, y, k);
    let var = |y: u32, x: u32, k: u32| VariableId::try_from(y * 81 + x * 9 + k).unwrap();

    let mut clauses = Vec::new();
    // every number has to appear at least once in each...

    // ...row
    for row in 0..9 {
        let mut clause = Clause::default();
        for col in 0..9 {
            for number in 0..9 {
                clause.literals.push(Literal::Variable(var(row, col, number)));
            }
        }

        clauses.push(clause);
    }

    CNF::new(clauses)
}

fn main() {
    let instance = parse_file(&PathBuf::from("./formula.sat"));

    println!("{}", instance);

    let solution = solve_dpll(&instance);
    eprintln!("solution = {:#?}", solution);

    // if let SolverResult::Sat(solution) = solution {
    //     if let Some(assignment) = solution {
    //         eprintln!("instance.expression.evaluate(&assignment) = {:#?}", instance.expression.evaluate(&assignment));
    //     }
    // }
}
