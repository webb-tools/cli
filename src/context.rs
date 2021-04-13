use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use bip39::Mnemonic;
use directories_next::ProjectDirs;
use jsonrpsee_ws_client::{WsClient, WsConfig};
use secrecy::SecretString;
use subxt::sp_core::sr25519::Pair as Sr25519Pair;
use subxt::sp_core::Pair;
use subxt::{Client, PairSigner, RpcClient};
use webb_cli::account;
use webb_cli::keystore::PublicFor;
use webb_cli::mixer::{Mixer, Note, TokenSymbol};
use webb_cli::runtime::WebbRuntime;

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
    /// RPC Endpoint.
    rpc_url: url::Url,
}

impl ExecutionContext {
    pub fn new(
        db: SledDatastore,
        dirs: ProjectDirs,
        rpc_url: url::Url,
    ) -> Result<Self> {
        let accounts = Self::load_accounts(&db)?;
        let notes = Self::load_notes(&db)?;
        let context = Self {
            accounts,
            notes,
            db,
            dirs,
            rpc_url,
        };
        Ok(context)
    }

    pub fn default_account(&self) -> Result<&AccountRaw> {
        self.accounts
            .iter()
            .find(|raw| raw.is_default)
            .context("must have a default account")
    }

    pub fn signer(&self) -> Result<PairSigner<WebbRuntime, Sr25519Pair>> {
        let default_account = self.default_account()?;
        let mut seed_key = default_account.uuid.clone();
        seed_key.push_str("_seed");
        let seed = self
            .db
            .read(seed_key.as_bytes())?
            .context("signer encrypted seed")?;
        let pair = Sr25519Pair::from_seed_slice(&seed).map_err(|_| {
            anyhow::anyhow!("failed to create keypair from seed")
        })?;
        let signer = PairSigner::new(pair);
        Ok(signer)
    }

    pub fn home(&self) -> PathBuf { self.dirs.data_dir().to_path_buf() }

    pub fn accounts(&self) -> &[AccountRaw] { self.accounts.as_slice() }

    pub fn notes(&self) -> &[NoteRaw] { self.notes.as_slice() }

    pub async fn client(&self) -> Result<Client<WebbRuntime>> {
        let client = subxt::ClientBuilder::new()
            .set_url(self.rpc_url.as_str())
            .build()
            .await?;
        Ok(client)
    }

    pub async fn rpc_client(&self) -> Result<RpcClient> {
        let mut config = WsConfig::with_url(self.rpc_url.as_str());
        config.max_notifs_per_subscription = 4096;
        Ok(RpcClient::WebSocket(Arc::new(WsClient::new(config).await?)))
    }

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

    pub fn generate_note(
        &mut self,
        alias: String,
        mixer_id: u32,
        token_symbol: TokenSymbol,
    ) -> Result<()> {
        let mut mixer = Mixer::new(mixer_id);
        let note = mixer.generate_note(token_symbol);
        self.import_note(alias, note)?;
        Ok(())
    }

    pub fn import_note(&mut self, alias: String, note: Note) -> Result<u32> {
        let uuid = uuid::Uuid::new_v4();
        let raw = NoteRaw {
            alias,
            mixer_id: note.mixer_id,
            token_symbol: note.token_symbol.to_string(),
            uuid: uuid.to_string(),
            used: false,
        };
        let mut buf = Vec::new();
        prost::Message::encode(&raw, &mut buf)?;
        self.db.write_plaintext(uuid.to_string().as_bytes(), buf)?;
        let mut secret_key = uuid.to_string();
        secret_key.push_str("_secret");
        let note_secret = note.to_string().into_bytes();
        self.db.write(secret_key.as_bytes(), note_secret)?;
        let maybe_ids = self.db.read_plaintext(b"notes_ids")?;
        let v = match maybe_ids {
            Some(b) => {
                let mut v: NotesIds = prost::Message::decode(b.as_ref())?;
                v.ids.push(uuid.to_string());
                v
            },
            None => NotesIds {
                ids: vec![uuid.to_string()],
            },
        };
        let mut buf = Vec::new();
        prost::Message::encode(&v, &mut buf)?;
        self.db.write_plaintext(b"notes_ids", buf)?;
        Ok(raw.mixer_id)
    }

    pub fn decrypt_note(&self, uuid: String) -> Result<Note> {
        let mut key = uuid;
        key.push_str("_secret");
        let buf = self
            .db
            .read(key.as_bytes())?
            .context("finding the encrypted note")?;
        let note_str = String::from_utf8(buf.to_vec())?;
        let note = note_str.parse()?;
        Ok(note)
    }

    pub fn mark_note_as_used(&mut self, uuid: String) -> Result<()> {
        let metadata = self
            .db
            .read_plaintext(uuid.as_bytes())?
            .context("reading note metadata")?;
        let mut note: NoteRaw = prost::Message::decode(metadata.as_ref())?;
        note.used = true;

        let mut buf = Vec::new();
        prost::Message::encode(&note, &mut buf)?;
        self.db.write_plaintext(uuid.as_bytes(), buf)?;
        Ok(())
    }

    pub fn forget_note(&self, uuid: String) -> Result<()> {
        self.db.remove(uuid.as_bytes())?;
        let mut key = uuid;
        key.push_str("_secret");
        self.db.remove(key.as_bytes())?;
        Ok(())
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

#[derive(Debug, Clone)]
pub struct SystemProperties {
    /// The address format
    pub ss58_format: u8,
    /// The number of digits after the decimal point in the native token
    pub token_decimals: u8,
    /// The symbol of the native token
    pub token_symbol: String,
}

impl Default for SystemProperties {
    fn default() -> Self {
        Self {
            ss58_format: 100,
            token_decimals: 12,
            token_symbol: String::from("Unit"),
        }
    }
}

impl<'a> From<&'a subxt::SystemProperties> for SystemProperties {
    fn from(v: &'a subxt::SystemProperties) -> Self {
        if subxt::SystemProperties::default().eq(v) {
            Self::default()
        } else {
            Self {
                ss58_format: v.ss58_format,
                token_decimals: v.token_decimals,
                token_symbol: v.token_symbol.clone(),
            }
        }
    }
}
