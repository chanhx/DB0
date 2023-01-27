mod codec;

use crate::{catalog::TableId, Row, Value};
pub use codec::{Decoder, Encoder};

pub trait Storage {
    type Error: std::error::Error;

    fn create(
        &mut self,
        table_id: TableId,
        id: Option<Value>,
        row: Row,
    ) -> Result<Value, Self::Error>;

    fn read(&self, table_id: TableId, id: Value) -> Result<Row, Self::Error>;
    // fn update(&mut self, table_id: TableId, id: Value, )
    fn delete(&mut self, table_id: TableId, id: Value) -> Result<(), Self::Error>;
}
