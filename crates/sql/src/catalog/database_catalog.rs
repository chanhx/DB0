use {super::table_schema::TableSchema, crate::error::Result};

pub type TableId = u32;

pub trait DatabaseCatalog {
    fn create_table(&mut self, table: TableSchema) -> Result<TableId>;
    fn delete_table(&mut self, id: TableId) -> Result<()>;
    fn get_table_id(&mut self, name: &str) -> Option<TableId>;
    fn get_table_schema(&mut self, id: TableId) -> Result<&TableSchema>;
    fn update_table_schema(&mut self, id: TableId, schema: TableSchema) -> Result<()>;
}
