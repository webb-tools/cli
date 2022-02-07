use std::{collections::HashMap, io::Write, str::FromStr};

use anyhow::Context;
use async_trait::async_trait;
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use secrecy::SecretString;
use structopt::StructOpt;
use subxt::Signer;
use webb::substrate::subxt::{self, TransactionStatus};
use webb_cli::{mixer, note::Note};

use crate::{
    context::{ExecutionContext, SystemProperties},
    ext::OptionPromptExt,
};

/// Webb Crypto Mixer.
#[derive(StructOpt)]
pub enum MixerCommand {
    /// List all of your saved Notes.
    ListNotes,
    /// Imports a previously generated Note.
    ImportNote(ImportNote),
    /// Generates a new Note and save it.
    GenerateNote(GenerateNote),
    /// Remove/Forget a Note.
    ForgetNote(ForgetNote),
    /// Deposit crypto assets to the mixer.
    Deposit(DepositAsset),
    /// Withdraw a previously deposited asset from the mixer.
    Withdraw(WithdrawAsset),
}

#[async_trait]
impl super::CommandExec for MixerCommand {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        match self {
            MixerCommand::ListNotes => {
                let mut term = console::Term::stdout();
                let mut notes = context.notes().to_owned();
                if notes.is_empty() {
                    writeln!(term)?;
                    writeln!(term, "there is no Notes saved")?;
                    writeln!(term, "try generating or importing them.")?;
                    writeln!(term)?;
                    writeln!(term, "$ webb mixer help")?;
                    return Ok(());
                }
                // put the unused account first.
                notes.sort_by(|a, b| b.used.cmp(&a.used));

                for note in notes {
                    writeln!(term, "{}", note)?;
                }
                Ok(())
            },
            MixerCommand::ImportNote(cmd) => cmd.exec(context).await,
            MixerCommand::GenerateNote(cmd) => cmd.exec(context).await,
            MixerCommand::ForgetNote(cmd) => cmd.exec(context).await,
            MixerCommand::Deposit(cmd) => cmd.exec(context).await,
            MixerCommand::Withdraw(cmd) => cmd.exec(context).await,
        }
    }
}

/// Import a previously generated Note to your local secure store.
///
/// The Note could be generated previously from the Webb UI.
#[derive(StructOpt)]
pub struct ImportNote {
    /// an easy to remember the Note.
    #[structopt(short, long)]
    alias: Option<String>,
    /// Note string.
    #[structopt(env = "WEBB_NOTE")]
    note: Option<String>,
}

#[async_trait]
impl super::CommandExec for ImportNote {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        let mut term = console::Term::stdout();
        let theme = dialoguer::theme::ColorfulTheme::default();
        let alias = self.alias.unwrap_or_prompt("Note Alias", &theme)?;
        let note = if let Some(val) = self.note {
            Note::from_str(&val)?
        } else {
            loop {
                let v = Option::<Note>::None.unwrap_or_prompt("Note", &theme);
                match v {
                    Ok(note) => break note,
                    Err(e) => {
                        writeln!(term, "{}", style(e).red())?;
                        continue;
                    },
                }
            }
        };
        if !context.has_secret() {
            let password = Option::<SecretString>::None
                .unwrap_or_prompt_password(
                    "Default Account Password",
                    &theme,
                )?;
            context.set_secret(password);
        }
        // to make sure that the password is correct.
        context
            .signer()
            .context("incorrect default account password!")?;
        let mixer_group_id = context.import_note(alias.clone(), note)?;
        writeln!(
            term,
            "Note Imported with alias {} for #{} Mixer Group",
            style(alias).green(),
            mixer_group_id
        )?;
        writeln!(term)?;
        writeln!(term, "Next, Do a dopist using this note.")?;
        writeln!(term, "    $ webb mixer deposit")?;
        Ok(())
    }
}

/// Generate a new Note and save it for later.
///
/// The Generated Note will be saved securely in your local store
/// for later usage (i.e doing a deposit using this note).
#[derive(StructOpt)]
pub struct GenerateNote {
    /// an easy to remember the Note.
    #[structopt(short, long)]
    alias: Option<String>,
    /// the mixer size that this note will be generated for.
    ///
    /// you can't change this later when you try to do a deposit
    /// using this note.
    ///
    /// leave empty to prompt with the available mixer sizes.
    #[structopt(short, long)]
    size: Option<u128>,
}

