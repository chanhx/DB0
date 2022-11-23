use super::{database_catalog::DatabaseCatalog, error::Result, CatalogId};

pub trait Catalog {
    type D: DatabaseCatalog;

    fn create_database(&mut self, name: &str) -> Result<CatalogId>;
    fn delete_database(&mut self, id: CatalogId) -> Result<()>;
    fn get_database_catalog(&self, name: &str) -> Option<Self::D>;
}
