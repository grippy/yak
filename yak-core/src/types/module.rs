use super::name::Name;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ModuleId {
    pub pkg_name: String,
}

impl ModuleId {
    pub fn new(pkg_name: String) -> Self {
        ModuleId { pkg_name }
    }
}

impl Name for ModuleId {
    fn name(&self) -> String {
        format!("{}", self.pkg_name)
    }
}
