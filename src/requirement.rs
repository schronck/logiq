use async_trait::async_trait;

pub type RequirementError = Box<dyn std::error::Error + Send + Sync>;
pub type RequirementResult = Result<bool, RequirementError>;

#[async_trait]
pub trait Requirement {
    type Querier: Sync;
    // NOTE reqwest suggests that a reqwest client is reused
    // instead of creating one for every request. Thus this
    // function should accept a reference to something (e.g. a request client)
    async fn check(&self, querier: &Self::Querier) -> RequirementResult;
}

#[cfg(test)]
mod test {
    use super::*;
    use reqwest::Client;
    use serde::Deserialize;

    const BALANCY: &str = "https://balancy.guild.xyz/api/addressTokens?address=";
    const BALANCY_CHAIN: &str = "&chain=";

    #[derive(Deserialize, Debug)]
    struct Erc20 {
        address: String,
        amount: String,
    }

    #[derive(Deserialize, Debug)]
    struct BalancyResponse {
        erc20: Vec<Erc20>,
    }

    struct Erc20Requirement {
        chain: u8,
        address: String,
        erc20address: String,
        relation: String,
        amount: u128,
    }

    #[async_trait]
    impl Requirement for Erc20Requirement {
        type Querier = Client;
        async fn check(&self, querier: &Self::Querier) -> RequirementResult {
            let body: BalancyResponse = querier
                .get(format!(
                    "{BALANCY}{}{BALANCY_CHAIN}{}",
                    self.address, self.chain
                ))
                .send()
                .await?
                .json()
                .await?;

            let balance = body
                .erc20
                .iter()
                .find(|i| i.address == self.erc20address)
                .map(|token| token.amount.parse::<u128>().unwrap())
                .unwrap_or_default();

            match self.relation.as_str() {
                "geq" => Ok(balance >= self.amount),
                _ => unimplemented!(),
            }
        }
    }

    #[tokio::test]
    async fn check_balance() {
        let client = Client::new();
        let requirement = Erc20Requirement {
            chain: 56, // 56 for Bsc
            address: "0xE43878Ce78934fe8007748FF481f03B8Ee3b97DE".to_string(),
            erc20address: "de4e179cc1d3b298216b96893767b9b01a6bc413".to_string(),
            relation: "geq".to_string(),
            amount: 1,
        };
        assert!(requirement.check(&client).await.unwrap());
    }
}
