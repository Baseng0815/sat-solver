use std::collections::HashMap;

use crate::expression::{Assignment, Expression, SATInstance, CNF};

fn solve_dpll_recursive(cnf: &CNF, literals:)

pub fn solve_dpll(instance: &SATInstance) -> Option<Assignment> {
    let cnf = CNF::from(instance.expression.clone());
    solve_dpll_recursive(&cnf, literals)

    // unit propagation: delete clauses with single literal since we know its value
    // for clause in clauses {
    //     if clause
    // }

    None
}

