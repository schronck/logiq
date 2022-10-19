use async_trait::async_trait;
use ethers::prelude::{Address, LocalWallet, Signature as EvmSignature};
use solana_client::rpc_client::RpcClient as SolClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature as SolSignature;
use solana_sdk::signer::{keypair::Keypair, Signer};
use web3::transports::Http as HttpTransport;
use web3::types::U256;
use web3::Web3 as EvmClient;

pub type Balance = u128;

trait Platform: Sync {
    type User: Identity + Sync;
    type Client: Sync;
}

pub enum EvmChain {
    Ethereum,
    Polygon,
}

pub struct Solana;

impl Platform for EvmChain {
    type User = EvmIdentity;
    type Client = EvmClient<HttpTransport>;
}

impl Platform for Solana {
    type User = SolIdentity;
    type Client = SolClient;
}

#[async_trait]
trait Requirement {
    type Source: Platform;
    type Extrinsic: Sized + Sync + Send;

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

#[async_trait]
impl<T: Platform + Sync> Requirement for Allowlist<T> {
    type Source = T;
    type Extrinsic = bool;

    async fn check(
        &self,
        user: &<Self::Source as Platform>::User,
        _client: &<Self::Source as Platform>::Client,
    ) -> Result<bool, String> {
        Ok(self.0.iter().any(|id| id == user.identifier()))
    }
}

#[async_trait]
impl Requirement for RequiredBalance<Solana, u64> {
    type Source = Solana;
    type Extrinsic = u64;

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
impl Requirement for RequiredBalance<EvmChain, U256> {
    type Source = EvmChain;
    type Extrinsic = U256;

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

#[async_trait]
impl Requirement for EvmIdentity {
    type Source = EvmChain;
    type Extrinsic = bool;

    async fn check(
        &self,
        _user: &<Self::Source as Platform>::User,
        _client: &<Self::Source as Platform>::Client,
    ) -> Result<bool, String> {
        Ok(self.verify())
    }
}

#[async_trait]
impl Requirement for SolIdentity {
    type Source = Solana;
    type Extrinsic = bool;

    async fn check(
        &self,
        _user: &<Self::Source as Platform>::User,
        _client: &<Self::Source as Platform>::Client,
    ) -> Result<bool, String> {
        Ok(self.verify())
    }
}
