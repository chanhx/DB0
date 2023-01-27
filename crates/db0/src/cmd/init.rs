use {super::error::Result, std::path::PathBuf};

pub fn create_global_meta_tables(_data_dir: PathBuf) -> Result<()> {
    // let capacity = 100;
    // let mut manager = BufferManager::new(capacity, DEFAULT_PAGE_SIZE, data_dir);

    // let file_node = FileNode::global_meta(META_TABLE_CATALOGS);
    // let file_dir = file_node.file_path();

    // fs::create_dir_all(file_dir.parent().unwrap()).unwrap();

    // let _btree = BTree::new(&mut manager, 30, 0, 0, file_node)
    //     .map_err(Into::into)
    //     .context(error::StorageSnafu)?;

    // manager
    //     .flush_pages()
    //     .map_err(Into::into)
    //     .context(error::StorageSnafu)

    Ok(())
}
