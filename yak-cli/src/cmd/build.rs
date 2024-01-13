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
    let build_args = yak_pkg::BuildArgs {
        pkg_root: true,
        pkg_as_pkg_id: None,
        path: path,
    };
    let pkg = yak_pkg::build(build_args)?;
    info!("{:#?}", &pkg);
    Ok(())
}
