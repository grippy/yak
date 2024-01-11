use super::name::Name;

#[derive(Debug, Clone, PartialEq)]
pub struct FieldId {
    field_name: String,
    field_num: usize,
}

impl FieldId {
    pub fn new(field_name: String, field_num: usize) -> Self {
        FieldId {
            field_name,
            field_num,
        }
    }
}

impl Name for FieldId {
    fn name(&self) -> String {
        format!("{}", &self.field_name)
    }
}
