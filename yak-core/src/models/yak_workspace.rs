use crate::models::yak_version::YakVersion;

#[derive(Debug)]
struct YakWorkspace {
    workspace_id: String,
    description: String,
    version: YakVersion,
    packages: Vec<String>,
}
