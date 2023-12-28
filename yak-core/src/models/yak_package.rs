use crate::models::yak_version::YakVersion;
use crate::utils::{download_file, normalize_path};
use anyhow::Result;
use log::info;
use std::path::Path;
use url::Url;

#[derive(Debug, Default)]
pub struct YakPackage {
    pub pkg_root: bool,
    pub pkg_id: String,
    pub pkg_local_path: String,
    pub pkg_remote_path: Option<String>,
    pub pkg_version: YakVersion,
    pub pkg_description: String,
    pub pkg_files: Vec<YakFile>,
    pub pkg_dependencies: Vec<YakDependency>,
    pub pkg_imports: Vec<YakImport>,
    pub pkg_exports: YakExport,
    // TODO: yak version semver requirement
    pub yak_version: YakVersion,
}

impl YakPackage {
    // iterate all dependencies and cache
    // them in the pkg_local_path directory
    pub fn get_remote_deps(&mut self) {}

    // iterate all files and cache
    // them in the pkg_local_path directory
    pub fn get_remote_files(&mut self) -> Result<()> {
        if self.pkg_remote_path.is_none() {
            return Ok(());
        }
        let pkg_local_path = self.pkg_local_path.clone();
        let pkg_remote_path = self.pkg_remote_path.clone().unwrap();
        for pkg_file in &mut self.pkg_files {
            pkg_file.get_remote_file(pkg_local_path.clone(), pkg_remote_path.clone())?
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct YakFile {
    pub path: String,
    pub local_path: String,
    pub remote_path: String,
    pub src: String,
}

impl YakFile {
    fn get_remote_file(
        &mut self,
        pkg_local_path: String,
        mut pkg_remote_path: String,
    ) -> Result<()> {
        if !pkg_remote_path.ends_with("/") {
            pkg_remote_path += "/";
        }
        let pkg_remote_url = Url::parse(&pkg_remote_path)?;
        let pkg_remote_file = pkg_remote_url.join(&self.path)?;
        let pkg_local_file = normalize_path(Path::new(
            format!("{}/{}", pkg_local_path, &self.path).as_str(),
        ))
        .display()
        .to_string();
        // save remote and local paths
        self.remote_path = pkg_remote_file.to_string();
        self.local_path = pkg_local_file.clone();
        info!("get file {}", &self.remote_path);
        info!("write file {}", &self.local_path);
        download_file(&pkg_remote_file.to_string(), &pkg_local_file)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct YakDependency {
    pub pkg_id: String,
    pub path: String,
}

#[derive(Debug, Default)]
pub struct YakFeature {
    pub name: String,
    pub enabled: bool,
    pub files: Vec<YakFile>,
    pub dependencies: Vec<YakDependency>,
}

#[derive(Debug, Default)]
pub struct YakImport {
    pub pkg_id: String,
    pub as_pkg_id: Option<String>,
    pub symbols: Vec<YakSymbol>,
}

#[derive(Debug, Default)]
pub struct YakExport {
    pub symbols: Vec<YakSymbol>,
}

#[derive(Debug)]
pub enum Symbol {
    None,
    Var(String),
    Func(String),
    Type(String),
    Trait(String),
}

impl Default for Symbol {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default)]
pub struct YakSymbol {
    // identity: Type, ^Trait, :fn, const, etc.
    pub symbol: Symbol,
    // `as` renames the import/export type
    pub as_symbol: Option<Symbol>,
}
