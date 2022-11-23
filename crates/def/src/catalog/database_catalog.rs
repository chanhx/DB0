use {
    super::{error::Result, CatalogId, Table, TableId},
    crate::DataType,
    common::pub_fields_struct,
};

pub trait DatabaseCatalog {
    type T: Table;

    fn id(&self) -> CatalogId;
    fn create_table(&mut self, args: CreateTableArgs) -> Result<TableId>;
    fn delete_table(&mut self, id: TableId) -> Result<()>;
    fn get_table(&self, name: &str) -> Result<&Self::T>;
    // fn update_table(&mut self, args: UpdateTableArgs) -> Result<()>;
}

pub_fields_struct! {
    #[derive(Debug)]
    struct ColumnDef {
        name: String,
        data_type: DataType,
        is_nullable: bool,
        comment: Option<String>,
        // default_value:
    }

    #[derive(Debug)]
    struct UniqueConstraint {
        name: String,
        columns: Vec<String>,
    }

    #[derive(Debug)]
    struct CreateTableArgs {
        if_not_exists: bool,
        name: String,
        columns: Vec<ColumnDef>,
        primary_key_columns: Option<Vec<String>>,
        unique_contraints: Vec<UniqueConstraint>,
    }
}