#[async_trait]
impl super::CommandExec for GenerateNote {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        let mut term = console::Term::stdout();
        let theme = dialoguer::theme::ColorfulTheme::default();
        let alias = self.alias.unwrap_or_prompt("Note Alias", &theme)?;
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(60);
        let pb_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        pb.set_style(pb_style.clone());
        pb.set_prefix("[1/3]");
        pb.set_message("Connecting ..");
        let api = context.client().await?;
        let props_raw = api.client.rpc().system_properties().await?;
        let props = SystemProperties::from(props_raw);
        let chain_id = api.constants().bridge().chain_identifier()?;
        pb.set_prefix("[2/3]");
        pb.set_message("Fetching Mixers and assets ..");
        let mut mixers_iter =
            api.storage().mixer_bn254().mixers_iter(None).await?;
        let mut mixers = Vec::new();
        let mut assets = HashMap::new();
        while let Some((_, mixer)) = mixers_iter.next().await? {
            let asset = api
                .storage()
                .asset_registry()
                .assets(mixer.asset, None)
                .await?
                .context(format!(
                    "failed to fetch asset #{} information",
                    mixer.asset
                ))?;
            assets.insert(mixer.asset, asset);
            mixers.push(mixer);
        }
        pb.finish_and_clear();
        let (asset, mixer) = if let Some(val) = self.size {
            // find the mixer with the size.
            let size = val;
            let maybe_mixer = mixers
                .iter()
                .find(|mixer| mixer.deposit_size == size)
                .cloned();
            match maybe_mixer {
                Some(v) => (assets[&v.asset].clone(), v),
                None => {
                    let sizes = mixers
                        .iter()
                        .map(|mixer| mixer.deposit_size)
                        .collect::<Vec<_>>();
                    writeln!(term, "Available sizes: {:?}", sizes)?;
                    anyhow::bail!("Invalid Mixer size!");
                },
            }
        } else {
            let f = |(size, asset)| format!("Mixer {size} {asset}");
            let items: Vec<_> = mixers
                .iter()
                .map(|v| {
                    (
                        v.deposit_size,
                        String::from_utf8_lossy(&assets[&v.asset].name.0),
                    )
                })
                .map(f)
                .collect();
            let i = dialoguer::Select::with_theme(&theme)
                .with_prompt("Select Your Mixer")
                .items(&items)
                .interact_on(&term)?;
            (assets[&mixers[i].asset].clone(), mixers[i].clone())
        };
        if !context.has_secret() {
            let password = Option::<SecretString>::None
                .unwrap_or_prompt_password(
                    "Default Account Password",
                    &theme,
                )?;
            context.set_secret(password);
        }
        context
            .signer()
            .context("incorrect default account password!")?;
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(60);
        pb.set_style(pb_style);
        pb.set_prefix("[3/3]");
        pb.set_message("Generating Note..");
        context.generate_note(
            alias.clone(),
            asset,
            mixer,
            props.token_decimals,
            chain_id as _,
        )?;
        pb.finish_with_message("Done!");
        pb.finish_and_clear();
        writeln!(
            term,
            "Note Generated with alias {} and saved locally",
            style(alias).green(),
        )?;
        writeln!(term)?;
        writeln!(term, "Next, Do a dopist using this note.")?;
        writeln!(term, "    $ webb mixer deposit")?;
        Ok(())
    }
}

/// Forget/Remove the Note from your local store.
/// This can be safely done on already used Notes.
///
/// The Notes that are ready to be removed will be marked with `*`.
#[derive(StructOpt)]
pub struct ForgetNote {}

#[async_trait]
impl super::CommandExec for ForgetNote {
    async fn exec(self, _context: &mut ExecutionContext) -> anyhow::Result<()> {
        todo!("Forget Note")
    }
}

/// Deposit an asset to the Mixer.
///
/// After generating a Note, you can do a deposit to the mixer
/// using this Note.
#[derive(StructOpt)]
pub struct DepositAsset {
    /// The Note alias that will be used to do the deposit.
    #[structopt(short, long)]
    alias: Option<String>,
}

#[async_trait]
impl super::CommandExec for DepositAsset {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        let mut term = console::Term::stdout();
        let theme = dialoguer::theme::ColorfulTheme::default();
        let notes: Vec<_> =
            context.notes().iter().filter(|n| !n.used).collect();
        if notes.is_empty() {
            writeln!(term)?;
            writeln!(term, "there is no unused notes saved")?;
            writeln!(term, "try generating new ones or importing them.")?;
            writeln!(term)?;
            writeln!(term, "$ webb mixer help")?;
            anyhow::bail!("No notes saved!");
        }
        let note = if let Some(val) = self.alias {
            notes
                .into_iter()
                .cloned()
                .find(|n| n.alias == val)
                .context("note not found")
        } else {
            let items: Vec<_> =
                notes.iter().map(|n| format!("{}", n)).collect();
            let notes = notes.to_owned();
            let i = dialoguer::Select::with_theme(&theme)
                .with_prompt("Select one of these notes")
                .items(&items)
                .interact_on(&term)?;
            Ok(notes[i].clone())
        }?;

