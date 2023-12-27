// `YakVersion` provides semantic versioning support
#[derive(Debug, Default)]
pub struct YakVersion {
    // semver
    pub version: String,
}

impl YakVersion {
    pub fn new(version: String) -> Self {
        Self { version }
    }
}
