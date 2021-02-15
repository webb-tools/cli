use async_trait::async_trait;
use bip39::{Language, Mnemonic};
use structopt::StructOpt;
use subxt::sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
use subxt::sp_runtime::traits::IdentifyAccount;

use crate::context::ExecutionContext;

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
    alias: String,
    /// a password to secure the account.
    /// if not provided as an option you will be prompted to enter it.
    ///
    /// the password can be provided environment variable.
    #[structopt(short, long, env = "WEBB_PASSWORD")]
    password: Option<String>,
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
    alias: String,
    /// a password to secure the account.
    /// if not provided as an option you will be prompted to enter it.
    ///
    /// the password can be provided environment variable.
    #[structopt(short, long, env = "WEBB_PASSWORD")]
    password: Option<String>,
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
    async fn exec(&self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        use AccountCommand::*;
        match self {
            List => {
                let mut accounts = context.accounts().to_owned();
                if accounts.is_empty() {
                    println!();
                    println!("it sounds that there is no accounts saved");
                    println!("try generating or importing them.");
                    println!();
                    println!("$ webb account help");
                    return Ok(());
                }
                // put the default account first.
                accounts.sort_by(|a, b| b.is_default.cmp(&a.is_default));

                for account in accounts {
                    println!("{}: {}", account.alias, account.address);
                }
                Ok(())
            }
            Import(cmd) => cmd.exec(context).await,
            Generate(cmd) => cmd.exec(context).await,
            Forget(cmd) => cmd.exec(context).await,
        }
    }
}

#[async_trait]
impl super::CommandExec for ImportAccount {
    async fn exec(&self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        println!("Importing account with {}", self.alias);
        let password = if let Some(password) = self.password.clone() {
            secrecy::SecretString::new(password)
        } else {
            crate::utils::ask_for_new_password(8)?
        };

        let paper_key = if let Some(paper_key) = self.mnemonic.clone() {
            Mnemonic::from_phrase(&paper_key, Language::English)?
        } else {
            crate::utils::ask_for_phrase("Enter PaperKey (Mnemonic Seed): ")?
        };
        let alias = self.alias.clone();
        let address = context.import_account(alias, password, paper_key)?;
        let account = address
            .into_account()
            .to_ss58check_with_version(Ss58AddressFormat::SubstrateAccount);
        println!("Account Imported:");
        println!("{}: {}", self.alias, account);
        println!();
        println!("To set this account as default:");
        println!("    $ webb default {}", self.alias);
        Ok(())
    }
}

#[async_trait]
impl super::CommandExec for GenerateAccount {
    async fn exec(&self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        println!("Generating new account with {}", self.alias);
        let password = if let Some(password) = self.password.clone() {
            secrecy::SecretString::new(password)
        } else {
            crate::utils::ask_for_new_password(8)?
        };
        let alias = self.alias.clone();
        let (address, seed) = context.generate_account(alias, password)?;
        println!("Account Generated:");
        println!("{}: {}", self.alias, address);
        println!();
        println!("IMPORTANT!!");
        println!("Generated 12-word mnemonic seed:");
        println!("{}", seed);
        println!();
        println!("Please write down your wallet's mnemonic seed and keep it in a safe place.");
        println!("The mnemonic can be used to restore your wallet.");
        println!("Keep it carefully to not lose your assets.");
        println!();
        println!("To set this account as default:");
        println!("    $ webb default {}", self.alias);
        Ok(())
    }
}

#[async_trait]
impl super::CommandExec for ForgetAccount {
    async fn exec(&self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        todo!("forget account")
    }
}
