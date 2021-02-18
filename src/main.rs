use std::fs;

use anyhow::Context;
use directories_next::ProjectDirs;
use secrecy::SecretString;
use structopt::StructOpt;

mod commands;
mod context;
mod database;
mod raw;
mod utils;

use commands::{CommandExec, NodeOpts, PasswordOpts, SubCommand};
use context::ExecutionContext;
use database::SledDatastore;

const PACKAGE_ID: [&str; 3] = ["tools", "webb", "webb-cli"];

/// 🕸️  The Webb Command-line tools 🧰
///
/// Start by generating new account:
///
///     $ webb account generate -a <YOUR_ACCOUNT_NAME>
///
/// or by importing existing one:
///
///     $ webb account import -a <YOUR_ACCOUNT_NAME>
///
/// To set an account as the default one for any operation try:
///
///     $ webb default <ACCOUNT_ALIAS_OR_ADDRESS>
#[derive(StructOpt)]
#[structopt(name = "Webb CLI")]
struct Opts {
    /// A level of verbosity, and can be used multiple times
    #[structopt(short, long, parse(from_occurrences))]
    verbose: i32,
    /// Enalbe unsafe operations.
    ///
    /// like delete an account, read the password from passed options
    /// and many other unsafe operations.
    #[structopt(global = true, long = "unsafe")]
    unsafe_flag: bool,
    /// Password Options.
    #[structopt(flatten)]
    password: PasswordOpts,
    /// Node Options.
    #[structopt(flatten)]
    node: NodeOpts,
    /// Sub-Commands.
    #[structopt(subcommand)]
    sub: SubCommand,
}

#[paw::main]
#[async_std::main]
async fn main(args: Opts) -> anyhow::Result<()> {
    let log_level = match args.verbose {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::max(),
    };
    // setup logger
    env_logger::builder()
        .format_timestamp(None)
        .filter_module("webb", log_level)
        .init();
    log::debug!("Getting default dirs for webb cli");
    let dirs = ProjectDirs::from(
        crate::PACKAGE_ID[0],
        crate::PACKAGE_ID[1],
        crate::PACKAGE_ID[2],
    )
    .context("getting project data")?;

    log::debug!("our data dirs live in: {}", dirs.data_dir().display());
    log::debug!("now let's try to get the account password");
    let db = if let Some(secret) = password(&args)? {
        log::debug!("now we have a secret, creating a secret datastore!");
        SledDatastore::with_secret(secret)
    } else {
        log::debug!("no secrets provided, open the datastore anyway");
        SledDatastore::new()
    }
    .context("failed to open the secret datastore!")?;

    log::debug!("creating an execution context for all of the commands");
    let mut context = ExecutionContext::new(db, dirs, args.node.url)
        .context("create execution context for other commands")?;
    match args.sub {
        SubCommand::Show(cmd) => cmd.exec(&mut context).await?,
        SubCommand::Default(cmd) => cmd.exec(&mut context).await?,
        SubCommand::Account(cmd) => cmd.exec(&mut context).await?,
        SubCommand::Mixer(cmd) => cmd.exec(&mut context).await?,
    };

    Ok(())
}

fn password(args: &Opts) -> anyhow::Result<Option<SecretString>> {
    let password_opts = &args.password;
    if password_opts.password_interactive {
        let theme = dialoguer::theme::ColorfulTheme::default();
        let password = dialoguer::Password::with_theme(&theme)
            .with_prompt("Password")
            .interact()?;
        Ok(Some(SecretString::new(password)))
    } else if let Some(ref path) = password_opts.password_filename {
        let password = fs::read_to_string(path)
            .context("trying to read the password from the file")?;
        Ok(Some(SecretString::new(password)))
    } else if password_opts.password.is_some() && args.unsafe_flag {
        log::warn!("using unsafe flag!!");
        // TODO(shekohex): emit a warning here about unsafe flag.
        Ok(password_opts.password.clone())
    } else if password_opts.password.is_some() && !args.unsafe_flag {
        anyhow::bail!(include_str!("messages/password_option.txt"));
    } else {
        Ok(None)
    }
}
