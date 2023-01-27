use core::{cmp::Ordering, mem::size_of};

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    TinyInt(i8),
    SmallInt(i16),
    Int(i32),
    BigInt(i64),
    TinyUint(u8),
    SmallUint(u16),
    Uint(u32),
    BigUint(u64),
    Float(f32),
    Double(f64),
    String(String),
}

impl Value {
    pub fn byte_count(&self) -> usize {
        match self {
            Self::Null => 0,
            Self::Boolean(_) => size_of::<bool>(),
            Self::TinyInt(_) => size_of::<i8>(),
            Self::SmallInt(_) => size_of::<i16>(),
            Self::Int(_) => size_of::<i32>(),
            Self::BigInt(_) => size_of::<i64>(),
            Self::TinyUint(_) => size_of::<u8>(),
            Self::SmallUint(_) => size_of::<u16>(),
            Self::Uint(_) => size_of::<u32>(),
            Self::BigUint(_) => size_of::<u64>(),
            Self::Float(_) => size_of::<f32>(),
            Self::Double(_) => size_of::<f64>(),
            Self::String(s) => s.as_bytes().len(),
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Null, _) => Ordering::Less,
            (_, Self::Null) => Ordering::Greater,

            (Self::Boolean(v1), Self::Boolean(v2)) => v1.cmp(v2),
            (Self::TinyInt(v1), Self::TinyInt(v2)) => v1.cmp(v2),
            (Self::SmallInt(v1), Self::SmallInt(v2)) => v1.cmp(v2),
            (Self::Int(v1), Self::Int(v2)) => v1.cmp(v2),
            (Self::BigInt(v1), Self::BigInt(v2)) => v1.cmp(v2),
            (Self::TinyUint(v1), Self::TinyUint(v2)) => v1.cmp(v2),
            (Self::SmallUint(v1), Self::SmallUint(v2)) => v1.cmp(v2),
            (Self::Uint(v1), Self::Uint(v2)) => v1.cmp(v2),
            (Self::BigUint(v1), Self::BigUint(v2)) => v1.cmp(v2),
            (Self::Float(v1), Self::Float(v2)) => v1.total_cmp(v2),
            (Self::Double(v1), Self::Double(v2)) => v1.total_cmp(v2),

            // (Self::TinyInt(v1), Self::SmallInt(v2)) | (Self::SmallInt(v2), Self::TinyInt(v1)) => {
            //     (*v1 as i16).cmp(v2)
            // }
            // (Self::TinyInt(v1), Self::Int(v2)) | (Self::Int(v2), Self::TinyInt(v1)) => {
            //     (*v1 as i32).cmp(v2)
            // }
            // (Self::TinyInt(v1), Self::BigInt(v2)) | (Self::BigInt(v2), Self::TinyInt(v1)) => {
            //     (*v1 as i64).cmp(v2)
            // }
            (Self::String(v1), Self::String(v2)) => v1.cmp(v2),

            (v1, v2) => panic!(
                "Cannot compare values of different types: {:?}, {:?}",
                v1, v2
            ),
        }
    }
}

pub type Row = Vec<Value>;
