use super::name::Name;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ModuleId {
    pub pkg_name: String,
    pub as_pkg_name: Option<String>,
}

impl ModuleId {
    pub fn new(pkg_name: String, as_pkg_name: Option<String>) -> Self {
        ModuleId {
            pkg_name,
            as_pkg_name,
        }
    }
}

impl Name for ModuleId {
    fn name(&self) -> String {
        if self.as_pkg_name.is_some() {
            format!("{}", self.as_pkg_name.as_ref().unwrap())
        } else {
            format!("{}", self.pkg_name)
        }
    }
}
