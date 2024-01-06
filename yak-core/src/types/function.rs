use super::name::Name;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionId {
    pkg_name: String,
    struct_name: Option<String>,
    func_name: String,
    is_main: bool,
}

impl FunctionId {
    pub fn new(pkg_name: String, struct_name: Option<String>, func_name: String) -> Self {
        let is_main = func_name == ":main" && struct_name == None;

        FunctionId {
            pkg_name,
            struct_name,
            func_name,
            is_main,
        }
    }
}

impl Name for FunctionId {
    fn name(&self) -> String {
        if self.struct_name.is_some() {
            let struct_name = self.struct_name.clone().unwrap();
            format!("{}#{}{}", &self.pkg_name, &struct_name, &self.func_name)
        } else {
            format!("{}{}", &self.pkg_name, &self.func_name)
        }
    }
}
