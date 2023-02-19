use crate::SchemaId;

pub const SCHEMA_ID_META: SchemaId = 1;
pub const SCHEMA_ID_PUBLIC: SchemaId = 2;

// meta_table_struct! {
//     struct Schema {
//         id: (SchemaId, DataType::Uint),
//         name: (String, DataType::Varchar(50)),
//         space_id: (TableSpaceId, DataType::Uint),
//     }
// }
