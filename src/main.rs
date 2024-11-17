use std::sync::mpsc;

use parser::ParsedExpression;
use solver::instance::SATInstance;

use crate::{expression::expression::Assignment, solver::{dpll::solve_dpll, instance::SolverResult}};

mod parser;
mod solver;
mod expression;

pub fn var_string(row: u32, col: u32, k: u32) -> String {
    format!("v{}{}{}", row, col, k)
}

const N: u32 = 9;
const THREADS: usize = 24;

// sudoku as an example
fn encode_sudoku() -> SATInstance {
    let mut expression = ParsedExpression::Constant(true);

    // each number at least once in each...

    // ...row
    for number in 0..N {
        for row in 0..N {
            let mut subexpr = ParsedExpression::Constant(false);
            for col in 0..N {
                subexpr = ParsedExpression::Or(Box::new(subexpr), Box::new(ParsedExpression::Variable(var_string(row, col, number))));
            }

            expression = ParsedExpression::And(Box::new(expression), Box::new(subexpr));
        }
    }

    // ...column
    for number in 0..N {
        for col in 0..N {
            let mut subexpr = ParsedExpression::Constant(false);
            for row in 0..N {
                subexpr = ParsedExpression::Or(Box::new(subexpr), Box::new(ParsedExpression::Variable(var_string(row, col, number))));
            }

            expression = ParsedExpression::And(Box::new(expression), Box::new(subexpr));
        }
    }

    // ...block
    for number in 0..N {
        for block_row in 0..N.isqrt() {
            for block_col in 0..N.isqrt() {
                let mut subexpr = ParsedExpression::Constant(false);

                for off_row in 0..N.isqrt() {
                    for off_col in 0..N.isqrt() {
                        let row = block_row * N.isqrt() + off_row;
                        let col = block_col * N.isqrt() + off_col;

                        subexpr = ParsedExpression::Or(Box::new(subexpr), Box::new(ParsedExpression::Variable(var_string(row, col, number))));
                    }
                }

                expression = ParsedExpression::And(Box::new(expression), Box::new(subexpr));
            }
        }
    }

    // at most one number in each cell
    for row in 0..N {
        for col in 0..N {
            for number0 in 0..N {
                let mut subexpr = ParsedExpression::Constant(true);

                for number1 in (number0 + 1)..N {
                    let var0 = Box::new(ParsedExpression::Variable(var_string(row, col, number0)));
                    let var1 = Box::new(ParsedExpression::Variable(var_string(row, col, number1)));
                    let not = ParsedExpression::Not(Box::new(ParsedExpression::And(var0, var1)));
                    subexpr = ParsedExpression::And(Box::new(subexpr), Box::new(not));
                }

                expression = ParsedExpression::And(Box::new(expression), Box::new(subexpr));
            }
        }
    }

    SATInstance::from(expression)
}

fn main() {
    // let sudoku_instance = parse_file(&PathBuf::from("./formula.sat"));
    let sudoku_instance = encode_sudoku();
    eprintln!("sudoku_instance.interned_variables = {:#?}", sudoku_instance.str_to_var);

    let initial_assignment = Assignment::from([
        (*sudoku_instance.str_to_var.get(&var_string(0, 3, 1)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(0, 4, 5)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(0, 6, 6)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(0, 8, 0)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(1, 0, 5)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(1, 1, 7)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(1, 4, 6)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(1, 7, 8)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(2, 0, 0)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(2, 1, 8)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(2, 5, 3)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(2, 6, 4)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(3, 0, 7)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(3, 1, 1)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(3, 3, 0)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(3, 7, 3)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(4, 2, 3)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(4, 3, 5)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(4, 5, 1)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(4, 6, 8)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(5, 1, 4)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(5, 5, 2)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(5, 7, 1)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(5, 8, 7)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(6, 2, 8)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(6, 3, 2)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(6, 7, 6)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(6, 8, 3)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(7, 1, 3)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(7, 4, 4)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(7, 7, 2)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(7, 8, 5)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(8, 0, 6)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(8, 2, 2)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(8, 4, 0)).unwrap(), true),
        (*sudoku_instance.str_to_var.get(&var_string(8, 5, 7)).unwrap(), true),
        ]);

    let initial_assignment = Assignment::default();

    let (tx, rx) = mpsc::channel();

    let mut join_handles = Vec::new();

    for _ in 0..THREADS {
        let thread_tx = tx.clone();
        let thread_instance = sudoku_instance.clone();
        let thread_assignment = initial_assignment.clone();
        join_handles.push(std::thread::spawn(move || {
            if let SolverResult::Sat(assignment) = solve_dpll(thread_instance, thread_assignment) {
                thread_tx.send(assignment.unwrap()).unwrap();
            }
        }));
    }

    loop {
        let assignment = rx.recv().unwrap();
        println!("Sat");
        eprintln!("sudoku_instance.expression.evaluate(assignment) = {:#?}", sudoku_instance.expression.clone().evaluate(&assignment));
        for row in 0..N {
            for col in 0..N {
                for number in 0..N {
                    let var = sudoku_instance.str_to_var.get(&var_string(row, col, number)).unwrap();
                    if *assignment.values.get(var).unwrap() {
                        print!("{} ", number);
                        break;
                    }
                }
            }
            println!("");
        }
    }
}
