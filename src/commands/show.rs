use std::io::Write;

use async_trait::async_trait;
use structopt::StructOpt;

use crate::context::ExecutionContext;

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
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        let mut term = console::Term::stdout();
        match self {
            Self::Home => {
                let home = context.home();
                writeln!(term, "{}", home.display())?;
            },
            Self::Account => {
                let accounts = context.accounts();
                if let Some(account) = accounts.iter().find(|a| a.is_default) {
                    writeln!(term, "{}", account)?;
                } else {
                    writeln!(term, "you don't have any accounts.")?;
                    writeln!(term, "try generating or importing them:")?;
                    writeln!(term, "    $ webb account help")?;
                    writeln!(term)?;
                }
            },
        };
        Ok(())
    }
}
