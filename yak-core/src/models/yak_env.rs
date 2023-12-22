use std::env;

// get env variable w/ default
fn get(key: &'static str, default: &'static str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}

// normalize version and remove (prefixed)
fn normalize_version(mut ver: String) -> String {
    if ver.starts_with("v") {
        ver.remove(0);
    }
    ver
}

#[derive(Debug)]
pub struct YakEnv {
    pub yak_home: String,
    pub yak_version: String,
}

impl Default for YakEnv {
    fn default() -> Self {
        YakEnv {
            yak_home: get("YAK_HOME", "~/.yak"),
            yak_version: normalize_version(get("YAK_VERSION", "0.0.0")),
        }
    }
}

impl YakEnv {
    pub fn cwd(&self) -> Result<std::path::PathBuf, std::io::Error> {
        env::current_dir()
    }
}
