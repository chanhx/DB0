mod error;

pub use error::{Error, Result};

use crate::storage::Storage;

pub type TransactionId = u32;

pub trait TransactionProcessor: Storage {
    fn begin(&mut self) -> Result<TransactionId>;
    fn commit(&mut self, id: TransactionId) -> Result<()>;
    fn rollback(&mut self, id: TransactionId) -> Result<()>;
}
