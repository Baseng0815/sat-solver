use std::{collections::HashMap, error::Error, path::{Path, PathBuf}};

use parser::parse_file;

use crate::solver::evaluate;

mod parser;
mod solver;

fn main() -> Result<(), Box<dyn Error>> {
    let expression = parse_file(&PathBuf::from("./formula.sat"))?;
    let assignment = HashMap::from([
        ("x1".to_string(), true),
        ("x3".to_string(), false),
    ]);

    eprintln!("expression = {:#?}", expression);
    eprintln!("evaluate(expression) = {:#?}", evaluate(&expression, &assignment));

    Ok(())
}
