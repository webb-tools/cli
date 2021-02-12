use subxt::sp_core::sr25519::Pair as Sr25519Pair;
use subxt::sp_core::Pair;
use zeroize::Zeroize;

use crate::error::Error;

pub struct KeyPair {
    pair: Sr25519Pair,
    phrase: Option<String>,
    seed: [u8; 32],
}

impl KeyPair {
    pub fn new(password: &str) -> Self {
        let (pair, phrase, seed) =
            Sr25519Pair::generate_with_phrase(Some(password));
        KeyPair {
            pair,
            phrase: Some(phrase),
            seed,
        }
    }

    pub fn restore(phrase: &str, password: &str) -> Result<Self, Error> {
        let (pair, seed) = Sr25519Pair::from_phrase(phrase, Some(password))
            .map_err(Error::SecretString)?;
        Ok(KeyPair {
            pair,
            phrase: Some(phrase.to_owned()),
            seed,
        })
    }

    pub fn backup(&self) -> Option<String> {
        self.phrase.clone()
    }

    pub fn clean(mut self) {
        self.seed.zeroize();
        self.phrase.zeroize();
        drop(self.pair);
    }

    pub fn pair(&self) -> &Sr25519Pair {
        &self.pair
    }

    pub fn seed(&self) -> [u8; 32] {
        self.seed
    }

    pub fn init(seed: [u8; 32]) -> Self {
        let pair = Sr25519Pair::from_seed(&seed);
        KeyPair {
            pair,
            seed,
            phrase: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use subxt::sp_core::Pair;

    #[test]
    fn create() {
        let keypair = KeyPair::new("super-secret");
        assert_eq!(keypair.seed().len(), 32);
        keypair.clean();
    }

    #[test]
    fn init() {
        let keypair = KeyPair::new("super-secret");
        let keypair2 = KeyPair::init(keypair.seed());
        assert_eq!(keypair.pair().public(), keypair2.pair().public());
        keypair.clean();
        keypair2.clean();
    }
    #[test]
    fn backup_restore() {
        let keypair = KeyPair::new("super-secret");
        let phrase = keypair.backup().unwrap();
        let keypair2 = KeyPair::restore(&phrase, "super-secret").unwrap();
        assert_eq!(keypair.pair().public(), keypair2.pair().public());
        keypair.clean();
        keypair2.clean();
    }
}
