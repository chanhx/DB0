mod error;
mod lru_replacer;
mod manager;
mod page;
mod pool;
mod replacer;

pub(crate) use self::{
    error::{Error, Result},
    lru_replacer::LruReplacer,
    manager::BufferManager,
    page::Page,
    replacer::Replacer,
};

use {super::PageNum, common::pub_fields_struct, def::catalog::TableId, std::path::PathBuf};

pub_fields_struct! {
    #[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
    struct FileNode {
        table_id: TableId,
    }

    #[derive(Debug, Clone, Eq, Hash, PartialEq)]
    struct PageTag {
        file_node: FileNode,
        page_num: PageNum,
    }
}

impl FileNode {
    fn file_path(&self) -> PathBuf {
        PathBuf::from(format!("./{}", self.table_id))
    }
}

pub(crate) type BufferId = usize;
