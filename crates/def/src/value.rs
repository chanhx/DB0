use {
    crate::SqlType,
    core::{cmp::Ordering, mem::size_of},
};

macro_rules! define_value {
    (@byte_count $ident:ident, String) => {
        $ident.as_bytes().len()
    };
    (@byte_count $ident:ident, $raw:ty) => {
        size_of::<$raw>()
    };

    ($($variant:ident($raw:ty),)*) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Value {
            Null,
            $($variant($raw),)*
        }

        impl Value {
            pub fn byte_count(&self) -> usize {
                match self {
                    Self::Null => 0,
                    $(Self::$variant(_v) => define_value!(@byte_count _v, $raw),)*
                }
            }
        }
    };
}

define_value! {
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

macro_rules! value_conversions {
    ($(($raw:ty, $val:ident),)*) => {
        $(
            impl From<$raw> for Value {
                fn from(raw: $raw) -> Self {
                    Value::$val(raw)
                }
            }
        )*
    };
}

value_conversions! {
    (bool, Boolean),
    (i8, TinyInt),
    (i16, SmallInt),
    (i32, Int),
    (i64, BigInt),
    (u8, TinyUint),
    (u16, SmallUint),
    (u32, Uint),
    (u64, BigUint),
    (f32, Float),
    (f64, Double),
    (String, String),
}

impl From<SqlType> for Value {
    fn from(raw: SqlType) -> Self {
        Value::SmallUint(raw as u16)
    }
}
