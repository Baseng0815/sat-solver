use std::{collections::HashMap, error::Error, path::{Path, PathBuf}};

use parser::parse_file;

mod parser;
mod solver;

fn main() {
    let instance = parse_file(&PathBuf::from("./formula.sat"));
    let assignment = HashMap::from([
        ("x1".to_string(), true),
        ("x3".to_string(), false),
    ]);

    eprintln!("instance.expression = {}", instance.expression);

    let instance_dnf = instance.expression.to_dnf();
    eprintln!("instance_dnf = {}", instance_dnf);

    // let instance_dnf = instance_dnf.to_dnf();
    // eprintln!("instance_dnf = {}", instance_dnf);

    // let instance_dnf = instance_dnf.to_dnf();
    // eprintln!("instance_dnf = {:#?}", instance_dnf);
}
