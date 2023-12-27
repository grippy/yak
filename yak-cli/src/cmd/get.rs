use anyhow::{bail, Context, Result};
use clap::{arg, Args};
use log::info;
use std::path::{Path, PathBuf};
use std::{fs, fs::File, io::copy};
use url::{ParseError, Url};
use yak_ast::Ast;
use yak_core::models::yak_home::YakHome;

fn download_file(url: &str, file_dir: &str, file_name: &str) -> Result<()> {
    // Send an HTTP GET request to the URL
    let mut response = reqwest::blocking::get(url)?;

    // Create file path...
    if !Path::new(file_dir).exists() {
        let _ = fs::create_dir_all(file_dir)
            .with_context(|| format!("failed to create file directory: {}", file_dir))?;
    }
    // Append file name to file path
    let mut fd = PathBuf::from(&file_dir);
    fd.push(&file_name);

    // Create a new file to write the downloaded image to
    let mut file = File::create(fd)?;

    // Copy the contents of the response to the file
    copy(&mut response, &mut file)?;

    Ok(())
}

#[derive(Args, Debug)]
pub(crate) struct GetArgs {
    /// Download remote package path to the local cache
    #[arg(short, long)]
    path: String,
}

pub(crate) fn call(args: &GetArgs) -> Result<()> {
    info!("get args: {:?}", args);

    // create home directory?
    let yak_home = YakHome::default();
    yak_home.create_home_dir()?;

    let remote_path = args.path.clone();
    if !remote_path.starts_with("http://") && !remote_path.starts_with("https://") {
        bail!("get only supports downloading remote packages via http or https")
    }

    // we need the package file
    let mut remote_pkgfile = remote_path.clone();
    if !remote_pkgfile.ends_with("/yak.pkg") {
        remote_pkgfile += "/yak.pkg";
    }

    // strip the scheme from the args path
    let mut src_root_path = remote_path.clone();
    src_root_path = src_root_path.replace("http://", "");
    src_root_path = src_root_path.replace("https://", "");

    let mut yak_pkg_src = yak_home.get_home_version_src_dir();
    yak_pkg_src.push(src_root_path);

    // download package file
    info!(
        "downloading package file {} to {}",
        &remote_pkgfile,
        &yak_pkg_src.display()
    );

    let pkg_local_path = yak_pkg_src.as_os_str().to_str().unwrap();
    download_file(&remote_pkgfile, pkg_local_path, "yak.pkg")?;

    // parse the package file
    let mut pkgfile = PathBuf::from(pkg_local_path);
    pkgfile.push("yak.pkg");
    let mut pkg_ast = Ast::from_file(pkgfile)?;
    pkg_ast.parse_package()?;
    let parsed_pkg = pkg_ast.parsed.package.clone();

    // convert AST => yak_package
    let mut yak_pkg =
        parsed_pkg.into_yak_package(true, pkg_local_path.to_string(), Some(remote_path))?;
    info!("yak_pkg: {:?}", yak_pkg);

    // get remote files...
    yak_pkg.get_remote_files()?;

    // parse local package files...
    for pkg_file in yak_pkg.pkg_files {
        pkg_ast.parse_file(PathBuf::from(&pkg_file.local_path))?;
    }

    info!("pkg_ast {:#?}", &pkg_ast);

    Ok(())
}
