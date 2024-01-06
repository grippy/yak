use super::name::Name;

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantId {
    pkg_name: String,
    const_name: String,
}

impl ConstantId {
    pub fn new(pkg_name: String, const_name: String) -> Self {
        ConstantId {
            pkg_name,
            const_name,
        }
    }
}

impl Name for ConstantId {
    fn name(&self) -> String {
        format!("{}#{}", &self.pkg_name, &self.const_name)
    }
}
