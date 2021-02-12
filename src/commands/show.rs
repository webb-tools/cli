use anyhow::bail;
use async_trait::async_trait;
use structopt::StructOpt;

use crate::context::Context;

/// Show the active account (if any)
/// and other information about the CLI Configrations.
#[derive(StructOpt)]
pub enum ShowCommand {
    /// Display the path to the Webb CLI.
    Home,
    /// Shows the active Account.
    Account,
}

#[async_trait]
impl super::CommandExec for ShowCommand {
    async fn exec(&self, context: &mut Context) -> anyhow::Result<()> {
        match self {
            Self::Home => {
                let home = context.home();
                println!("{}", home.display());
            }
            Self::Account => {
                let accounts = context.accounts();
                if let Some(account) = accounts.iter().find(|a| a.is_default) {
                    println!("{}: {}", account.alias, account.address);
                } else {
                    eprintln!("it sounds that you don't have any accounts.");
                    eprintln!("try generating or importing them.");
                    eprintln!("$ webb account help");
                    eprintln!();
                    bail!("no account available");
                }
            }
        };
        Ok(())
    }
}
