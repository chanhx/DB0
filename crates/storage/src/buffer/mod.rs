mod error;
mod manager;
mod replacer;

pub use self::{
    error::{Error, Result},
    manager::{BufferManager, BufferRef},
};

pub(self) use self::replacer::Replacer;

use {
    crate::btree::PageNum,
    common::pub_fields_struct,
    def::{
        catalog::{CatalogId, TableId},
        tablespace::{self, TableSpaceId},
    },
    std::path::PathBuf,
};

pub_fields_struct! {
    #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
    struct FileNode {
        space_id: TableSpaceId,
        catalog_id: CatalogId,
        table_id: TableId,
    }

    #[derive(Debug, Clone, Eq, Hash, PartialEq)]
    struct PageTag {
        file_node: FileNode,
        page_num: PageNum,
    }
}

impl FileNode {
    pub fn new(space_id: TableSpaceId, catalog_id: CatalogId, table_id: TableId) -> Self {
        Self {
            space_id,
            catalog_id,
            table_id,
        }
    }

    pub fn global_meta(table_id: TableId) -> Self {
        Self {
            space_id: tablespace::GLOBAL_TABLESPACE_ID,
            catalog_id: 0,
            table_id,
        }
    }

    pub fn file_path(&self) -> PathBuf {
        match self.space_id {
            tablespace::GLOBAL_TABLESPACE_ID => PathBuf::from("global"),
            tablespace::DEFAULT_TABLESPACE_ID => PathBuf::from(format!("base/{}", self.table_id)),
            _ => PathBuf::from(format!("tablespace/{}", self.space_id)),
        }
        .join(self.table_id.to_string())
    }
}

pub(crate) type BufferId = usize;
