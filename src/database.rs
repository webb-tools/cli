use anyhow::Context;
use chacha::{
    aead::{Aead, NewAead},
    Key, XChaCha20Poly1305, XNonce,
};
use directories_next::ProjectDirs;
use rand::RngCore;
use secrecy::{SecretString, Zeroize};

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
        let secret = self
            .secret
            .clone()
            .context("password must be provided for decryption!")?;
        let mut deckey_hash = utils::sha256(secret);
        let encrypted = self.sled.get(key.into())?;
        if let Some(data) = encrypted {
            let nonce_bytes = &data[0..24]; // 24 bytes are the nonce.
            let contents = &data[24..]; // the rest is the encrypted data.
            let deckey = Key::from_slice(&deckey_hash);
            let nonce = XNonce::from_slice(nonce_bytes);
            let aead = XChaCha20Poly1305::new(deckey);
            let plaintext = aead
                .decrypt(nonce, contents)
                .map_err(|_| anyhow::anyhow!("datastore decrypt failed"))
                .context("data decryption!")?;
            deckey_hash.zeroize();
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
        let secret = self
            .secret
            .clone()
            .context("password must be provided for encryption")?;
        let mut enckey_hash = utils::sha256(secret);
        let mut buffer = Vec::new(); // a buffer to hold the nonce + encrypted bytes.
        let mut nonce_bytes = [0u8; 24];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);
        let enckey = Key::from_slice(&enckey_hash);
        let aead = XChaCha20Poly1305::new(enckey);
        let mut encrypted = aead
            .encrypt(nonce, value.into().as_ref())
            .map_err(|_| anyhow::anyhow!("datastore encryption failed"))
            .context("data encryption")?;
        buffer.extend(&nonce_bytes); // add nonce. [0..24]
        buffer.append(&mut encrypted); // add encrypted bytes [24..]
        enckey_hash.zeroize(); // clear the key.
        let val = self
            .sled
            .insert(key.into(), buffer)
            .map_err(anyhow::Error::from)?;
        self.sled.flush()?;
        Ok(val)
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
        let val = self
            .sled
            .insert(key.into(), value.into())
            .map_err(anyhow::Error::from)?;
        self.sled.flush()?;
        Ok(val)
    }

    pub fn has_secret(&self) -> bool { self.secret.is_some() }

    pub fn set_secret(&mut self, secret: SecretString) {
        self.secret = Some(secret);
    }

    pub fn remove(
        &self,
        key: impl Into<sled::IVec>,
    ) -> anyhow::Result<Option<sled::IVec>> {
        self.sled.remove(key.into()).map_err(anyhow::Error::from)
    }
}
