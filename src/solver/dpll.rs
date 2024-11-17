// Simple DPLL solver implementation.

use std::{collections::HashSet, process::exit};

use chumsky::container::{Container, Seq};
use rand::Rng;
use rand::seq::SliceRandom;

use crate::expression::{expression::{Assignment, VariableId}, normal::{Clause, Literal, CNF}};

use super::instance::{SATInstance, SolverResult};

#[derive(Debug)]
enum DpllSolverResult {
    Sat,
    Unsat,
}

#[derive(Debug)]
struct DpllClause {
    literals: Vec<Literal>,
    is_disabled: bool,
}

#[derive(Debug)]
struct DpllCNF {
    clauses: Vec<DpllClause>,
}

impl DpllCNF {
    fn has_no_clauses(&self) -> bool {
        self.clauses.iter().filter(|clause| !clause.is_disabled).count() == 0
    }

    fn has_empty_clause(&self, assignment: &Assignment) -> bool {
        for clause in self.clauses.iter().filter(|clause| !clause.is_disabled) {
            if clause.literal_count(&assignment) == 0 {
                return true;
            }
        }

        false
    }
}

impl DpllClause {
    fn new(literals: Vec<Literal>, is_disabled: bool) -> Self {
        Self { literals, is_disabled }
    }

    fn literal_count(&self, assignment: &Assignment) -> usize {
        self.literals.iter().filter(|literal| !assignment.values.contains_key(&literal.var_id)).count()
    }
}

impl From<Clause> for DpllClause {
    fn from(value: Clause) -> Self {
        let literals = value.literals.into_iter().map(|literal| Literal::from(literal)).collect::<Vec<_>>();
        Self::new(literals, false)
    }
}

impl From<CNF> for DpllCNF {
    fn from(value: CNF) -> Self {
        let clauses = value.clauses.into_iter().map(|clause| DpllClause::from(clause)).collect::<Vec<_>>();
        Self::new(clauses)
    }
}

impl DpllCNF {
    fn new(clauses: Vec<DpllClause>) -> Self {
        Self { clauses }
    }

    pub fn disable(&mut self, literal: Literal) {
        // disable enabled clauses with literal
        for clause in self.clauses.iter_mut().filter(|clause| !clause.is_disabled) {
            if clause.literals.contains(&literal) {
                clause.is_disabled = true;
            }
        }
    }

    pub fn enable(&mut self, literal: Literal, assignment: &Assignment) {
        // enable disabled clauses with literal (if no other positive literals are set)
        'next_clause: for clause in self.clauses.iter_mut().filter(|clause| clause.is_disabled) {
            if clause.literals.contains(&literal) {
                for clause_literal in &clause.literals {
                    if assignment.values.get(&clause_literal.var_id).is_some_and(|value| *value == clause_literal.value) {
                        continue 'next_clause;
                    }
                }
                clause.is_disabled = false;
            }
        }
    }
}

fn remove_unit_clauses(cnf: &mut DpllCNF, assignment: &mut Assignment, new_assignments: &mut Vec<Literal>) {
    let mut keep_going = true;

    while keep_going {
        keep_going = false;

        // find unit clause literal for which the value is known
        let unit_clause_literal = cnf.clauses
            .iter()
            .filter(|clause| !clause.is_disabled)
            .find(|clause| clause.literal_count(&assignment) == 1)
            .map(|clause| clause.literals.iter().filter(|literal| !assignment.values.contains_key(&literal.var_id)).next().expect("We just checked this")).copied();

        if let Some(literal) = unit_clause_literal {
            keep_going = true;

            // insert into assignment
            assignment.values.insert(literal.var_id, literal.value);
            new_assignments.push(literal);

            // reduce cnf
            cnf.disable(literal);
        }
    }
}

fn eliminate_pure_literals(cnf: &mut DpllCNF, assignment: &mut Assignment, new_assignments: &mut Vec<Literal>) {
    // find pure literals
    let mut pure_literals: HashSet<Literal> = HashSet::new();
    let mut impure_literals: HashSet<Literal> = HashSet::new();

    for clause in cnf.clauses.iter().filter(|clause| !clause.is_disabled) {
        for literal in clause.literals.iter().filter(|literal| !assignment.values.contains_key(&literal.var_id)).map(|literal| literal).copied() {
            if pure_literals.contains(&literal.not()) {
                pure_literals.remove(&literal.not());

                // don't include in the future
                impure_literals.insert(literal);
                impure_literals.insert(literal.not());
            } else if !impure_literals.contains(&literal) {
                pure_literals.insert(literal);
            }
        }
    }

    for pure_literal in pure_literals {
        // assign values to pure literals to make them true
        assignment.values.insert(pure_literal.var_id, pure_literal.value);
        new_assignments.push(pure_literal);

        // disable clauses that contain pure literals
        cnf.disable(pure_literal);
    }
}

