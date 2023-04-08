use {
    super::{Error, Evaluatate},
    def::{ColumnId, DataType, Value},
};

#[derive(Debug)]
pub struct ColumnRef {
    column_id: ColumnId,
    data_type: DataType,
}

impl Evaluatate for ColumnRef {
    fn return_type(&self) -> DataType {
        self.data_type.clone()
    }

    fn evaluate(&self, values: &[Value]) -> Result<Value, Error> {
        // SAFETY: the validity of index should have been checked by binder before evaluating
        unsafe { Ok(values.get_unchecked(self.column_id as usize).clone()) }
    }
}
