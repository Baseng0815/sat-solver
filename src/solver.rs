use std::collections::{HashMap, HashSet};

use chumsky::container::{Container, Seq};

use crate::expression::{Assignment, Expression, Literal, SATInstance, VariableId, CNF};

#[derive(Debug)]
pub enum SolverResult {
    Sat(Option<Assignment>),
    Unsat
}

fn remove_unit_clauses(cnf: &mut CNF, assignment: &mut Assignment) -> SolverResult {
    let mut keep_going = true;

    while keep_going {
        keep_going = false;

        // find unit clause literal for which the value is known
        let unit_clause_literal = cnf.clauses
            .iter()
            .find(|clause| clause.literals.len() == 1)
            .map(|clause| clause.literals.iter().next().expect("We just checked this"))
            .map(|l| l.clone());

        if let Some(literal) = unit_clause_literal {
            keep_going = true;

            // insert into assignment
            assignment.values.insert(literal.id(), literal.value());

            // remove all clauses with literal since they evaluate to true
            cnf.clauses.retain(|clause| !clause.literals.contains(&literal));

            // remove negative occurence in all remaining clauses
            for clause in cnf.clauses.iter_mut() {
                clause.literals.remove(&literal.not());

                // empty clause arises => unsat
                if clause.literals.is_empty() {
                    return SolverResult::Unsat;
                }
            }
        }
    }

    SolverResult::Sat(None)
}

fn eliminate_pure_literals(cnf: &mut CNF, assignment: &mut Assignment) {
    // find pure literals
    let mut pure_literals: HashSet<Literal> = HashSet::new();
    let mut impure_literals: HashSet<Literal> = HashSet::new();

    for clause in &cnf.clauses {
        for literal in clause.literals.iter() {
            if pure_literals.contains(&literal.not()) {
                pure_literals.remove(&literal.not());

                // don't include in the future
                impure_literals.push(literal.clone());
                impure_literals.push(literal.not());
            } else if !impure_literals.contains(literal) {
                pure_literals.insert(literal.clone());
            }
        }
    }

    // assign values to pure literals to make them true
    for pure_literal in &pure_literals {
        assignment.values.insert(pure_literal.id(), pure_literal.value());
    }

    // remove clauses that contain pure literals
    cnf.clauses.retain(|clause| clause.literals.intersection(&pure_literals).count() == 0);
}

pub fn solve_dpll(instance: &SATInstance) -> SolverResult {
    let mut cnf = CNF::from(instance.expression.clone());
    let mut assignment = Assignment::default();
    eprintln!("CNF = {:#?}", cnf);

    while !cnf.clauses.is_empty() {
        if let SolverResult::Unsat = remove_unit_clauses(&mut cnf, &mut assignment) {
            return SolverResult::Unsat;
        }

        eliminate_pure_literals(&mut cnf, &mut assignment);
    }

    SolverResult::Sat(Some(assignment))
}

