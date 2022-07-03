#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataType {
    Boolean,

    // Numeric types
    Bigint,
    Decimal,
    Float,
    Integer,
    SmallInt,

    // String types
    Char(u32),
    Varchar(u32),
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Bigint(i128),
    Integer(i32),
    SmallInt(i8),
    Float(f32),
    Decimal(f64),
    Char(String),
}

pub type Row = Vec<Value>;
