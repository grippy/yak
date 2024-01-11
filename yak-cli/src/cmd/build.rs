use anyhow::Result;
use clap::{arg, Args};
use log::info;

#[derive(Args, Debug)]
pub(crate) struct BuildArgs {
    /// Yak package folder path
    #[arg(index = 1, default_value_t = String::from("."))]
    path: String,
}

pub(crate) fn call(args: &BuildArgs) -> Result<()> {
    info!("build args: {:?}", args);
    let path = args.path.clone();
    yak_pkg::build(path)?;
    Ok(())
}
