use anyhow::{bail, Context, Result};
use log::info;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use yak_ast::Ast;
use yak_compiler::compiler::{Compiler, CompilerOpts};
use yak_compiler::hir::Hir;
use yak_core::models::yak_env::YakEnv;
use yak_core::models::yak_home::YakHome;
use yak_core::models::yak_package::YakPackage;
use yak_core::utils::download_file;

#[derive(Debug, Default)]
pub struct YakPkg {
    pkg: YakPackage,
    deps: HashMap<String, Self>,
    ast: Option<Ast>,
    hir: Option<Hir>,
}

#[derive(Debug, Default)]
pub struct GetArgs {
    pub pkg_root: bool,
    pub path: String,
}

// Get a remote package and cache it locally
// in the YAK_HOME version package src directory
pub fn get(args: &mut GetArgs) -> Result<YakPkg> {
    info!("get args: {:?}", &args);

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
    let local_pkgfile = format!("{}/yak.pkg", pkg_local_path);
    download_file(&remote_pkgfile, &local_pkgfile)?;

    // parse the package file
    let mut pkgfile = PathBuf::from(pkg_local_path);
    pkgfile.push("yak.pkg");
    let mut pkg_ast = Ast::from_file(pkgfile)?;
    pkg_ast.parse_package()?;
    let parsed_pkg = pkg_ast.parsed.package.clone();

    // convert AST => yak_package
    let mut yak_pkg = parsed_pkg.into_yak_package(
        args.pkg_root,
        None,
        pkg_local_path.to_string(),
        Some(remote_path),
    )?;
    info!("yak_pkg: {:?}", yak_pkg);

    // create YakPkg
    let mut pkg = YakPkg::default();

    // cache dependencies
    let dep_urls = yak_pkg.get_remote_dep_urls()?;
    info!("get remote dependencies: {:#?}", &dep_urls);
    for dep in dep_urls {
        // reuse args
        args.pkg_root = false;
        args.path = dep.1.into();
        let _pkg = get(args)?;
        pkg.deps.insert(dep.0, _pkg);
    }

    // get remote files...
    yak_pkg.get_remote_files()?;

    // parse local package files...
    for pkg_file in &yak_pkg.pkg_files {
        pkg_ast.parse_file(PathBuf::from(&pkg_file.local_path))?;
    }

    info!("=============");
    info!("{:#?}", &pkg_ast);
    info!("=============");

    // store
    pkg.pkg = yak_pkg;
    pkg.ast = Some(pkg_ast);

    Ok(pkg)
}

// BuildArgs
#[derive(Debug, Default)]
pub struct BuildArgs {
    pub pkg_root: bool,
    pub pkg_as_pkg_id: Option<String>,
    pub path: String,
}

pub fn build(args: BuildArgs) -> Result<YakPkg> {
    info!("build args: {:?}", args);

    let yak_home = YakHome::default();
    yak_home.create_home_dir()?;

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
    pkg_ast.parse_package()?;
    let parsed_pkg = pkg_ast.parsed.package.clone();

    // convert AST => yak_package
    let mut pkg_local_path = abs_path.into_os_string().into_string().unwrap();
    if pkg_local_path.ends_with("/yak.pkg") {
        pkg_local_path = pkg_local_path.replace("/yak.pkg", "");
    }

    // we only build local packages
    let mut yak_pkg = parsed_pkg.into_yak_package(
        args.pkg_root,
        args.pkg_as_pkg_id.clone(),
        pkg_local_path,
        None,
    )?;
    info!("======PKG======");
    info!("{:#?}", yak_pkg);
    info!("===============");

    // create YakPkg
    let mut pkg = YakPkg::default();

    // cache dependencies here
    let remote_dep_urls = yak_pkg.get_remote_dep_urls()?;
    info!("get remote dependencies: {:#?}", &remote_dep_urls);
    for dep in remote_dep_urls {
        // should get all remote deps
        let mut get_args = GetArgs {
            path: dep.1.into(),
            pkg_root: false,
        };
        let _pkg = get(&mut get_args)?;
    }

    // should build all dependencies
    // and skip the compiler phase...
    let build_dep_paths = yak_pkg.get_local_dep_paths()?;
    info!("build dependency paths: {:#?}", &build_dep_paths);
    for dep in build_dep_paths {
        let build_args = BuildArgs {
            pkg_as_pkg_id: Some(dep.0.clone()),
            pkg_root: false,
            path: dep.1.into_os_string().into_string().unwrap(),
        };
        let _pkg = build(build_args)?;
        pkg.deps.insert(dep.0, _pkg);
    }

    // build src code
    let pkg_files = yak_pkg.get_local_file_paths()?;
    for pkg_file in pkg_files.into_iter() {
        pkg_ast.parse_file(pkg_file)?;
    }
    info!("======AST======");
    info!("{:#?}", &pkg_ast);
    info!("===============");

    let mut hir = Hir::default();

    // Parse pkg ast
    hir.from_ast(args.pkg_root, args.pkg_as_pkg_id, &pkg_ast)?;

    info!("======HIR======");
    info!("{:#?}", &hir);
    info!("===============");

    // We only compile the pkg root
    if args.pkg_root {
        // Iterate all pkg.deps and merge modules
        merge_dep_hir_modules(&pkg, &mut hir);
        // Compiler options
        let pkg_local_path = yak_pkg.pkg_local_path.clone();
        let output_dir = format!("{}/target", &pkg_local_path);
        let compiler_opts = CompilerOpts {
            pkg_id: yak_pkg.pkg_id.clone(),
            pkg_local_path: yak_pkg.pkg_local_path.clone(),
            output_dir: output_dir,
        };
        // Build package
        Compiler::build(compiler_opts, hir.clone())?;
    }

    // Pack pkg
    pkg.pkg = yak_pkg;
    pkg.ast = Some(pkg_ast);
    pkg.hir = Some(hir);

    Ok(pkg)
}

fn merge_dep_hir_modules(pkg: &YakPkg, hir: &mut Hir) {
    for (_as_pkg_id, pkg_dep) in pkg.deps.iter() {
        if let Some(pkg_dep_hir) = &pkg_dep.hir {
            hir.merge_modules(pkg_dep_hir);
        }
        merge_dep_hir_modules(pkg_dep, hir)
    }
}
