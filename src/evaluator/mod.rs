mod bdd;
pub use bdd::build_bdd;

use crate::requirement::Requirement;
use crate::token::ParsedTokens;
use boolean_expression::{BDDFunc, BDD};
use futures::future::try_join_all;

use std::collections::HashMap;

type TerminalId = char;

pub struct Evaluator<R> {
    bdd: BDD<TerminalId>,
    bdd_func: BDDFunc,
    requirements: HashMap<TerminalId, R>,
    evals: HashMap<TerminalId, bool>,
}

impl<R: Requirement> Evaluator<R> {
    pub fn new(
        tokens: &ParsedTokens,
        requirements: HashMap<TerminalId, R>,
    ) -> Result<Self, String> {
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

        Ok(Self {
            bdd,
            bdd_func,
            requirements,
            evals: HashMap::new(),
        })
    }

    pub async fn evaluate<Q: Sync>(&mut self, querier: &Q) -> Result<bool, String> {
        let future_evals = self
            .requirements
            .values()
            .map(|req| req.check(querier))
            .collect::<Vec<_>>();
        let evals = try_join_all(future_evals)
            .await
            .map_err(|e| e.to_string())?;
        self.evals = self
            .requirements
            .keys()
            .copied()
            .zip(evals.into_iter())
            .collect();
        Ok(self.bdd.evaluate(self.bdd_func, &self.evals))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::requirement::RequirementResult;
    use async_trait::async_trait;
    use reqwest::Client;

    use std::str::FromStr;

    struct Free;

    #[async_trait]
    impl Requirement for Free {
        async fn check<Q: Sync>(&self, _querier: &Q) -> RequirementResult {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_free() {
        let tokens = ParsedTokens::from_str("a").unwrap();
        let mut requirements = HashMap::new();
        requirements.insert('a', Free);
        let mut evaluator = Evaluator::new(&tokens, requirements).unwrap();
        let client = Client::new();
        assert!(evaluator.evaluate(&client).await.unwrap());
    }
}
