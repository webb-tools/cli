use std::io::Write;

use anyhow::bail;
use async_trait::async_trait;
use dialoguer::theme::ColorfulTheme;
use structopt::StructOpt;

use crate::context::ExecutionContext;

/// Set the default account to be used for all operations.
#[derive(StructOpt)]
pub struct DefaultCommand {
    /// Account alias, such as 'shekohex' or supply the account address
    /// directly.
    /// such as '5GHnQYfvZdxJHSWnZqiM5eKdj2UawJs4s9Tqn22ckvLEENvc'.
    ///
    /// to list all accounts you own try `webb account list`.
    #[structopt(short, long)]
    alias_or_address: Option<String>,
}

#[async_trait]
impl super::CommandExec for DefaultCommand {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        let mut term = console::Term::stdout();
        let handler = if let Some(val) = self.alias_or_address {
            Result::<_, anyhow::Error>::Ok(val)
        } else {
            // Prompt the user to choose one of the accounts.
            let non_default_accounts: Vec<_> = context
                .accounts()
                .to_owned()
                .into_iter()
                .filter(|a| !a.is_default)
                .map(|v| v.alias)
                .collect();
            if non_default_accounts.is_empty() {
                bail!("you don't have any accounts saved.");
            }
            let i = dialoguer::Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select one of these accounts")
                .items(&non_default_accounts)
                .interact_on(&term)?;
            Ok(non_default_accounts[i].clone())
        }?;
        let changed = context.set_default_account(&handler)?;
        if changed {
            writeln!(term, "default: {}", handler)?;
        } else {
            bail!("no account with alias nor address equal to: {}", handler);
        }
        Ok(())
    }
}
