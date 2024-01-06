use super::name::Name;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeId {
    pkg_name: String,
    type_name: String,
}

impl TypeId {
    pub fn new(pkg_name: String, type_name: String) -> Self {
        TypeId {
            pkg_name,
            type_name,
        }
    }
}

impl Name for TypeId {
    fn name(&self) -> String {
        format!("{}#{}", &self.pkg_name, &self.type_name)
    }
}
