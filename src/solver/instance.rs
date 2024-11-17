// SAT problem instance and solution representation.

use std::{collections::HashMap, fmt::Display};

use crate::expression::expression::{Assignment, Expression, VariableId};

#[derive(Debug, Clone)]
pub struct SATInstance {
    pub expression: Expression,
    pub var_to_str: HashMap<VariableId, String>,
    pub str_to_var: HashMap<String, VariableId>,
}

#[derive(Debug)]
pub enum SolverResult {
    Sat(Option<Assignment>),
    Unsat
}

impl SATInstance {
    pub fn new(expression: Expression, var_to_str: HashMap<VariableId, String>) -> Self {
        let mut str_to_var = HashMap::new();
        for (var, str) in var_to_str.iter() {
            str_to_var.insert(str.clone(), *var);
        }
        Self { expression, var_to_str, str_to_var }
    }
}

impl Display for SATInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Instance containing {} variables", self.var_to_str.len())?;

        write!(f, "Expression: {}", self.expression)
    }
}
