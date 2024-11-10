use std::{collections::HashMap, error::Error, path::{Path, PathBuf}};

use parser::parse_file;

use crate::{expression::Assignment, solver::solve_dpll};

mod parser;
mod solver;
mod expression;

fn main() {
    let instance = parse_file(&PathBuf::from("./formula.sat"));

    println!("{}", instance);

    let solution = solve_dpll(&instance);
    eprintln!("solution = {:#?}", solution);
}
