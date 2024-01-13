use anyhow::Result;
use clap::{arg, Args};
use log::info;

#[derive(Args, Debug)]
pub(crate) struct GetArgs {
    /// Download remote package path to the local cache
    #[arg(index = 1)]
    path: String,
}

pub(crate) fn call(args: &GetArgs) -> Result<()> {
    info!("get args: {:?}", args);
    // get package path
    let path = args.path.clone();
    let get_args = &mut yak_pkg::GetArgs {
        pkg_root: true,
        path: path,
    };
    let pkg = yak_pkg::get(get_args)?;
    info!("{:#?}", &pkg);
    Ok(())
}
