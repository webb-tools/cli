use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use bip39::Mnemonic;
use directories_next::ProjectDirs;
use secrecy::{SecretString, Zeroize};
use subxt::{
    sp_core::{sr25519::Pair as Sr25519Pair, Pair},
    PairSigner,
};
use webb::substrate::{
    protocol_substrate_runtime::api::{
        runtime_types::{
            frame_support::storage::bounded_vec::BoundedVec,
            pallet_asset_registry::types::AssetDetails,
            pallet_mixer::types::MixerMetadata,
        },
        RuntimeApi,
    },
    subxt,
};
use webb_cli::{account, keystore::PublicFor, mixer, note};

use crate::{
    database::SledDatastore,
    raw::{AccountRaw, AccountsIds, NoteRaw, NotesIds},
};

type WebbRuntimeApi =
    RuntimeApi<subxt::DefaultConfig, subxt::DefaultExtra<subxt::DefaultConfig>>;
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

    pub fn signer(
        &self,
    ) -> Result<
        PairSigner<
            subxt::DefaultConfig,
            subxt::DefaultExtra<subxt::DefaultConfig>,
            Sr25519Pair,
        >,
    > {
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

    pub async fn client(&self) -> Result<WebbRuntimeApi> {
        let client = subxt::ClientBuilder::new()
            .set_url(self.rpc_url.as_str())
            .build()
            .await?;
        Ok(client.to_runtime_api())
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
        asset: AssetDetails<u32, u128, BoundedVec<u8>>,
        mixer: MixerMetadata<u128, u32>,
        denomination: u8,
        chain_id: u32,
    ) -> Result<()> {
        let curve = note::Curve::Bn254;
        let exponentiation = 5;
        let width = 5;
        let rng = &mut rand::thread_rng();
        let asset_name = String::from_utf8_lossy(&asset.name.0).to_string();
        let secret =
            mixer::generate_secrets(curve, exponentiation, width, rng)?;
        let v = note::Note::builder()
            .prefix(note::NotePrefix::Mixer)
            .version(note::NoteVersion::V1)
            .target_chain_id(chain_id)
            .source_chain_id(chain_id)
            .backend(note::Backend::Circom)
            .hash_function(note::HashFunction::Poseidon)
            .curve(curve)
            .exponentiation(exponentiation)
            .width(width)
            .token_symbol(asset_name)
            .amount(mixer.deposit_size.to_string())
            .denomination(denomination)
            .secret(secret)
            .build();
        self.import_note(alias, v)?;
        Ok(())
    }

    pub fn import_note(
        &mut self,
        alias: String,
        mut note: note::Note,
    ) -> Result<String> {
        let uuid = uuid::Uuid::new_v4();
        let secret = zeroize::Zeroizing::new(note.secret);
        // zeroize the secret in the note.
        note.secret.zeroize();
        let raw = NoteRaw {
            alias,
            value: note.to_string(),
            uuid: uuid.to_string(),
            used: false,
        };
        let mut buf = Vec::new();
        prost::Message::encode(&raw, &mut buf)?;
        self.db.write_plaintext(uuid.to_string().as_bytes(), buf)?;
        let mut secret_key = uuid.to_string();
        secret_key.push_str("_secret");
        self.db.write(secret_key.as_bytes(), &secret[..])?;
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
        Ok(raw.uuid)
    }

    pub fn decrypt_note(&self, uuid: String) -> Result<note::Note> {
        let mut key = uuid;
        let raw = self
            .db
            .read_plaintext(key.as_bytes())?
            .context("note not found")?;
        let note: NoteRaw = prost::Message::decode(raw.as_ref())?;
        key.push_str("_secret");
        let secret = self
            .db
            .read(key.as_bytes())?
            .context("finding the encrypted note")?
            .to_vec()
            .try_into()
            .map_err(|_| {
                anyhow::anyhow!("invalid secret bytes (not 64 bytes)")
            })?;
        let mut note = note::Note::from_str(&note.value)?;
        note.secret = secret;
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
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
            ss58_format: 42,
            token_decimals: 12,
            token_symbol: String::from("Unit"),
        }
    }
}

impl From<subxt::SystemProperties> for SystemProperties {
    fn from(v: subxt::SystemProperties) -> Self {
        let json = serde_json::Value::Object(v);
        match serde_json::from_value(json) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Failed to deserialize SystemProperties: {}", e);
                Self::default()
            },
        }
    }
}
