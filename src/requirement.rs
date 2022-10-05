use async_trait::async_trait;
#[cfg(feature = "ethereum")]
use ethers::{Address, Signature};
use reqwest::Client;

type RequirementError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait Requirement {
    // NOTE reqwest suggests that a reqwest client is reused
    // instead of creating one for every request. Thus this
    // function should accept a reference to something (e.g. a request client)
    async fn check<Q>(&self, querier: &Q) -> Result<bool, RequirementError>;
}

/* Desired behaviour
 *
 * (a AND b OR (c NAND d)) OR (e NOR f) all symbolic values `a`, `b`, `c`, `d`,
 * `e`, `f` should implement a trait that allows us to resolve them into
 * boolean values
 *
 * the collected boolean values are then evaluated along the boolean logic
 * gates
 *
 * i.e. `a` is a requirement that eventually resolves into a boolean
 *
 * initiate all requirements to be executed in a parallel way, i.e. (try_)join
 * futures
 *
 * once all futures resolved, aggregate them via the evaluation logic
 *
 * could use the `boolean_expressions` crate for this
*/
