use async_trait::async_trait;
use ethers::prelude::{Address, LocalWallet, Signature as EthSignature};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature as SolSignature;
use solana_sdk::signer::{keypair::Keypair, Signer};

pub type Balance = u128;

trait Platform {
    type User: Identity;
    fn endpoint(&self) -> &str;
}

#[async_trait]
trait Requirement {
    type Querier;
    type Platform: Platform;
    type VerificationData: Sized;
    async fn verification_data(
        &self,
        platform: &Self::Platform,
        querier: &Self::Querier,
    ) -> Result<Self::VerificationData, String>
    where
        Self: Sized;

    async fn check(
        &self,
        user: &<<Self as Requirement>::Platform as Trait>::User,
        data: &Self::VerificationData,
    ) -> Result<bool, String>;
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

struct RequiredBalance<T: Identity> {
    balance_type: BalanceType<T>,
    relation: Relation,
    amount: Balance,
}

struct Allowlist<T: Identity>(Vec<T::Identifier>);

#[async_trait]
impl<T: Platform> Requirement for Allowlist<T> {
    type Querier = reqwest::Client;
    type Platform = T;
    type VerificationData = ();

    async fn verification_data(
        &self,
        platform: &Self::Platform,
        querier: &Self::Querier,
    ) -> Result<Self::VerificationData, String> {
        Ok(())
    }

    async fn check(
        &self,
        user: &Self::Platform::User,
        data: &Self::VerificationData,
    ) -> Result<bool, String> {
        if !user.verify() {
            return Err("invalid signature".to_string());
        }
        Ok(self.0.iter().any(|id| id == user.identifier()))
    }
}

/*
#[async_trait]
impl<T: Identity> Requirement for RequiredBalance<T> {
    type Querier = reqwest::Client;
    type User = T;
    type VerificationData = Balance; // type alias for u128

    async fn verification_data(
        &self,
        user: &Self::User,
        querier: &Self::Querier,
    ) -> Result<Self::VerificationData, String> {
    }
}
*/

fn main() {}
