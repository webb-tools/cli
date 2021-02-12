use anyhow::bail;
use async_trait::async_trait;
use structopt::StructOpt;

use crate::context::Context;

/// Set the default account to be used for all operations.
#[derive(StructOpt)]
pub struct DefaultCommand {
    /// Account alias, such as 'shekohex' or supply the account address directly.
    /// such as '5GHnQYfvZdxJHSWnZqiM5eKdj2UawJs4s9Tqn22ckvLEENvc'.
    ///
    /// to list all accounts you own try `webb account list`.
    alias_or_address: String,
}

#[async_trait]
impl super::CommandExec for DefaultCommand {
    async fn exec(&self, context: &mut Context) -> anyhow::Result<()> {
        let changed = context.set_default_account(&self.alias_or_address)?;
        if changed {
            println!("default: {}", self.alias_or_address);
        } else {
            bail!(
                "no account available with alias nor address equal to: {}",
                self.alias_or_address
            );
        }
        Ok(())
    }
}
