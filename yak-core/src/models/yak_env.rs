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
    pub home: String,
    pub version: String,
}

impl Default for YakEnv {
    fn default() -> Self {
        YakEnv {
            home: get("YAK_HOME", "~/.yak"),
            version: normalize_version(get("YAK_VERSION", "0.0.0")),
        }
    }
}
