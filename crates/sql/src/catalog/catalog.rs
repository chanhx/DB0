use {
    super::{database_catalog::DatabaseCatalog, DatabaseId},
    crate::error::Result,
};

pub trait Catalog {
    fn create_database(&mut self, name: &str) -> Result<DatabaseId>;
    fn delete_database(&mut self, id: DatabaseId) -> Result<()>;
    fn get_database_id(&self, name: &str) -> Option<DatabaseId>;
    fn get_database_catalog<D: DatabaseCatalog>(&self, id: DatabaseId) -> Option<D>;
}
