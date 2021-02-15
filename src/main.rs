use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use directories_next::ProjectDirs;
use secrecy::SecretString;
use structopt::StructOpt;

mod commands;
mod context;
mod database;
mod raw;
mod utils;

use commands::{CommandExec, SubCommand};
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
///
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

    /// Use interactive shell for entering the password used by the secret datastore.
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

    let dirs = ProjectDirs::from(
        crate::PACKAGE_ID[0],
        crate::PACKAGE_ID[1],
        crate::PACKAGE_ID[2],
    )
    .context("getting project data")?;

    let db = if let Some(secret) = password(&args)? {
        SledDatastore::with_secret(utils::get_password(
            dirs.data_dir().to_path_buf(),
            Some(secret),
        )?)
    } else {
        SledDatastore::new()
    }
    .context("failed to open the secret datastore!")?;

    let mut context = ExecutionContext::new(db, dirs)
        .context("create execution context for other commands")?;

    match args.sub {
        SubCommand::Show(cmd) => cmd.exec(&mut context).await?,
        SubCommand::Default(cmd) => cmd.exec(&mut context).await?,
        SubCommand::Account(cmd) => cmd.exec(&mut context).await?,
    };

    Ok(())
}

fn password(args: &Opts) -> anyhow::Result<Option<SecretString>> {
    if args.password_interactive {
        utils::ask_for_password("Password: ", 6).map(Option::Some)
    } else if let Some(ref path) = args.password_filename {
        let password =
            fs::read_to_string(path).context("reading password file")?;
        Ok(Some(SecretString::new(password)))
    } else if args.password.is_some() && args.unsafe_flag {
        // TODO(shekohex): emit a warning here about unsafe flag.
        Ok(args.password.clone())
    } else if args.password.is_some() && !args.unsafe_flag {
        let msg = r#"Passing passwords in the options is not recommened.
try using password file or input the password interactively (run `webb --help`).
and also search on `how to delete a command from shell history` to delete this command from
your shell history.

if you going to do this **anyway**, re-run the same command with `--unsafe` flag.
            "#;
        anyhow::bail!(msg);
    } else {
        Ok(None)
    }
}
