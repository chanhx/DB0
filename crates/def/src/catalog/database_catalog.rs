use super::{error::Result, table_schema::TableSchema, TableId};

pub trait DatabaseCatalog {
    fn create_table(&mut self, table: TableSchema) -> Result<TableId>;
    fn delete_table(&mut self, id: TableId) -> Result<()>;
    fn get_table_id(&self, name: &str) -> Option<TableId>;
    fn get_table_schema(&self, id: TableId) -> Result<&TableSchema>;
    fn update_table_schema(&mut self, id: TableId, schema: TableSchema) -> Result<()>;
}
