use {
    super::{ColumnId, Result, TableId},
    crate::DataType,
    common::pub_fields_struct,
};

pub trait Table {
    fn id(&self) -> TableId;
    fn get_column(&self, name: String) -> Result<Column>;
}

pub_fields_struct! {
    struct Column {
        id: ColumnId,
        name: String,
        data_type: DataType,
        is_nullable: bool,
        comment: Option<String>,
    }
}
