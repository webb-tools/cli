use bip39::Mnemonic;
use std::path::Path;

use crate::raw::AccountRaw;
use anyhow::Result;
use directories_next::ProjectDirs;
use secrecy::{ExposeSecret, SecretString};
use subxt::sp_core::sr25519::Pair as Sr25519Pair;
use webb_cli::account;
use webb_cli::keystore::PublicFor;

/// Commands Execution Context.
///
/// Holds the state needed for all commands.
pub struct ExecutionContext {
    /// All Saved accounts.
    accounts: Vec<AccountRaw>,
    /// The Main Database.
    db: sled::Db,
    /// Home of Webb CLI.
    dirs: ProjectDirs,
}

impl ExecutionContext {
    pub fn new(db: sled::Db, dirs: ProjectDirs) -> Result<Self> {
        let accounts = Self::load_accounts(&db)?;
        let context = Self { accounts, db, dirs };
        Ok(context)
    }

    #[allow(unused)]
    pub fn default_account(&self) -> Option<&AccountRaw> {
        self.accounts.iter().find(|raw| raw.is_default)
    }

    pub fn home(&self) -> &Path {
        self.dirs.data_dir()
    }

    pub fn accounts(&self) -> &[AccountRaw] {
        self.accounts.as_slice()
    }

    pub fn set_default_account(
        &mut self,
        alias_or_address: &str,
    ) -> Result<bool> {
        let mut changed = false;
        let tree = self.db.open_tree("accounts")?;
        // let's loop over all the accounts
        for acc in &mut self.accounts {
            // first we set any account as not default.
            acc.is_default = false;
            let alias_match = acc.alias == alias_or_address;
            let address_match = acc.address == alias_or_address;
            let matched = alias_match || address_match;
            // we found it!
            if matched && !changed {
                // set it to default account
                acc.is_default = true;
                // and mark it as changed.
                changed = true;
            }
            // save any changes to the database.
            let mut buf = Vec::new();
            prost::Message::encode(acc, &mut buf)?;
            tree.insert(acc.uuid.as_bytes(), buf)?;
        }
        Ok(changed)
    }

    pub fn generate_account(
        &mut self,
        alias: String,
        password: SecretString,
    ) -> Result<(PublicFor<Sr25519Pair>, String)> {
        let (account, paper_key) = account::generate(alias);
        let address = account.address;
        let uuid = account.uuid.to_string();
        let mut raw = AccountRaw {
            alias: account.alias,
            address: address.to_string(),
            uuid: account.uuid.to_string(),
            seed: account.seed.to_vec(),
            is_default: false,
        };
        // if we don't have any accounts
        if self.accounts.is_empty() {
            // then make this as a default account
            raw.is_default = true;
        }

        let tree = self.db.open_tree("accounts")?;
        let mut buf = Vec::new();
        prost::Message::encode(&raw, &mut buf)?;
        tree.insert(uuid.as_bytes(), buf)?;
        Ok((address, paper_key))
    }

    pub fn import_account(
        &mut self,
        alias: String,
        password: SecretString,
        paper_key: Mnemonic,
    ) -> Result<PublicFor<Sr25519Pair>> {
        let account = account::restore(alias, paper_key.phrase())?;
        let address = account.address;
        let uuid = account.uuid.to_string();
        let mut raw = AccountRaw {
            alias: account.alias,
            address: address.to_string(),
            uuid: account.uuid.to_string(),
            seed: account.seed.to_vec(),
            is_default: false,
        };
        // if we don't have any accounts
        if self.accounts.is_empty() {
            // then make this as a default account
            raw.is_default = true;
        }

        let tree = self.db.open_tree("accounts")?;
        let mut buf = Vec::new();
        prost::Message::encode(&raw, &mut buf)?;
        tree.insert(uuid.as_bytes(), buf)?;
        Ok(address)
    }

    fn load_accounts(db: &sled::Db) -> Result<Vec<AccountRaw>> {
        let tree = db.open_tree("accounts")?;
        let kvs: sled::Result<Vec<_>> = tree.iter().collect();
        let mut result = Vec::new();
        for (_, v) in kvs? {
            let raw: AccountRaw = prost::Message::decode(&*v)?;
            result.push(raw);
        }
        Ok(result)
    }
}
