mod column;
mod database;
pub mod error;
mod schema;
mod table;
mod tablespace;

use {
    crate::{
        meta::error::{Error, Result},
        DataType, DatabaseId, SchemaId, SqlType, TableId, TableSpaceId, Value,
    },
    snafu::prelude::*,
};
pub use {column::*, database::*, schema::*, table::*, tablespace::*};

#[repr(u32)]
enum MetaTableId {
    Tablespace = 1,
    Database,
    Schema,
    Table,
    Column,
}

pub trait MetaTable {
    const TABLE_ID: TableId;

    fn table() -> Table;
    fn columns() -> Vec<Column>;
}

macro_rules! meta_table_struct {
    (@value_for_type Varchar, $prop:ident) => {
        Value::String($prop)
    };
    (@value_for_type Char, $prop:ident) => {
        Value::String($prop)
    };
    (@value_for_type $ty:tt, $prop:ident) => {
        Value::$ty($prop)
    };

    (@type_cast $prop:ident, SqlType) => {
        $prop.try_into().context(error::TypeEncodingSnafu)?
    };
    (@type_cast $prop:ident, $ty:ty) => {
        $prop
    };

    (
        $(
            $(#[$meta:meta])*
            struct $name:ident {
                $($prop:ident: ($ty:tt, DataType::$sql_ty:tt$(($constraint:literal))?),)*
            }
        )*
    ) => {

        $(
            $(#[$meta])*
            pub struct $name {
                $(pub $prop: $ty,)*
            }

            impl $name {
                pub fn new($($prop: $ty),*) -> Self {
                    Self {
                        $($prop,)*
                    }
                }
            }

            impl MetaTable for $name {
                const TABLE_ID: TableId = MetaTableId::$name as u32;

                fn table() -> Table {
                    Table {
                        id: MetaTableId::$name as TableId,
                        name: stringify!($name).to_string().to_lowercase(),
                        schema_id: SCHEMA_ID_META,
                    }
                }

                fn columns() -> Vec<Column> {
                    vec![
                        $(
                            Column {
                                table_id: Self::TABLE_ID as u32,
                                num: ${index()} + 1,
                                name: stringify!($prop).to_string(),
                                type_id: DataType::$sql_ty$(($constraint))?.value_repr().0,
                                type_len: DataType::$sql_ty$(($constraint))?.value_repr().1,
                                is_nullable: false,
                            },
                        )*
                    ]
                }
            }

            impl From<$name> for Vec<Value> {
                fn from(obj: $name) -> Self {
                    vec![
                        $(obj.$prop.into(),)*
                    ]
                }
            }

            impl TryFrom<Vec<Value>> for $name {
                type Error = Error;

                fn try_from(values: Vec<Value>) -> Result<Self> {
                    let values: [Value; ${count(prop)}] = values
                        .try_into()
                        .map_err(|_| error::InternalSnafu.build())?;

                    match values {
                        [$(meta_table_struct!(@value_for_type $sql_ty, $prop),)*] => {
                            Ok(Self::new(
                                $(meta_table_struct!(@type_cast $prop, $ty),)*
                            ))
                        }
                        _ => Err(error::InternalSnafu.build()),
                    }
                }
            }
        )*
    };
}

meta_table_struct! {
    #[derive(Debug, Clone, PartialEq)]
    struct Database {
        id: (DatabaseId, DataType::Uint),
        name: (String, DataType::Varchar(50)),
        space_id: (TableSpaceId, DataType::Uint),
    }

    #[derive(Debug, Clone)]
    struct Column {
        table_id: (TableId, DataType::Uint),
        num: (i16, DataType::SmallInt),
        name: (String, DataType::Varchar(50)),
        type_id: (SqlType, DataType::SmallUint),
        type_len: (u16, DataType::SmallUint),
        is_nullable: (bool, DataType::Boolean),
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Schema {
        id: (SchemaId, DataType::Uint),
        name: (String, DataType::Varchar(50)),
        space_id: (TableSpaceId, DataType::Uint),
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Table {
        id: (TableId, DataType::Uint),
        name: (String, DataType::Varchar(50)),
        schema_id: (SchemaId, DataType::Uint),
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Tablespace {
        id: (TableSpaceId, DataType::Uint),
        name: (String, DataType::Varchar(50)),
    }
}
