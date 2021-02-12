use async_trait::async_trait;
use structopt::StructOpt;

use crate::context::Context;

mod account;
mod default;
mod show;

#[async_trait]
pub trait CommandExec {
    async fn exec(&self, context: &mut Context) -> anyhow::Result<()>;
}

#[derive(StructOpt)]
pub enum SubCommand {
    Show(show::ShowCommand),
    Default(default::DefaultCommand),
    Account(account::AccountCommand),
}
