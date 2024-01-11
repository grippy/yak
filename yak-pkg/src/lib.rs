use anyhow::{bail, Context, Result};
use log::info;
use std::fs;
use std::path::PathBuf;
use yak_ast::Ast;
use yak_compiler::compiler::Compiler;
use yak_compiler::hir::Hir;
use yak_core::models::yak_env::YakEnv;
use yak_core::models::yak_home::YakHome;
// use yak_core::models::yak_package::YakPackage;
use yak_core::utils::download_file;

// Get a remote package and cache it locally
// in the YAK_HOME version package src directory
pub fn get(path: String) -> Result<()> {
    info!("get path: {path}");

    // create home directory?
    let yak_home = YakHome::default();
    yak_home.create_home_dir()?;

    let remote_path = path.clone();
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
    let local_pkgfile = format!("{}/yak.pkg", pkg_local_path);
    download_file(&remote_pkgfile, &local_pkgfile)?;

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

    // TODO: cache dependencies
    let dep_urls = yak_pkg.get_remote_dep_urls();
    info!("get remote dependency urls: {:#?}", &dep_urls);

    // get remote files...
    yak_pkg.get_remote_files()?;

    // parse local package files...
    for pkg_file in &yak_pkg.pkg_files {
        pkg_ast.parse_file(PathBuf::from(&pkg_file.local_path))?;
    }

    info!("=============");
    info!("{:#?}", &pkg_ast);
    info!("=============");

    Ok(())
}

pub fn build(path: String) -> Result<()> {
    info!("build path: {}", path);

    let yak_env = YakEnv::default();
    let cwd = yak_env.cwd()?;
    info!("current directory: {}", &cwd.display());

    // package directory
    let abs_path = fs::canonicalize(&path)
        .with_context(|| format!("Failed to canonicalize build args path: {}", &path))?;
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
    pkg_ast.parse_package()?;
    let parsed_pkg = pkg_ast.parsed.package.clone();

    // convert AST => yak_package
    let mut pkg_local_path = abs_path.into_os_string().into_string().unwrap();
    if pkg_local_path.ends_with("/yak.pkg") {
        pkg_local_path = pkg_local_path.replace("/yak.pkg", "");
    }
    let mut yak_pkg = parsed_pkg.into_yak_package(true, pkg_local_path, None)?;
    info!("======PKG======");
    info!("{:#?}", yak_pkg);
    info!("===============");

    // TODO: cache dependencies here

    // build src code
    let pkg_files = yak_pkg.get_local_file_paths()?;
    for pkg_file in pkg_files.into_iter() {
        pkg_ast.parse_file(pkg_file)?;
    }
    info!("======AST======");
    info!("{:#?}", &pkg_ast);
    info!("===============");

    let hir = Hir::from_ast(&pkg_ast)?;
    info!("======HIR======");
    info!("{:#?}", &hir);
    info!("===============");

    // Build
    Compiler::build(hir)?;

    Ok(())
}
