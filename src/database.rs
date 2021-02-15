use anyhow::Context;
use chacha::{aead::Aead, aead::NewAead, Key, XChaCha20Poly1305, XNonce};
use directories_next::ProjectDirs;
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString, Zeroize};

use crate::utils;

pub struct SledDatastore {
    sled: sled::Db,
    secret: Option<SecretString>,
}

impl SledDatastore {
    pub fn new() -> anyhow::Result<Self> {
        let dirs = ProjectDirs::from(
            crate::PACKAGE_ID[0],
            crate::PACKAGE_ID[1],
            crate::PACKAGE_ID[2],
        )
        .context("getting project data")?;

        let db_path = dirs.data_dir().join("db");
        let db = sled::open(db_path).context("open database")?;
        Ok(Self {
            secret: None,
            sled: db,
        })
    }

    pub fn with_secret(secret: SecretString) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut this = Self::new()?;
        this.secret = Some(secret);
        Ok(this)
    }

    pub fn read(
        &self,
        key: impl Into<sled::IVec>,
    ) -> anyhow::Result<Option<sled::IVec>> {
        let secret =
            self.secret.clone().expect("password must be provided here");
        // hash the password to get a 32 bytes for the secret key.
        // I guess this fine?
        let mut deckey_bytes = utils::sha256(secret.expose_secret().as_bytes());
        let encrypted = self.sled.get(key.into())?;
        if let Some(data) = encrypted {
            let nonce_bytes = &data[0..24]; // 24 bytes are the nonce.
            let conents = &data[24..]; // the rest is the encrypted data.
            let deckey = Key::from_slice(&deckey_bytes);
            let nonce = XNonce::from_slice(nonce_bytes);
            let aead = XChaCha20Poly1305::new(deckey);
            let plaintext =
                aead.decrypt(nonce, conents).expect("decryption failure!");
            deckey_bytes.zeroize();
            Ok(Some(plaintext.into()))
        } else {
            Ok(None)
        }
    }

    pub fn write(
        &self,
        key: impl Into<sled::IVec>,
        value: impl Into<sled::IVec>,
    ) -> anyhow::Result<Option<sled::IVec>> {
        let secret =
            self.secret.clone().expect("password must be provided here");
        let mut enckey_bytes = utils::sha256(secret.expose_secret().as_bytes());
        let mut buffer = Vec::new(); // a buffer to hold the nonce + encrypted bytes.
        let mut nonce_bytes = [0u8; 24];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);
        let enckey = Key::from_slice(&enckey_bytes);
        let aead = XChaCha20Poly1305::new(enckey);

        let mut encrypted = aead
            .encrypt(&nonce, value.into().as_ref())
            .expect("encryption failure");
        buffer.extend_from_slice(&nonce_bytes); // add nonce. [0..24]
        buffer.append(&mut encrypted); // add encrypted bytes [24..]
        enckey_bytes.zeroize(); // clear the key.
        self.sled
            .insert(key.into(), buffer)
            .map_err(anyhow::Error::from)
    }

    pub fn read_plaintext(
        &self,
        key: impl Into<sled::IVec>,
    ) -> anyhow::Result<Option<sled::IVec>> {
        self.sled.get(key.into()).map_err(anyhow::Error::from)
    }

    pub fn write_plaintext(
        &self,
        key: impl Into<sled::IVec>,
        value: impl Into<sled::IVec>,
    ) -> anyhow::Result<Option<sled::IVec>> {
        self.sled
            .insert(key.into(), value.into())
            .map_err(anyhow::Error::from)
    }

    pub fn has_secret(&self) -> bool {
        self.secret.is_some()
    }

    pub fn set_secret(&mut self, secret: SecretString) {
        self.secret = Some(secret);
    }
}
