use crate::TableSpaceId;

pub const TABLESPACE_ID_DEFAULT: TableSpaceId = 0;
pub const TABLESPACE_ID_GLOBAL: TableSpaceId = 1;

// meta_table_struct! {
//     struct Tablespace {
//         id: (TableSpaceId, DataType::Uint),
//         name: (String, DataType::Varchar(50)),
//     }
// }
