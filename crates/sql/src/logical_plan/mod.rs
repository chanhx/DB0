mod create_table;
mod planner;

pub use planner::Planner;

use crate::catalog::TableSchema;

#[derive(Debug)]
pub enum Node {
    CreateDatabase {
        if_not_exists: bool,
        name: String,
    },
    CreateTable {
        if_not_exists: bool,
        schema: TableSchema,
    },
}
