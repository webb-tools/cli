use anyhow::Context;
use directories_next::ProjectDirs;
use structopt::StructOpt;

mod commands;
mod context;
mod database;
mod raw;
mod utils;

use commands::{CommandExec, SubCommand};
use context::ExecutionContext;

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
    #[structopt(long = "unsafe")]
    unsafe_flag: bool,
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

    let dirs = ProjectDirs::from("tools", "webb", "webb-cli")
        .context("getting project data")?;

    let db_path = dirs.data_dir().join("db");
    let db = sled::open(db_path).context("open database")?;

    let mut context = ExecutionContext::new(db.clone(), dirs)
        .context("create execution context")?;

    match args.sub {
        SubCommand::Show(cmd) => cmd.exec(&mut context).await?,
        SubCommand::Default(cmd) => cmd.exec(&mut context).await?,
        SubCommand::Account(cmd) => cmd.exec(&mut context).await?,
    };
    db.flush()?;
    Ok(())
}
