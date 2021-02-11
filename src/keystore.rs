use bip39::{Language, Mnemonic, MnemonicType};
use substrate_subxt::sp_core::sr25519::Pair;
use zeroize::Zeroize;

use crate::error::Error;

pub struct KeyPair {
    pair: Pair,
    entropy: Vec<u8>,
}

impl KeyPair {
    pub fn new(password: &str) -> Self {
        let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
        let entropy = mnemonic.entropy();
        let (pair, _) = Pair::from_entropy(entropy, Some(password));
        KeyPair {
            pair,
            entropy: entropy.to_vec(),
        }
    }

    pub fn init(entropy: &[u8], password: &str) -> Self {
        let (pair, _) = Pair::from_entropy(entropy, Some(password));
        KeyPair {
            pair,
            entropy: entropy.to_vec(),
        }
    }

    pub fn restore(phrase: &str, password: &str) -> Result<Self, Error> {
        let mnemonic = Mnemonic::from_phrase(phrase, Language::English)
            .map_err(|e| Error::Mnemonic(e.to_string()))?;
        let entropy = mnemonic.entropy();
        let (pair, _) = Pair::from_entropy(entropy, Some(password));
        Ok(KeyPair {
            pair,
            entropy: entropy.to_vec(),
        })
    }

    pub fn backup(&self) -> String {
        let mnemonic = Mnemonic::from_entropy(&self.entropy, Language::English)
            .expect("entropy should be valid!");
        mnemonic.into_phrase()
    }

    pub fn clean(mut self) {
        self.entropy.zeroize();
        drop(self.pair);
    }

    pub fn pair(&self) -> &Pair {
        &self.pair
    }

    pub fn entropy(&self) -> &[u8] {
        self.entropy.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use substrate_subxt::sp_core::Pair;

    #[test]
    fn create() {
        let keypair = KeyPair::new("super-secret");
        assert_eq!(keypair.entropy().len(), 16);
        keypair.clean();
    }

    #[test]
    fn init() {
        let keypair = KeyPair::new("super-secret");
        let keypair2 = KeyPair::init(keypair.entropy(), "super-secret");
        assert_eq!(keypair.pair().public(), keypair2.pair().public());
        keypair.clean();
        keypair2.clean();
    }
    #[test]
    fn backup_restore() {
        let keypair = KeyPair::new("super-secret");
        let phrase = keypair.backup();
        let keypair2 = KeyPair::restore(&phrase, "super-secret").unwrap();
        assert_eq!(keypair.pair().public(), keypair2.pair().public());
        keypair.clean();
        keypair2.clean();
    }
}
