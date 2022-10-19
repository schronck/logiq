use async_trait::async_trait;
use ethers::prelude::{Address, LocalWallet, Signature as EthSignature};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature as SolSignature;
use solana_sdk::signer::{keypair::Keypair, Signer};

pub type Balance = u128;

trait Platform {
    type User: Identity + Sync;
    fn endpoint(&self) -> &str;
}

trait Requirement {
    type Source: Platform;
    type VerificationData: Sized;
    fn build_request(
        &self,
        source: &Self::Source,
        querier: &reqwest::Client,
    ) -> Option<request::Request> {
        None
    }

    fn check(
        &self,
        user: &<Self::Source as Platform>::User,
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

struct Allowlist<T: Platform>(Vec<<<T as Platform>::User as Identity>::Identifier>);

impl<T: Platform + Sync> Requirement for Allowlist<T> {
    type Source = T;
    type VerificationData = Option<()>;

    fn check(
        &self,
        user: &<Self::Source as Platform>::User,
        _data: &OSelf::VerificationData,
    ) -> Result<bool, String> {
        if !user.verify() {
            Ok(false)
        } else {
            Ok(self.0.iter().any(|id| id == user.identity()))
        }
    }
}

impl<T: Platform + Sync> Requirement for RequiredBalance<T> {
    type Source = T;
    type VerificationData = Balance; // type alias for u128

    fn build_request(
        &self,
        user: &Self::User,
        client: &reqwest::Client,
    ) -> Option<reqwest::Request> {
    }

    fn check(&self, user: &<Self::Source as Platform>::User, data: &Self::VerificationData) {}
}

fn main() {}
