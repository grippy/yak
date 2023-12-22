use anyhow::{Context, Result};
use clap::{arg, Args};
use log::info;
use std::fs;
use std::path::PathBuf;
use yak_ast::Ast;
use yak_core::models::yak_env::YakEnv;

#[derive(Args, Debug)]
pub(crate) struct BuildArgs {
    /// Yak package folder path
    #[arg(short, long, default_value_t = String::from("."))]
    path: String,
}

pub(crate) fn call(args: &BuildArgs) -> Result<()> {
    info!("build args: {:?}", args);

    let yak_env = YakEnv::default();
    let cwd = yak_env.cwd()?;
    info!("current directory: {}", &cwd.display());

    // package directory
    let abs_path = fs::canonicalize(&args.path)
        .with_context(|| format!("Failed to canonicalize build args path: {}", &args.path))?;
    info!("checking abs path directory: {:?}", &abs_path);

    // append yak.pkg to end of path
    let pkgfile = if !abs_path.ends_with("/yak.pkg") {
        PathBuf::from_iter([&abs_path, &PathBuf::from("yak.pkg")].iter())
    } else {
        abs_path.clone()
    };

    // check if yak.pkg file exists
    let pkgfile = fs::canonicalize(&pkgfile)
        .with_context(|| format!("Failed to find  package file path: {}", &pkgfile.display()))?;
    info!("building package file: {:?}", &pkgfile);

    // parse the package file
    let mut pkg_ast = Ast::from_file(pkgfile)?;
    pkg_ast.parse()?;

    Ok(())
}
