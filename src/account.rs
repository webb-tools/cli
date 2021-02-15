use core::fmt;

use subxt::sp_core::sr25519::Pair as Sr25519Pair;
use subxt::sp_core::Pair;
use subxt::PairSigner;
use uuid::Uuid;

use crate::error::Error;
use crate::keystore::{KeyPair, PublicFor};
use crate::runtime::WebbRuntime;

pub struct Account {
    pub uuid: Uuid,
    pub alias: String,
    pub address: PublicFor<Sr25519Pair>,
    pub signer: PairSigner<WebbRuntime, Sr25519Pair>,
    pub seed: [u8; 32],
}

impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Account")
            .field("uuid", &self.uuid)
            .field("alias", &self.alias)
            .field("address", &self.address)
            .field("signer", &"[....]")
            .field("entropy", &"[....]")
            .finish()
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.alias, self.address)
    }
}

impl Account {
    pub fn init(uuid: Uuid, alias: String, seed: [u8; 32]) -> Self {
        let keys = KeyPair::init(seed);
        let account = Self {
            uuid,
            alias,
            seed,
            address: keys.pair().public(),
            signer: PairSigner::new(keys.pair().clone()),
        };
        keys.clean();
        account
    }
}

/// Generates new `KeyPair` and returns new [Account] with Paper backup phrase.
pub fn generate(alias: String) -> (Account, String) {
    let keys = KeyPair::new(None);
    let account = Account {
        alias,
        uuid: Uuid::new_v4(),
        address: keys.pair().public(),
        signer: PairSigner::new(keys.pair().clone()),
        seed: keys.seed(),
    };
    let paper_key =
        keys.backup().expect("new generated accound have paper key");
    keys.clean();
    (account, paper_key)
}

/// Restores the [Account] using the Paper backup phrase.
pub fn restore(alias: String, paper_key: &str) -> Result<Account, Error> {
    let keys = KeyPair::restore(paper_key, None)?;
    let account = Account {
        alias,
        uuid: Uuid::new_v4(),
        address: keys.pair().public(),
        signer: PairSigner::new(keys.pair().clone()),
        seed: keys.seed(),
    };
    keys.clean();
    Ok(account)
}
