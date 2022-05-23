use {super::Identifier, crate::common::macros};

#[derive(Debug, PartialEq)]
pub enum SelectItem {}

macros::pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct Select {
        table: Identifier,
        items: Vec<SelectItem>,
    }
}
