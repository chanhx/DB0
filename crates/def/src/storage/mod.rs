mod error;

pub use error::{Error, Result};

use crate::{catalog::TableId, Row, Value};

pub trait Storage {
    fn create(&mut self, table_id: TableId, id: Option<Value>, row: Row) -> Result<Value>;
    fn delete(&mut self, table_id: TableId, id: Value) -> Result<()>;
    fn read(&self, table_id: TableId, id: Value) -> Result<Row>;
}
