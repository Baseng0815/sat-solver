use std::collections::{HashMap, HashSet};

use chumsky::container::{Container, Seq};

use crate::expression::{Assignment, Expression, Literal, SATInstance, VariableId, CNF};

#[derive(Debug)]
pub enum SolverResult {
    Sat(Option<Assignment>),
    Unsat
}

fn remove_unit_clauses(cnf: &mut CNF, assignment: &mut Assignment) {
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
            }
        }
    }
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

fn choose_variable(cnf: &CNF, max_id: VariableId, assignment: &Assignment) -> Option<VariableId> {
    for id in 0..=max_id {
        let variable_id = VariableId::try_from(id).expect("Couldn't convert to variable id");
        if !assignment.values.contains_key(&variable_id) {
            return Some(variable_id);
        }
    }

    None
}

pub fn solve_dpll(instance: &SATInstance) -> SolverResult {
    let max_id = VariableId::try_from(instance.interned_variables.len() - 1).expect("Couldn't convert to variable id");

    let mut stack = vec![(CNF::from(instance.expression.clone()), Assignment::default())];

    eprintln!("CNF = {:#?}", stack[0].0);

    loop {
        let (mut cnf, mut assignment) = stack.pop().expect("There must be something left at this point");

        // try to find solution by repeatedly applying simple steps
        remove_unit_clauses(&mut cnf, &mut assignment);
        eliminate_pure_literals(&mut cnf, &mut assignment);

        // no clauses left => solution found
        if cnf.clauses.is_empty() {
            return SolverResult::Sat(Some(assignment));
        }

        // empty clause left => unsat
        for clause in &cnf.clauses {
            if clause.literals.is_empty() {
                return SolverResult::Unsat;
            }
        }

        // now we need to guess
        let var_id = match choose_variable(&cnf, max_id, &assignment) {
            Some(var_id) => var_id,
            None => return SolverResult::Unsat // nothing left => unsat
        };

        // TODO use better data structure to prevent cloning (only track differences)
        let mut assignment_true = assignment.clone();
        let mut assignment_false = assignment.clone();
        assignment_true.values.insert(var_id, true);
        assignment_false.values.insert(var_id, false);

        let mut cnf_true = cnf.clone();
        let mut cnf_false = cnf.clone();
        cnf_true.reduce(Literal::new(var_id, true));
        cnf_false.reduce(Literal::new(var_id, false));

        stack.push((cnf_true, assignment_true));
        stack.push((cnf_false, assignment_false));
    }
}

