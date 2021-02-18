use std::path::PathBuf;

use anyhow::Result;
use bip39::Mnemonic;
use directories_next::ProjectDirs;
use secrecy::SecretString;
use subxt::sp_core::sr25519::Pair as Sr25519Pair;
use webb_cli::account;
use webb_cli::keystore::PublicFor;

use crate::database::SledDatastore;
use crate::raw::{AccountRaw, AccountsIds, NoteRaw, NotesIds};

/// Commands Execution Context.
///
/// Holds the state needed for all commands.
pub struct ExecutionContext {
    /// All Saved accounts.
    accounts: Vec<AccountRaw>,
    /// All Saved notes.
    notes: Vec<NoteRaw>,
    /// The Safe encrypted datastore.
    db: SledDatastore,
    /// Home of Webb CLI.
    dirs: ProjectDirs,
}

impl ExecutionContext {
    pub fn new(db: SledDatastore, dirs: ProjectDirs) -> Result<Self> {
        let accounts = Self::load_accounts(&db)?;
        let notes = Self::load_notes(&db)?;
        let context = Self {
            accounts,
            notes,
            db,
            dirs,
        };
        Ok(context)
    }

    #[allow(unused)]
    pub fn default_account(&self) -> Option<&AccountRaw> {
        self.accounts.iter().find(|raw| raw.is_default)
    }

    pub fn home(&self) -> PathBuf { self.dirs.data_dir().to_path_buf() }

    pub fn accounts(&self) -> &[AccountRaw] { self.accounts.as_slice() }

    pub fn notes(&self) -> &[NoteRaw] { self.notes.as_slice() }

    pub fn has_secret(&self) -> bool { self.db.has_secret() }

    pub fn set_secret(&mut self, secret: SecretString) {
        self.db.set_secret(secret)
    }

    pub fn set_default_account(
        &mut self,
        alias_or_address: &str,
    ) -> Result<bool> {
        let mut changed = false;
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
            self.db.write_plaintext(acc.uuid.as_bytes(), buf)?;
        }
        Ok(changed)
    }

    pub fn generate_account(
        &mut self,
        alias: String,
    ) -> Result<(PublicFor<Sr25519Pair>, String)> {
        let (account, paper_key) = account::generate(alias);
        let address = account.address;
        let uuid = account.uuid.to_string();
        let mut raw = AccountRaw {
            alias: account.alias,
            address: address.to_string(),
            uuid: account.uuid.to_string(),
            is_default: false,
        };
        // if we don't have any accounts
        if self.accounts.is_empty() {
            // then make this as a default account
            raw.is_default = true;
        }

        let mut buf = Vec::new();
        prost::Message::encode(&raw, &mut buf)?;
        self.db.write_plaintext(uuid.as_bytes(), buf)?;
        let mut seed_key = uuid.clone();
        seed_key.push_str("_seed");
        self.db.write(seed_key.as_bytes(), &account.seed)?;
        // save the account to account ids.
        let maybe_ids = self.db.read_plaintext(b"account_ids")?;
        let v = match maybe_ids {
            Some(b) => {
                let mut v: AccountsIds = prost::Message::decode(b.as_ref())?;
                v.ids.push(uuid);
                v
            },
            None => AccountsIds { ids: vec![uuid] },
        };
        let mut buf = Vec::new();
        prost::Message::encode(&v, &mut buf)?;
        self.db.write_plaintext(b"account_ids", buf)?;
        Ok((address, paper_key))
    }

    pub fn import_account(
        &mut self,
        alias: String,
        paper_key: Mnemonic,
    ) -> Result<PublicFor<Sr25519Pair>> {
        let account = account::restore(alias, paper_key.phrase())?;
        let address = account.address;
        let uuid = account.uuid.to_string();
        let mut raw = AccountRaw {
            alias: account.alias,
            address: address.to_string(),
            uuid: account.uuid.to_string(),
            is_default: false,
        };
        // if we don't have any accounts
        if self.accounts.is_empty() {
            // then make this as a default account
            raw.is_default = true;
        }

        let mut buf = Vec::new();
        prost::Message::encode(&raw, &mut buf)?;
        self.db.write_plaintext(uuid.as_bytes(), buf)?;
        let mut seed_key = uuid.clone();
        seed_key.push_str("_seed");
        self.db.write(seed_key.as_bytes(), &account.seed)?;
        // save the account to account ids.
        let maybe_ids = self.db.read_plaintext(b"account_ids")?;
        let v = match maybe_ids {
            Some(b) => {
                let mut v: AccountsIds = prost::Message::decode(b.as_ref())?;
                v.ids.push(uuid);
                v
            },
            None => AccountsIds { ids: vec![uuid] },
        };
        let mut buf = Vec::new();
        prost::Message::encode(&v, &mut buf)?;
        self.db.write_plaintext(b"account_ids", buf)?;
        Ok(address)
    }

    fn load_accounts(db: &SledDatastore) -> Result<Vec<AccountRaw>> {
        let maybe_ids = db.read_plaintext(b"account_ids")?;
        if let Some(ids) = maybe_ids {
            let AccountsIds { ids } = prost::Message::decode(ids.as_ref())?;
            let mut result = Vec::new();
            for id in ids {
                let maybe_metadata = db.read_plaintext(id.as_bytes())?;
                let account: AccountRaw = match maybe_metadata {
                    Some(m) => prost::Message::decode(m.as_ref())?,
                    None => continue,
                };
                result.push(account);
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    fn load_notes(db: &SledDatastore) -> Result<Vec<NoteRaw>> {
        let maybe_ids = db.read_plaintext(b"notes_ids")?;
        if let Some(ids) = maybe_ids {
            let NotesIds { ids } = prost::Message::decode(ids.as_ref())?;
            let mut result = Vec::new();
            for id in ids {
                let maybe_metadata = db.read_plaintext(id.as_bytes())?;
                let note: NoteRaw = match maybe_metadata {
                    Some(m) => prost::Message::decode(m.as_ref())?,
                    None => continue,
                };
                result.push(note);
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
}
