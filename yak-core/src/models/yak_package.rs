use crate::models::yak_version::YakVersion;
use std::collections::HashMap;

#[derive(Debug)]
pub struct YakPackage {
    package_id: String,
    package_path: String,
    description: String,
    yak_version: YakVersion,
    version: YakVersion,
    files: Vec<YakFile>,
    features: HashMap<String, YakFeature>,
    dependencies: Vec<YakDependency>,
    imports: Vec<YakImport>,
    exports: Vec<YakExport>,
}

#[derive(Debug)]
pub struct YakFile {
    path: String,
    source: String,
}

#[derive(Debug)]
pub struct YakDependency {
    package_id: String,
    package_path: String,
}

#[derive(Debug)]
pub struct YakFeature {
    name: String,
    enabled: bool,
    files: Vec<YakFile>,
    dependencies: Vec<YakDependency>,
}

#[derive(Debug)]
pub struct YakImport {
    package_id: String,
    symbols: Vec<YakSymbol>,
}

#[derive(Debug)]
pub struct YakExport {
    // package_id is optional
    // if omitted we're exporting "local" symbols
    // otherwise, we're re-exporting symbols from a dependency
    package_id: Option<String>,
    symbols: Vec<YakSymbol>,
}

#[derive(Debug)]
pub struct YakSymbol {
    // identity: Type, ^Trait, :fn, const, etc.
    id: String,
    // `as` renames the import type
    as_: Option<String>,
}
