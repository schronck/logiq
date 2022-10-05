mod bdd;
pub use bdd::build_bdd;

use crate::requirement::Requirement;
use crate::token::ParsedTokens;
use boolean_expression::{BDDFunc, BDD};

use std::collections::HashMap;

type TerminalId = char;

struct Evaluator<R> {
    bdd: BDD<TerminalId>,
    bdd_func: BDDFunc,
    terminal_evals: HashMap<TerminalId, R>,
}

impl<R: Requirement> Evaluator<R> {
    pub fn new(tokens: &ParsedTokens, requirements: Vec<R>) -> Result<Self, String> {
        let mut bdd = BDD::<char>::new();
        let bdd_func = build_bdd(&mut bdd, tokens)?;
        let bdd_terminal_ids = bdd.labels();

        if requirements.len() != bdd_terminal_ids.len() {
            return Err(format!(
                "number of requirements ({}) must match number of BDD terminals ({})",
                requirements.len(),
                bdd_terminal_ids.len()
            ));
        }

        let terminal_evals = bdd_terminal_ids
            .into_iter()
            .zip(requirements.into_iter())
            .collect();

        Ok(Self {
            bdd,
            bdd_func,
            terminal_evals,
        })
    }

    pub async fn evaluate(&self) -> bool {
        todo!()
    }
}
