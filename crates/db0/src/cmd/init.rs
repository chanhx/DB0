use {
    super::error::{self, Result},
    def::catalog::META_TABLE_CATALOGS,
    snafu::ResultExt,
    std::{fs, num::NonZeroUsize, path::PathBuf},
    storage::{
        btree::BTree,
        buffer::{BufferManager, FileNode, LruReplacer},
    },
};

pub fn create_global_meta_tables(data_dir: PathBuf) -> Result<()> {
    let size = NonZeroUsize::new(10).unwrap();
    let replacer = LruReplacer::new(size);
    let mut manager = BufferManager::new(size, replacer, data_dir);

    let file_node = FileNode::global_meta(META_TABLE_CATALOGS);
    let file_dir = file_node.file_path();

    fs::create_dir_all(file_dir.parent().unwrap()).unwrap();

    let _btree = BTree::new(&mut manager, 30, 0, 0, file_node)
        .map_err(Into::into)
        .context(error::StorageSnafu)?;

    manager
        .flush_pages()
        .map_err(Into::into)
        .context(error::StorageSnafu)
}
