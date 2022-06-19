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
