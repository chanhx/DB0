use super::{database_catalog::DatabaseCatalog, error::Result, DatabaseId};

pub trait Catalog {
    type D: DatabaseCatalog;

    fn create_database(&mut self, name: &str) -> Result<DatabaseId>;
    fn delete_database(&mut self, id: DatabaseId) -> Result<()>;
    fn get_database_catalog(&self, name: &str) -> Option<Self::D>;
}
