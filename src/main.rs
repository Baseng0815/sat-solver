#![feature(trace_macros)]
trace_macros!(true);

use std::sync::mpsc;

use parser::ParsedExpression;
use solver::instance::SATInstance;

use crate::{expression::expression::Assignment, solver::{dpll::solve_dpll, instance::SolverResult}};

mod parser;
mod solver;
mod expression;

fn main() {
    // let sudoku_instance = parse_file(&PathBuf::from("./formula.sat"));
    let parsed_expression = prop_expr!((a & (b | c)) & (-d));
    let instance = SATInstance::from(parsed_expression);
    eprintln!("instance = {:#?}", instance);
    eprintln!("instance.expression = {}", instance.expression);

    let assignment = Assignment::default();
    let solution = solve_dpll(instance, assignment);
    eprintln!("solution = {:#?}", solution);
}
