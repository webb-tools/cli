use std::io::Write;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use secrecy::SecretString;
use structopt::StructOpt;
use webb_cli::mixer::{Note, TokenSymbol};
use webb_cli::runtime::{MixerGroupIdsStore, WebbRuntime};

use crate::context::ExecutionContext;

/// Webb Crypto Mixer.
#[derive(StructOpt)]
pub enum MixerCommand {
    /// List all of your saved Notes.
    List,
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
            MixerCommand::List => {
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
        let alias = if let Some(val) = self.alias {
            val
        } else {
            dialoguer::Input::with_theme(&theme)
                .with_prompt("Note Alias")
                .interact_on(&term)?
        };
        let note = if let Some(val) = self.note {
            Note::from_str(&val)?
        } else {
            loop {
                let value: String = dialoguer::Input::with_theme(&theme)
                    .with_prompt("Note")
                    .interact_on(&term)?;
                match Note::from_str(&value) {
                    Ok(v) => break v,
                    Err(e) => {
                        writeln!(term, "{}", style(e).red())?;
                        continue;
                    },
                };
            }
        };
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
    /// the mixer group that this note will be generated for.
    ///
    /// you can't change this later when you try to do a deposit
    /// using this note.
    ///
    /// leave empty to prompt with the available mixer groups.
    #[structopt(short, long)]
    group: Option<u32>,
}

#[async_trait]
impl super::CommandExec for GenerateNote {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        type MixerGroupIds = MixerGroupIdsStore<WebbRuntime>;
        let mut term = console::Term::stdout();
        let theme = dialoguer::theme::ColorfulTheme::default();
        let alias = if let Some(val) = self.alias {
            val
        } else {
            dialoguer::Input::with_theme(&theme)
                .with_prompt("Note Alias")
                .interact_on(&term)?
        };
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(60);
        let pb_style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        pb.set_style(pb_style.clone());
        pb.set_prefix("[1/3]");
        pb.set_message("Connecting ..");
        async_std::task::sleep(Duration::from_secs(2)).await;
        let client = context.client().await?;
        pb.set_prefix("[2/3]");
        pb.set_message("Getting Mixer Groups ..");
        let mixer_group_ids = client
            .fetch_or_default(&MixerGroupIds::default(), None)
            .await?;
        async_std::task::sleep(Duration::from_secs(3)).await;
        pb.finish_and_clear();
        let mixer_group_id = if let Some(val) = self.group {
            if mixer_group_ids.contains(&val) {
                val
            } else {
                writeln!(term, "Available groups: {:?}", mixer_group_ids)?;
                anyhow::bail!("Invalid Mixer group!");
            }
        } else {
            let items: Vec<_> = mixer_group_ids
                .iter()
                .map(|i| {
                    format!(
                        "Group #{} with 1,000{} EDG",
                        i,
                        "0".repeat(*i as usize)
                    )
                })
                .collect();
            let i = dialoguer::Select::with_theme(&theme)
                .with_prompt("Select Mixer Group")
                .items(&items)
                .interact_on(&term)?;
            mixer_group_ids[i]
        };
        if !context.has_secret() {
            let password = dialoguer::Password::with_theme(&theme)
                .with_prompt("Password")
                .interact_on(&term)?;
            let secret = SecretString::new(password);
            context.set_secret(secret);
        }
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(60);
        pb.set_style(pb_style);
        pb.set_prefix("[3/3]");
        pb.set_message("Generating Note..");
        context.generate_note(
            alias.clone(),
            mixer_group_id,
            TokenSymbol::Edg,
        )?;
        pb.finish_with_message("Done!");
        pb.finish_and_clear();
        writeln!(
            term,
            "Note Generated with alias {} for #{} Mixer Group",
            style(alias).green(),
            mixer_group_id
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
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        todo!()
    }
}

/// Deposit an asset to the Mixer.
///
/// After generating a Note, you can do a deposit to the mixer
/// using this Note.
#[derive(StructOpt)]
pub struct DepositAsset {}

#[async_trait]
impl super::CommandExec for DepositAsset {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        todo!()
    }
}

/// Withdraw from the Mixer.
///
/// After doing a deposit, you use the same Note used in the `Deposit`
/// Operation to do a Withdraw and optain your assets again.
#[derive(StructOpt)]
pub struct WithdrawAsset {}

#[async_trait]
impl super::CommandExec for WithdrawAsset {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        todo!()
    }
}
