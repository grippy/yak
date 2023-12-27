mod cmd;
use anyhow::Result;
use clap::{Parser, Subcommand};
use log::error;
use pretty_env_logger;

const YAK_VERSION: &'static str = "0.0.1";

#[derive(Parser)]
#[command(name = "yak")]
#[command(author = "Greg Melton <gmelton@gmail.com>")]
#[command(version = YAK_VERSION)]
#[command(about = "Yak Programming Tools", long_about = None)]
struct Cli {
    /// Set yak home directory
    #[clap(long, default_value = "~/.yak", env = "YAK_HOME")]
    yak_home: String,
    /// Set default log-level
    #[clap(long, default_value = "info", env = "YAK_LOG")]
    yak_log: String,
    /// Set yak version
    #[clap(long, default_value = YAK_VERSION, env = "YAK_VERSION")]
    yak_version: String,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build Yak Packages
    Build(cmd::build::BuildArgs),
    /// Get Yak Packages
    Get(cmd::get::GetArgs),
}

fn main() -> Result<()> {
    let mut cli = Cli::parse();

    // set YAK_LOG env var if it doesn't exist
    let yak_log = std::env::var("YAK_LOG");
    match yak_log {
        Ok(val) => {
            cli.yak_log = val;
        }
        _ => {}
    }
    // set the log level here
    std::env::set_var("YAK_LOG", cli.yak_log);
    pretty_env_logger::init_custom_env("YAK_LOG");

    // set YAK_VERSION env variable if it doesn't exist
    let yak_version = std::env::var("YAK_VERSION");
    match yak_version {
        Ok(val) => {
            cli.yak_version = val;
        }
        _ => {}
    }
    std::env::set_var("YAK_VERSION", cli.yak_version);

    let results = match &cli.cmd {
        Cmd::Build(args) => cmd::build::call(args),
        Cmd::Get(args) => cmd::get::call(args),
    };
    if results.is_err() {
        let err = results.err().unwrap();
        error!("{:?}", &err);
        std::process::exit(1)
    }
    Ok(())
}
