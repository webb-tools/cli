use std::path::PathBuf;

use async_trait::async_trait;
use secrecy::SecretString;
use structopt::StructOpt;

use crate::{context::ExecutionContext, utils};

mod account;
mod default;
mod mixer;
mod show;

/// A General trait used to organize all commands.
#[async_trait]
pub trait CommandExec {
    async fn exec(self, context: &mut ExecutionContext) -> anyhow::Result<()>;
}

#[derive(StructOpt)]
pub enum SubCommand {
    Show(show::ShowCommand),
    Default(default::DefaultCommand),
    Account(account::AccountCommand),
    Mixer(mixer::MixerCommand),
}

#[derive(StructOpt, Clone, Debug)]
pub struct PasswordOpts {
    /// Use interactive shell for entering the password used by the secret
    /// datastore.
    #[structopt(
        global = true,
        long = "password-interactive",
        conflicts_with_all = &["password", "password-filename"]
    )]
    pub password_interactive: bool,

    /// Password used by the secret datastore.
    #[structopt(
        global = true,
        long = "password",
        short,
        parse(try_from_str = utils::secret_string_from_str),
        conflicts_with_all = &["password-interactive", "password-filename"]
    )]
    pub password: Option<SecretString>,

    /// File that contains the password used by secret datastore.
    #[structopt(
        global = true,
        long = "password-filename",
        value_name = "PATH",
        parse(from_os_str),
        conflicts_with_all = &["password-interactive", "password"]
    )]
    pub password_filename: Option<PathBuf>,
}

#[derive(StructOpt, Clone, Debug)]
pub struct NodeOpts {
    /// Set the Node Url where we will connect to.
    #[structopt(
        global = true,
        long = "node-url",
        default_value = "ws://127.0.0.1:9944",
        env = "WEBB_NODE_URL",
        parse(try_from_str = url::Url::parse)
    )]
    pub url: url::Url,
}
