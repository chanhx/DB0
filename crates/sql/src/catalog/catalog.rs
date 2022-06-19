use {super::database_catalog::DatabaseCatalog, crate::error::Result};

pub type DatabaseId = u32;

pub trait Catalog {
    fn create_database(&mut self, name: &str) -> Result<DatabaseId>;
    fn delete_database(&mut self, id: DatabaseId) -> Result<()>;
    fn get_database_id(&mut self, name: &str) -> Option<DatabaseId>;
    fn get_database_catalog<D: DatabaseCatalog>(&mut self, id: DatabaseId) -> Option<D>;
}
