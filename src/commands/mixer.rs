use std::io::Write;

use async_trait::async_trait;
use structopt::StructOpt;

use crate::context::ExecutionContext;

/// Webb Crypto Mixer.
#[derive(StructOpt)]
pub enum MixerCommand {
    /// List all of your saved Notes.
    List,
    /// Imports a previously generated Note.
    Import(ImportNote),
    /// Generates a new Note and save it.
    Generate(GenerateNote),
    /// Remove/Forget a Note.
    Forget(ForgetNote),
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
            MixerCommand::Import(cmd) => cmd.exec(context).await,
            MixerCommand::Generate(cmd) => cmd.exec(context).await,
            MixerCommand::Forget(cmd) => cmd.exec(context).await,
            MixerCommand::Deposit(cmd) => cmd.exec(context).await,
            MixerCommand::Withdraw(cmd) => cmd.exec(context).await,
        }
    }
}

/// Import a previously generated Note to your local secure store.
///
/// The Note could be generated previously from the Webb UI.
#[derive(StructOpt)]
pub struct ImportNote {}

#[async_trait]
impl super::CommandExec for ImportNote {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        todo!()
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
    group: Option<u8>,
}

#[async_trait]
impl super::CommandExec for GenerateNote {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        todo!()
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
