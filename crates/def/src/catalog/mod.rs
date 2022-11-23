mod catalog;
mod database_catalog;
mod error;
mod meta;
mod table;

pub use {
    catalog::Catalog,
    database_catalog::{ColumnDef, CreateTableArgs, DatabaseCatalog, UniqueConstraint},
    error::{Error, Result},
    meta::*,
    table::Table,
};

pub type CatalogId = u32;
pub type SchemaId = u32;
pub type TableId = u32;
pub type ColumnId = u32;

// pub const META_SCHEMA_ID: SchemaId = 1;
