#![allow(unused)]
#![allow(dead_code)]

use async_trait::async_trait;
use ethers::prelude::{Address, Signature as EvmSignature};
use solana_client::rpc_client::RpcClient as SolClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature as SolSignature;
use web3::types::U256;

pub type EvmClient = web3::Web3<web3::transports::Http>;
pub type Balance = u128;

trait Platform: Sync {
    const ID: u32;

    type User: Identity + Sync;
    type Client: Sync;
}

pub enum EvmChain {
    Ethereum,
    Polygon,
}

pub struct Solana;

impl Platform for EvmChain {
    const ID: u32 = 0;
    type User = EvmIdentity;
    type Client = EvmClient;
}

impl Platform for Solana {
    const ID: u32 = 1;
    type User = SolIdentity;
    type Client = SolClient;
}

#[async_trait]
trait ExternalRequirement {
    type Source: Platform;
    type ExternalData: Sized + Sync + Send;

    async fn check(
        &self,
        user: &<Self::Source as Platform>::User,
        client: &<Self::Source as Platform>::Client,
    ) -> Result<bool, String>;
}

trait Identity {
    type Identifier: Sized + PartialEq + Sync;
    fn verify(&self) -> bool;
    fn identifier(&self) -> &Self::Identifier;
}

struct EvmIdentity {
    address: Address,
    signature: EvmSignature,
    msg: String,
}

struct SolIdentity {
    pubkey: Pubkey,
    signature: SolSignature,
    msg: String,
}

impl Identity for EvmIdentity {
    type Identifier = Address;
    fn verify(&self) -> bool {
        // TODO check msg is something that we expect
        self.signature
            .verify(self.msg.as_str(), self.address)
            .is_ok()
    }

    fn identifier(&self) -> &Self::Identifier {
        &self.address
    }
}

impl Identity for SolIdentity {
    type Identifier = Pubkey;
    fn verify(&self) -> bool {
        // TODO check msg is something that we expect
        self.signature
            .verify(&self.pubkey.to_bytes(), self.msg.as_bytes())
    }

    fn identifier(&self) -> &Self::Identifier {
        &self.pubkey
    }
}

enum TokenType<T: Identity> {
    Fungible {
        address: T::Identifier,
    },
    NonFungible {
        address: T::Identifier,
        id: Option<u32>,
    },
}

enum Relation {
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

impl Relation {
    pub fn assert<T: PartialEq + PartialOrd>(&self, a: &T, b: &T) -> bool {
        match self {
            Relation::Equal => a == b,
            Relation::Greater => a > b,
            Relation::GreaterOrEqual => a >= b,
            Relation::Less => a < b,
            Relation::LessOrEqual => a <= b,
        }
    }
}

struct RequiredBalance<T: Platform, B> {
    token_type: Option<TokenType<<T as Platform>::User>>,
    relation: Relation,
    amount: B,
}

struct Allowlist<T: Platform>(Vec<<<T as Platform>::User as Identity>::Identifier>);

impl<T: Platform> Allowlist<T> {
    pub fn is_member(&self, identifier: &<<T as Platform>::User as Identity>::Identifier) -> bool {
        self.0.iter().any(|id| id == identifier)
    }
}

#[async_trait]
impl ExternalRequirement for RequiredBalance<Solana, u64> {
    type Source = Solana;
    type ExternalData = u64;

    async fn check(
        &self,
        user: &<Self::Source as Platform>::User,
        client: &<Self::Source as Platform>::Client,
    ) -> Result<bool, String> {
        let balance = match self.token_type {
            None => client
                .get_balance(user.identifier())
                .map_err(|e| e.to_string())?,
            Some(TokenType::Fungible { address }) => todo!(),
            Some(TokenType::NonFungible { address, id }) => todo!(),
        };
        Ok(self.relation.assert(&balance, &self.amount))
    }
}

#[async_trait]
impl ExternalRequirement for RequiredBalance<EvmChain, U256> {
    type Source = EvmChain;
    type ExternalData = U256;

    async fn check(
        &self,
        user: &<Self::Source as Platform>::User,
        client: &<Self::Source as Platform>::Client,
    ) -> Result<bool, String> {
        let balance = match self.token_type {
            None => client
                .eth()
                .balance(*user.identifier(), None)
                .await
                .map_err(|e| e.to_string())?,
            Some(TokenType::Fungible { address }) => todo!(),
            Some(TokenType::NonFungible { address, id }) => todo!(),
        };
        Ok(self.relation.assert(&balance, &self.amount))
    }
}

enum Requirement {
    Free,
    EvmSignature(EvmSignature),
    SolSignature(SolSignature),
    EvmBalance(RequiredBalance<EvmChain, U256>),
    SolBalance(RequiredBalance<Solana, u64>),
    EvmAllowlist(Allowlist<EvmChain>),
    SolAllowlist(Allowlist<Solana>),
}

// NOTE in the long term this could be merged into a single client
// that sends specific json to specific endpoints because
// an evm client and a sol client are just wrappers around a
// reqwest client
struct Evaluator {
    evm: EvmClient,
    sol: SolClient,
}

/*
impl Evaluator {
    async fn evaluate<'a>(
        &self,
        identities: &'a HashMap<u32, &'a dyn Identity>,
        requirements: &[Requirement],
    ) -> HashMap<requiem::TerminalId, bool> {
        let mut evaluations = HashMap::<requiem::TerminalId, bool>::new();
        let mut externals = HashMap::new();
        for (i, requirement) in requirements.iter().enumerate() {
            match requirement {
                Free => evaluations.insert(i, true),
                Requirement::EvmSignature(sig) | Requirement::SolSignature(sig) => {
                    evaluations.insert(i, sig.verify())
                }
                Requirement::EvmAllowlist(list) => {
                    let result = if let Some(id) = identities.get(EvmChain::ID) {
                        list.is_member(id)
                    } else {
                        false
                    };
                    evaluations.insert(i, result);
                }
                Requirement::SolAllowlist(list) => {
                    todo!();
                }
                _ => todo!(),
            }
        }
        todo!()
    }
}
*/
fn main() {}
