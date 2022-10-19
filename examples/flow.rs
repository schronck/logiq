use async_trait::async_trait;
use ethers::prelude::{Address, LocalWallet, Signature as EthSignature};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature as SolSignature;
use solana_sdk::signer::{keypair::Keypair, Signer};

use reqwest::Client;

pub type Balance = u128;

trait Platform: Sync {
    type User: Identity + Sync;
    fn endpoint(&self) -> &str;
}

#[async_trait]
trait Requirement {
    type Source: Platform;
    type Extrinsic: Sized + Sync + Send;
    async fn retrieve_extrinsic(
        &self,
        source: &Self::Source,
        user: &<Self::Source as Platform>::User,
        client: &Client,
    ) -> Result<Self::Extrinsic, String>;

    async fn check_identity(&self, user: &<Self::Source as Platform>::User) -> bool;

    async fn check_extrinsic(&self, extrinsic: &Self::Extrinsic) -> bool;

    async fn check(
        &self,
        source: &Self::Source,
        user: &<Self::Source as Platform>::User,
        client: &Client,
    ) -> Result<bool, String> {
        let extrinsic = self.retrieve_extrinsic(source, user, client).await?;
        Ok(self.check_identity(user).await && self.check_extrinsic(&extrinsic).await)
    }
}

trait Identity {
    type Identifier: Sized + PartialEq + Sync;
    fn verify(&self) -> bool;
    fn identifier(&self) -> &Self::Identifier;
}

struct EthereumSignature {
    address: Address,
    signature: EthSignature,
    msg: String,
}

struct SolanaSignature {
    pubkey: Pubkey,
    signature: SolSignature,
    msg: String,
}

impl Identity for EthereumSignature {
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

impl Identity for SolanaSignature {
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
    Other {
        metadata: [u8; 512], // e.g. ERC-1155
    },
}

type BalanceType<T> = Option<TokenType<T>>;

enum Relation {
    Equal,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
}

struct RequiredBalance<T: Platform> {
    balance_type: BalanceType<<T as Platform>::User>,
    relation: Relation,
    amount: Balance,
}

struct Allowlist<T: Platform>(Vec<<<T as Platform>::User as Identity>::Identifier>);

#[async_trait]
impl<T: Platform + Sync> Requirement for Allowlist<T> {
    type Source = T;
    type Extrinsic = bool;

    async fn retrieve_extrinsic(
        &self,
        _source: &Self::Source,
        _user: &<Self::Source as Platform>::User,
        _client: &Client,
    ) -> Result<Self::Extrinsic, String> {
        Ok(true)
    }

    async fn check_identity(&self, user: &<Self::Source as Platform>::User) -> bool {
        user.verify() && self.0.iter().any(|id| id == user.identifier())
    }

    async fn check_extrinsic(&self, extrinsic: &Self::Extrinsic) -> bool {
        *extrinsic
    }
}

#[async_trait]
impl<T: Platform + Sync> Requirement for RequiredBalance<T> {
    type Source = T;
    type Extrinsic = Balance; // type alias for u128

    async fn retrieve_extrinsic(
        &self,
        source: &Self::Source,
        user: &<Self::Source as Platform>::User,
        client: &Client,
    ) -> Result<Self::Extrinsic, String> {
        todo!()
    }

    async fn check_extrinsic(&self, extrinsic: &Self::Extrinsic) -> bool {
        match self.relation {
            Relation::Equal => *extrinsic == self.amount,
            Relation::Greater => *extrinsic > self.amount,
            Relation::GreaterOrEqual => *extrinsic >= self.amount,
            Relation::Less => *extrinsic < self.amount,
            Relation::LessOrEqual => *extrinsic <= self.amount,
        }
    }

    async fn check_identity(&self, user: &<Self::Source as Platform>::User) -> bool {
        user.verify()
    }
}

fn main() {}