fn choose_variable(cnf: &DpllCNF, max_id: VariableId, assignment: &Assignment) -> Option<VariableId> {
    if assignment.values.len() < usize::try_from(max_id).unwrap() / 2 {
        loop {
            let varid_rand = rand::thread_rng().gen_range(0..=max_id);
            if !assignment.values.contains_key(&varid_rand) {
                return Some(varid_rand);
            }
        }
    } else {
        let available_varids = (0..=max_id).filter(|id| !assignment.values.contains_key(id)).collect::<Vec<_>>();
        available_varids.choose(&mut rand::thread_rng()).map(|v| *v)
    }
}

fn solve_dpll_recursive(cnf: &mut DpllCNF, assignment: &mut Assignment, max_id: VariableId) -> DpllSolverResult {
    // keep track of new assignments so they can be removed on backtrack
    let mut new_assignments: Vec<Literal> = Vec::new();

    // try to find solution by repeatedly applying simple steps
    remove_unit_clauses(cnf, assignment, &mut new_assignments);
    eliminate_pure_literals(cnf, assignment, &mut new_assignments);

    // no clauses left => solution found
    if cnf.has_no_clauses() {
        return DpllSolverResult::Sat;
    }

    // empty clause left => unsat
    if cnf.has_empty_clause(assignment) {
        // restore
        for new_literal in new_assignments.into_iter() {
            assignment.values.remove(&new_literal.var_id);
            cnf.enable(new_literal, assignment);
        }

        return DpllSolverResult::Unsat;
    }

    // now we need to guess
    let var_id = choose_variable(&cnf, max_id, &assignment).expect("There has to be a variable left");

    // try with var_id set to true
    assignment.values.insert(var_id, true);
    cnf.disable(Literal::new(var_id, true));

    if let DpllSolverResult::Sat = solve_dpll_recursive(cnf, assignment, max_id) {
        return DpllSolverResult::Sat;
    }

    // restore
    assignment.values.remove(&var_id);
    cnf.enable(Literal::new(var_id, true), assignment);

    // try with var_id set to false
    assignment.values.insert(var_id, false);
    cnf.disable(Literal::new(var_id, false));

    if let DpllSolverResult::Sat = solve_dpll_recursive(cnf, assignment, max_id) {
        return DpllSolverResult::Sat;
    }

    // didn't work? too bad => Unsat

    // restore
    assignment.values.remove(&var_id);
    cnf.enable(Literal::new(var_id, false), assignment);

    for new_literal in new_assignments.into_iter() {
        assignment.values.remove(&new_literal.var_id);
        cnf.enable(new_literal, assignment);
    }

    DpllSolverResult::Unsat
}

pub fn solve_dpll(instance: SATInstance, initial_assignment: Assignment) -> SolverResult {
    let max_id = VariableId::try_from(instance.var_to_str.len() - 1).expect("Couldn't convert to variable id");

    // reduce cnf according to initial assignment
    let mut cnf: DpllCNF = CNF::from(instance.expression).into();
    let mut assignment = initial_assignment.clone();

    for (var_id, value) in assignment.values.iter() {
        cnf.disable(Literal::new(*var_id, *value).into());
    }

    match solve_dpll_recursive(&mut cnf, &mut assignment, max_id) {
        DpllSolverResult::Sat => SolverResult::Sat(Some(assignment)),
        DpllSolverResult::Unsat => SolverResult::Unsat,
    }
}

#[test]
fn test_disable() {
    let lit0 = Literal::new(0, false);
    let lit1 = Literal::new(1, true);
    let lit2 = Literal::new(2, true);
    let lit3 = Literal::new(3, false);

    let mut cnf = DpllCNF::new(vec![
        DpllClause::new(vec![lit0, lit1], false),
        DpllClause::new(vec![lit1, lit2], false),
        DpllClause::new(vec![lit3], false),
    ]);

    let mut assignment = Assignment::from([(1, true)]);

    cnf.disable(lit2);

    assignment.values.remove(&1);
    cnf.enable(lit2, &assignment);
}