        if !context.has_secret() {
            let password = Option::<SecretString>::None
                .unwrap_or_prompt_password(
                    "Default Account Password",
                    &theme,
                )?;
            context.set_secret(password);
        }
        let signer = context
            .signer()
            .context("incorrect default account password!")?;
        let secret_note = context.decrypt_note(note.uuid.clone())?;
        let api = context.client().await?;
        let pb = ProgressBar::new_spinner();
        let pb_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        pb.enable_steady_tick(60);
        pb.set_style(pb_style);
        pb.set_prefix("[1/3]");
        pb.set_message("Fetching Mixers and assets ..");
        let mixer_count =
            api.storage().merkle_tree_bn254().next_tree_id(None).await?;
        let mut mixers = HashMap::new();
        let mut assets = HashMap::new();
        for i in 0..mixer_count {
            let maybe_mixer =
                api.storage().mixer_bn254().mixers(i, None).await?;
            let mixer = match maybe_mixer {
                Some(m) => m,
                None => continue,
            };
            let asset = api
                .storage()
                .asset_registry()
                .assets(mixer.asset, None)
                .await?
                .context(format!(
                    "failed to fetch asset #{} information",
                    mixer.asset
                ))?;
            assets.insert(mixer.asset, asset);
            mixers.insert(i, mixer);
        }

        let (asset_id, _) = assets
            .into_iter()
            .find(|(_, a)| a.name.0 == secret_note.token_symbol.as_bytes())
            .context(format!(
                "No asset with symbol {} found on-chain!",
                secret_note.token_symbol
            ))?;
        let note_deposit_size = u128::from_str(&secret_note.amount)
            .context("failed to parse note deposit size from it's amount")?;
        let (mixer_id, _) = mixers
            .into_iter()
            .find(|(_, m)| {
                m.asset == asset_id && m.deposit_size == note_deposit_size
            })
            .context("No mixer found for this asset!")?;
        pb.set_prefix("[2/3]");
        pb.set_message("Generating Your secret leaf ...");
        let (leaf, ..) = mixer::get_leaf_from_note(&secret_note)?;
        pb.set_prefix("[3/3]");
        pb.set_message("Doing the deposit...");
        let mut progress = api
            .tx()
            .mixer_bn254()
            .deposit(mixer_id, leaf)
            .sign_and_submit_then_watch(&signer)
            .await?;
        while let Some(state) = progress.next_item().await {
            let s = state?;
            match s {
                TransactionStatus::Ready => {
                    pb.set_message("Transaction is ready ...")
                },
                TransactionStatus::Broadcast(_) => {
                    pb.set_message("Transaction is broadcasted ...");
                },
                TransactionStatus::InBlock(details) => {
                    let tx_hash = details.block_hash();
                    pb.set_message(format!(
                        "Transaction is in block {tx_hash}"
                    ));
                },
                TransactionStatus::Retracted(_) => {
                    pb.set_message("Transaction is retracted ...");
                },
                TransactionStatus::FinalityTimeout(_) => {
                    pb.set_message("Transaction is timeout ...");
                },
                TransactionStatus::Finalized(details) => {
                    let tx_hash = details.block_hash();
                    pb.set_message(format!(
                        "Transaction is finalized {tx_hash}"
                    ));
                },
                TransactionStatus::Usurped(_) => {
                    pb.set_message("Transaction is usurped ...");
                },
                TransactionStatus::Dropped => {
                    pb.set_message("Transaction is dropped ...");
                },
                TransactionStatus::Invalid => {
                    pb.set_message("Transaction is invalid ...");
                    anyhow::bail!("Transaction is invalid!");
                },
                _ => continue,
            };
        }
        context.mark_note_as_used(note.uuid)?;
        pb.finish_and_clear();
        let account_id = signer.account_id().clone();
        let account = api.storage().system().account(account_id, None).await?;
        let props_raw = api.client.rpc().system_properties().await?;
        let props = SystemProperties::from(props_raw);
        let balance =
            account.data.free / 10u128.pow(props.token_decimals as u32);
        writeln!(term, "{} Note Deposited Successfully!", Emoji("🎉", "※"))?;
        writeln!(term)?;
        writeln!(
            term,
            "Your Current Free Balance: {} {}",
            style(balance).green().bold(),
            props.token_symbol,
        )?;
        writeln!(term)?;
        writeln!(term, "Next! to do a withdraw:")?;
        writeln!(term, "    $ webb mixer withdraw -a {}", note.alias)?;

        Ok(())
    }
}

/// Withdraw from the Mixer.
///
/// After doing a deposit, you use the same Note used in the `Deposit`
/// Operation to do a Withdraw and optain your assets again.
#[derive(StructOpt)]
pub struct WithdrawAsset {
    /// The Note alias that will be used for withdrawal.
    ///
    /// this note must be used before in a deposit.
    #[structopt(short, long)]
    alias: Option<String>,
}

#[async_trait]
impl super::CommandExec for WithdrawAsset {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        todo!();
    }
}
