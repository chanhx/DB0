mod common;

pub(crate) use common::DataType;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    CreateDatabase { if_not_exists: bool, name: String },
}
