mod bdd;
pub use bdd::BDDData;

use crate::requirement::Requirement;
use crate::TerminalId;
use boolean_expression::{BDDFunc, BDD};
use futures::future::try_join_all;

use std::collections::HashMap;

pub type RequirementsVec<'a, Q> = Vec<&'a dyn Requirement<Querier = Q>>;

pub struct Evaluator<'a, Q> {
    bdd: BDD<TerminalId>,
    root_bdd_func: BDDFunc,
    requirements: RequirementsVec<'a, Q>,
    evals: HashMap<TerminalId, bool>,
}

impl<'a, Q: Sync> Evaluator<'a, Q> {
    pub fn new(
        bdd_data: BDDData,
        requirements: RequirementsVec<'a, Q>,
    ) -> Result<Self, anyhow::Error> {
        let BDDData { bdd, root_bdd_func } = bdd_data;
        let bdd_terminal_ids = bdd.labels();

        anyhow::ensure!(
            requirements.len() == bdd_terminal_ids.len(),
            "number of requirements ({}) must match number of BDD terminals ({})",
            requirements.len(),
            bdd_terminal_ids.len()
        );

        Ok(Self {
            bdd,
            root_bdd_func,
            requirements,
            evals: HashMap::new(),
        })
    }

    pub async fn evaluate(&mut self, querier: &Q) -> Result<bool, anyhow::Error> {
        let future_evals = self
            .requirements
            .iter()
            .map(|req| req.check(querier))
            .collect::<Vec<_>>();
        let evals = try_join_all(future_evals)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        self.evals = evals.into_iter().enumerate().collect();
        Ok(self.bdd.evaluate(self.root_bdd_func, &self.evals))
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use crate::requirement::RequirementResult;
    use async_trait::async_trait;
    use ethers::prelude::{Address, LocalWallet, Signature};
    use ethers::signers::Signer;

    #[derive(Clone, Debug)]
    struct Free;

    #[async_trait]
    impl Requirement for Free {
        type Querier = u8;
        async fn check(&self, _querier: &Self::Querier) -> RequirementResult {
            Ok(true)
        }
    }

    #[derive(Clone, Debug)]
    struct ControlsAddress {
        address: Address,
        signature: Signature,
        msg: String,
    }

    #[async_trait]
    impl Requirement for ControlsAddress {
        type Querier = u8;
        async fn check(&self, _querier: &Self::Querier) -> RequirementResult {
            Ok(self
                .signature
                .verify(self.msg.as_str(), self.address)
                .is_ok())
        }
    }

    #[tokio::test]
    async fn free_requirement() {
        let requirements = vec![&Free as &dyn Requirement<Querier = u8>];
        let client = 0u8; // querier can be any type

        let mut evaluator = Evaluator::new(BDDData::from_str("0").unwrap(), requirements).unwrap();
        assert!(evaluator.evaluate(&client).await.unwrap());
    }

    #[tokio::test]
    async fn reused_terminals() {
        let requirements = vec![&Free as &dyn Requirement<Querier = u8>; 3];
        let client = 0u8; // querier can be any type

        let mut evaluator = Evaluator::new(
            BDDData::from_str("(0 AND 1) OR (0 AND 2)").unwrap(),
            requirements.clone(),
        )
        .unwrap();

        assert!(evaluator.evaluate(&client).await.unwrap());

        let mut evaluator = Evaluator::new(
            BDDData::from_str("(0 NAND 1) OR (0 AND 2)").unwrap(),
            requirements.clone(),
        )
        .unwrap();
        assert!(evaluator.evaluate(&client).await.unwrap());

        let mut evaluator = Evaluator::new(
            BDDData::from_str("(0 NAND 1) OR (0 NAND 2)").unwrap(),
            requirements,
        )
        .unwrap();
        assert!(!evaluator.evaluate(&client).await.unwrap());
    }

    #[tokio::test]
    async fn eth_signatures() {
        let wallet_a = LocalWallet::new(&mut rand_core::OsRng);
        let wallet_b = LocalWallet::new(&mut rand_core::OsRng);
        let msg = "requiem aeternam dona eis";
        let signature_a = wallet_a.sign_message(msg).await.unwrap();
        let signature_b = wallet_b.sign_message(msg).await.unwrap();
        let client = 0u8; // querier can be any type

        let controls_address_a = ControlsAddress {
            address: wallet_a.address(),
            signature: signature_a,
            msg: msg.to_string(),
        };

        let controls_address_b = ControlsAddress {
            address: wallet_b.address(),
            signature: signature_b,
            msg: msg.to_string(),
        };

        let requirements = vec![
            &controls_address_a as &dyn Requirement<Querier = u8>,
            &controls_address_b as &dyn Requirement<Querier = u8>,
        ];

        let mut evaluator =
            Evaluator::new(BDDData::from_str("0 AND 1").unwrap(), requirements.clone()).unwrap();
        assert!(evaluator.evaluate(&client).await.unwrap());

        let mut evaluator =
            Evaluator::new(BDDData::from_str("0 NAND 1").unwrap(), requirements).unwrap();
        assert!(!evaluator.evaluate(&client).await.unwrap());
    }

    #[tokio::test]
    async fn different_types_same_trait() {
        struct U32(u32);
        struct U128(u128);

        #[async_trait]
        impl Requirement for U32 {
            type Querier = reqwest::Client;
            async fn check(&self, _querier: &Self::Querier) -> RequirementResult {
                if self.0 < 100 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }

        #[async_trait]
        impl Requirement for U128 {
            type Querier = reqwest::Client;
            async fn check(&self, _querier: &Self::Querier) -> RequirementResult {
                if self.0 > 100 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }

        let requirements = vec![
            &U128(123) as &dyn Requirement<Querier = reqwest::Client>,
            &U32(400) as &dyn Requirement<Querier = reqwest::Client>,
            &U32(1) as &dyn Requirement<Querier = reqwest::Client>,
            &U128(2) as &dyn Requirement<Querier = reqwest::Client>,
        ];

        let mut evaluator = Evaluator::new(
            BDDData::from_str("0 OR (1 AND 2) OR 3").unwrap(),
            requirements,
        )
        .unwrap();
        let client = reqwest::Client::new();
        // true || (false && true) || false
        assert!(evaluator.evaluate(&client).await.unwrap());
    }
}
