use async_trait::async_trait;

pub type RequirementError = Box<dyn std::error::Error + Send + Sync>;
pub type RequirementResult = Result<bool, RequirementError>;

#[async_trait]
pub trait Requirement {
    // NOTE reqwest suggests that a reqwest client is reused
    // instead of creating one for every request. Thus this
    // function should accept a reference to something (e.g. a request client)
    async fn check<Q: Sync>(&self, querier: &Q) -> RequirementResult;
}
