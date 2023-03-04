mod error;
mod manager;
mod replacer;

pub(self) use self::replacer::Replacer;
pub use self::{
    error::{Error, Result},
    manager::{BufferManager, BufferRef},
};
use {
    crate::PageNum,
    common::pub_fields_struct,
    def::{
        meta::{TABLESPACE_ID_DEFAULT, TABLESPACE_ID_GLOBAL},
        DatabaseId, TableId, TableSpaceId,
    },
    std::path::PathBuf,
};

pub_fields_struct! {
    #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
    struct FileNode {
        space_id: TableSpaceId,
        database_id: DatabaseId,
        table_id: TableId,
    }

    #[derive(Debug, Clone, Eq, Hash, PartialEq)]
    struct PageTag {
        file_node: FileNode,
        page_num: PageNum,
    }
}

impl FileNode {
    pub fn new(space_id: TableSpaceId, database_id: DatabaseId, table_id: TableId) -> Self {
        Self {
            space_id,
            database_id,
            table_id,
        }
    }

    pub fn global_meta(table_id: TableId) -> Self {
        Self {
            space_id: TABLESPACE_ID_GLOBAL,
            database_id: 0,
            table_id,
        }
    }

    pub fn file_path(&self) -> PathBuf {
        match self.space_id {
            TABLESPACE_ID_GLOBAL => PathBuf::from("global"),
            TABLESPACE_ID_DEFAULT => PathBuf::from(format!("base/{}", self.table_id)),
            _ => PathBuf::from(format!("tablespace/{}", self.space_id)),
        }
        .join(self.table_id.to_string())
    }
}

pub(crate) type BufferId = usize;
