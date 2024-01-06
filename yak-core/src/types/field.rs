use super::name::Name;

#[derive(Debug, Clone, PartialEq)]
pub struct FieldId {
    pkg_name: String,
    struct_name: String,
    field_name: String,
    field_num: usize,
}

impl FieldId {
    pub fn new(
        pkg_name: String,
        struct_name: String,
        field_name: String,
        field_num: usize,
    ) -> Self {
        FieldId {
            pkg_name,
            struct_name,
            field_name,
            field_num,
        }
    }
}

impl Name for FieldId {
    fn name(&self) -> String {
        format!(
            "@{}/{}.{}",
            &self.pkg_name, &self.struct_name, &self.field_name
        )
    }
}
