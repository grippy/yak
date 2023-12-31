use crate::models::yak_env::YakEnv;
use anyhow::{Context, Result};
use log::info;
use std::fs;
use std::path::{Path, PathBuf};

// YakHome defines the structure of the $YAK_HOME directory
#[derive(Debug)]
pub struct YakHome {
    pub env: YakEnv,
}

impl Default for YakHome {
    fn default() -> Self {
        YakHome {
            env: YakEnv::default(),
        }
    }
}

impl YakHome {
    // ~/.yak
    pub fn get_home_dir(&self) -> PathBuf {
        PathBuf::from_iter([&self.env.yak_home].iter())
    }
    // ~/.yak/yak/v0.0.0
    pub fn get_home_version_dir(&self) -> PathBuf {
        PathBuf::from_iter([&self.env.yak_home, "yak", &self.version_path_part()].iter())
    }
    // ~/.yak/yak/v0.0.0/bin
    pub fn get_home_version_bin_dir(&self) -> PathBuf {
        PathBuf::from_iter([&self.env.yak_home, "yak", &self.version_path_part(), "bin"].iter())
    }
    // ~/.yak/yak/v0.0.0/pkg
    pub fn get_home_version_pkg_dir(&self) -> PathBuf {
        PathBuf::from_iter([&self.env.yak_home, "yak", &self.version_path_part(), "pkg"].iter())
    }
    // ~/.yak/yak/v0.0.0/src
    pub fn get_home_version_src_dir(&self) -> PathBuf {
        PathBuf::from_iter([&self.env.yak_home, "yak", &self.version_path_part(), "src"].iter())
    }
    // prefix v to the version part
    pub fn version_path_part(&self) -> String {
        format!("v{}", &self.env.yak_version)
    }
    // generate a list of paths to create for the YAK_HOME directory
    pub fn paths(&self) -> Vec<PathBuf> {
        let mut paths = vec![self.get_home_dir()];
        // We might not have a yak version enabled
        // so skip creating these paths
        if self.env.yak_version != "0.0.0" {
            paths.append(&mut vec![
                self.get_home_version_dir(),
                self.get_home_version_bin_dir(),
                self.get_home_version_pkg_dir(),
                self.get_home_version_src_dir(),
            ]);
        }
        paths
    }

    pub fn create_home_dir(&self) -> Result<()> {
        for path in &self.paths() {
            if Path::new(path).exists() {
                continue;
            }
            info!("create yak dir: {}", &path.display());
            let _ = fs::create_dir_all(path)
                .with_context(|| format!("failed to create file path: {}", path.display()))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum YakPath {
    Root(PathBuf),
    RootVersion(PathBuf),
    RootVersionBin(PathBuf),
    RootVersionPkg(PathBuf),
    RootVersionSrc(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn _get_epoch() -> u128 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis(),
            Err(_) => panic!("unable to get epoch: SystemTime before UNIX EPOCH!"),
        }
    }

    #[test]
    fn yak_create_home_dir() {
        // let yak_home_dir = format!("/tmp/.yak-{}", get_epoch());
        let yak_home_dir = "/tmp/.yak-static";
        let yak_version = "1.0.0";
        std::env::set_var("YAK_HOME", yak_home_dir);
        std::env::set_var("YAK_VERSION", yak_version);
        let yak_home = YakHome::default();
        println!("paths {:?}", yak_home.paths());
        assert!(
            yak_home.create_home_dir().is_ok(),
            "expected to create yak home dir"
        );
    }
}
