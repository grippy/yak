use crate::models::yak_home::YakHome;
use crate::models::yak_version::YakVersion;
use crate::utils::{download_file, normalize_path};
use anyhow::Result;
use log::info;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Debug, Default)]
pub struct YakPackage {
    pub pkg_root: bool,
    // pkg name
    pub pkg_id: String,
    // pkg import as name
    pub pkg_as_pkg_id: Option<String>,
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
    // returns a list of all remote dependency urls
    // - remote path + relative dep.path
    // - remote dep.path
    pub fn get_remote_dep_urls(&mut self) -> Result<Vec<(String, Url)>> {
        let mut urls = vec![];
        if self.pkg_remote_path.is_none() {
            return Ok(urls);
        }
        let mut pkg_remote_path = self.pkg_remote_path.clone().unwrap();
        if !pkg_remote_path.ends_with("/") {
            pkg_remote_path += "/";
        }
        for pkg_dep in self.pkg_dependencies.iter() {
            // default to remote + relative path
            let mut pkg_remote_url = Url::parse(&pkg_remote_path)?;
            pkg_remote_url = pkg_remote_url.join(&pkg_dep.path)?;

            // or use pkg dep path if already remote
            if pkg_dep.path.starts_with("http://") || pkg_dep.path.starts_with("https://") {
                pkg_remote_url = Url::parse(&pkg_dep.path)?;
            }
            urls.push((pkg_dep.pkg_id.clone(), pkg_remote_url))
        }
        Ok(urls)
    }

    // `get_local_dep_paths` assumes all remote packages have already been downloaded
    // and returns all deps as if the src exists locally
    pub fn get_local_dep_paths(&mut self) -> Result<Vec<(String, PathBuf)>> {
        let mut paths = vec![];

        let yak_home = YakHome::default();
        let yak_home_src_path = yak_home
            .get_home_version_src_dir()
            .into_os_string()
            .into_string()
            .unwrap();
        for pkg_dep in self.pkg_dependencies.iter() {
            // convert remote paths to local ones...
            // this assumes the pkg was already downloaded
            let mut pkg_local_path = self.pkg_local_path.clone();
            let mut pkg_dep_path = pkg_dep.path.clone();
            if pkg_dep_path.starts_with("http://") || pkg_dep_path.starts_with("https://") {
                pkg_local_path = yak_home_src_path.clone();
                pkg_dep_path = pkg_dep_path.replace("http://", "");
                pkg_dep_path = pkg_dep_path.replace("https://", "");
            }
            let dep_path = PathBuf::from(format!("{}/{}", &pkg_local_path, &pkg_dep_path));
            let src_path = normalize_path(dep_path.as_path());
            paths.push((pkg_dep.pkg_id.clone(), src_path));
        }
        Ok(paths)
    }

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

    pub fn get_local_file_paths(&mut self) -> Result<Vec<PathBuf>> {
        let paths = self
            .pkg_files
            .iter()
            .map(|file| {
                let path = PathBuf::from(format!("{}/{}", &self.pkg_local_path, &file.path));
                normalize_path(path.as_path())
            })
            .collect();
        Ok(paths)
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
    pub local_path: String,
    pub remote_path: String,
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
    Builtin(String),
    Primitive(String),
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
