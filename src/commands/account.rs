use std::io::Write;

use async_trait::async_trait;
use bip39::{Language, Mnemonic};
use console::{style, Emoji};
use dialoguer::theme::ColorfulTheme;
use secrecy::SecretString;
use structopt::StructOpt;
use subxt::{
    sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec},
    sp_runtime::traits::IdentifyAccount,
};
use webb::substrate::subxt;

use crate::{context::ExecutionContext, ext::OptionPromptExt};

/// Modify or query the saved accounts.
#[derive(StructOpt)]
pub enum AccountCommand {
    /// List all accounts you own.
    List,
    /// Imports an Account using the Mnemonic phrase
    /// or as we call it a `PaperKey`.
    Import(ImportAccount),
    /// Generates a new account and save it.
    Generate(GenerateAccount),
    /// Remove/Forget an account.
    Forget(ForgetAccount),
}

/// To Restore an existing account.
/// you need to supply the `alias` and a password.
///
/// the password can be provided environment variable.
///
/// Note: The Password must be the same as the one used to generate the account
/// otherwise it will not generate the same account.
#[derive(StructOpt)]
pub struct ImportAccount {
    /// an easy to remember account name.
    #[structopt(short, long)]
    alias: Option<String>,
    /// the paper key or the mnemonic seed phrase
    /// that got generated with this account.
    ///
    /// could be also provided using the environment variable.
    #[structopt(short, long, env = "WEBB_MNEMONIC")]
    mnemonic: Option<String>,
}

/// For Generate a new account.
/// you need to supply the `alias` and a password.
///
/// the password can be provided environment variable.
#[derive(StructOpt)]
pub struct GenerateAccount {
    /// an easy to remember account name.
    #[structopt(short, long)]
    alias: Option<String>,
}

/// Removes the account from the local store.
/// you can re-import the account again using the password
/// and the mnemonic seed phrase.
///
/// to import an account see:
///
///     $ webb account import --help
#[derive(StructOpt)]
pub struct ForgetAccount {}

#[async_trait]
impl super::CommandExec for AccountCommand {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        use AccountCommand::*;
        match self {
            List => {
                let mut accounts = context.accounts().to_owned();
                let mut term = console::Term::stdout();
                if accounts.is_empty() {
                    write!(term, "{} ", style("uh oh").red())?;
                    writeln!(term, "there is no accounts saved")?;
                    writeln!(term, "try generating or importing them.")?;
                    writeln!(term)?;
                    writeln!(term, "$ webb account help")?;
                    return Ok(());
                }
                // put the default account first.
                accounts.sort_by(|a, b| b.is_default.cmp(&a.is_default));

                for account in accounts {
                    writeln!(term, "{}", account)?;
                }
                Ok(())
            },
            Import(cmd) => cmd.exec(context).await,
            Generate(cmd) => cmd.exec(context).await,
            Forget(cmd) => cmd.exec(context).await,
        }
    }
}

#[async_trait]
impl super::CommandExec for ImportAccount {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        let mut term = console::Term::stdout();
        let theme = ColorfulTheme::default();
        let alias = self.alias.unwrap_or_prompt("Account Alias", &theme)?;
        writeln!(term, "Importing account with {}", style(&alias).blue())?;

        let paper_key = if let Some(paper_key) = self.mnemonic {
            Mnemonic::from_phrase(&paper_key, Language::English)?
        } else {
            crate::utils::ask_for_phrase("Enter PaperKey (Mnemonic Seed): ")?
        };
        if !context.has_secret() {
            let password = Option::<SecretString>::None
                .unwrap_or_prompt_password_with_confirmation(
                    "Password", &theme,
                )?;
            context.set_secret(password);
        }
        let address = context.import_account(alias.clone(), paper_key)?;
        let account = address.into_account().to_ss58check_with_version(
            Ss58AddressFormatRegistry::SubstrateAccount.into(),
        );
        writeln!(term, "{} Account Imported!", Emoji("🎉", "※"))?;
        writeln!(
            term,
            "{}: {}",
            style(&alias).blue(),
            style(account).dim().green()
        )?;
        writeln!(term)?;
        writeln!(term, "Next! to set this account as default:")?;
        writeln!(term, "    $ webb default {}", alias)?;
        Ok(())
    }
}

#[async_trait]
impl super::CommandExec for GenerateAccount {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        let mut term = console::Term::stdout();
        let theme = ColorfulTheme::default();
        let alias = self.alias.unwrap_or_prompt("Account Alias", &theme)?;
        writeln!(term, "Generating new account with {}", style(&alias).blue())?;

        if !context.has_secret() {
            let password = Option::<SecretString>::None
                .unwrap_or_prompt_password_with_confirmation(
                    "Password", &theme,
                )?;
            context.set_secret(password);
        }
        let (address, seed) = context.generate_account(alias.clone())?;
        writeln!(term, "{} Account Generated!", Emoji("🎉", "※"))?;
        writeln!(term)?;
        writeln!(
            term,
            "{}: {}",
            style(&alias).blue(),
            style(address).dim().green()
        )?;
        writeln!(term)?;
        writeln!(
            term,
            "{emoji} {i} {emoji}",
            i = style("IMPORTANT").bright().bold().red(),
            emoji = Emoji("⚠️ ", "!!")
        )?;
        writeln!(term, "Generated 12-word mnemonic seed:")?;
        writeln!(term, "{}", style(seed).bright().bold())?;
        writeln!(term)?;
        writeln!(term, "Please write down your wallet's mnemonic seed and keep it in a safe place.")?;
        writeln!(term, "The mnemonic can be used to restore your wallet.")?;
        writeln!(term, "Keep it carefully to not lose your assets.")?;
        writeln!(term)?;
        writeln!(term, "To set this account as default:")?;
        writeln!(term, "    $ webb default -a {}", alias)?;
        Ok(())
    }
}

#[async_trait]
impl super::CommandExec for ForgetAccount {
    async fn exec(self, _context: &mut ExecutionContext) -> anyhow::Result<()> {
        todo!("forget account")
    }
}
