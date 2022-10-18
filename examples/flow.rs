use async_trait::async_trait;
use ethers::prelude::{Address, LocalWallet, Signature as EthSignature};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature as SolSignature;
use solana_sdk::signer::{keypair::Keypair, Signer};

pub type Balance = u128;

#[async_trait]
trait Requirement {
    type Querier;
    type User: Identity;
    type VerificationData: Sized;
    async fn verification_data(
        &self,
        user: &Self::User,
        querier: &Self::Querier,
    ) -> Result<VerificationData, String>
    where
        Self: Sized;

    async fn check(&self, user: &Self::User, data: &Self::VerificationData)
        -> Result<bool, String>;
}

trait Identity {
    type Identifier: Sized;
    fn verify(&self) -> Result<bool, String>;
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
    fn verify(&self) -> Result<bool, String> {
        // TODO check msg is something that we expect
        self.signature
            .verify(self.msg.as_str(), self.address)
            .map_err(|e| e.to_string())
    }

    fn identifier(&self) -> &Self::Identifier {
        &self.address
    }
}

impl Identity for SolanaSignature {
    type Identifier = Pubkey;
    fn verify(&self) -> Result<bool, String> {
        // TODO check msg is something that we expect
        Ok(self
            .signature
            .verify(self.pubkey.as_bytes(), self.msg.as_bytes()))
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

struct RequiredBalance {
    balance_type: BalanceType,
    relation: Relation,
    amount: Balance,
}

struct Allowlist<T>(Vec<T>);

impl<T: Idenity> Requirement for Allowlist<T> {}

fn main() {}
