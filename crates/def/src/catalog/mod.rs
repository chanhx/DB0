mod catalog;
mod database_catalog;
mod error;
mod table_schema;

pub use {
    catalog::Catalog,
    database_catalog::DatabaseCatalog,
    error::{Error, Result},
    table_schema::{Column, TableSchema, UniqueConstraint},
};

pub type DatabaseId = u32;
pub type TableId = u32;
pub type ColumnId = u32;
