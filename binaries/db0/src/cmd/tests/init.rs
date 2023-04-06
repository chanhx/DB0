use {
    crate::cmd::{create_meta_tables, Error},
    access::{BTree, Codec},
    def::{
        meta::{self, MetaTable},
        storage::{Decoder, Encoder},
        Value,
    },
    storage::{
        buffer::{BufferManager, FileNode},
        DEFAULT_PAGE_SIZE,
    },
    tempfile::tempdir,
};

#[cfg(test)]
#[test]
fn it_works() -> Result<(), Error> {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    create_meta_tables(path).unwrap();

    let manager = BufferManager::new(100, DEFAULT_PAGE_SIZE, path.to_path_buf());

    let file_node = FileNode::new(meta::TABLESPACE_ID_DEFAULT, 1, meta::Table::TABLE_ID);

    let (key_codec, values_codec) = {
        let mut columns = meta::Table::columns();
        let v_columns = columns.split_off(1);
        let k_columns = columns;

        (Codec::new(k_columns), Codec::new(v_columns))
    };

    let btree = BTree::new(key_codec, values_codec.max_size(), file_node, &manager);

    let key = vec![Value::Uint(meta::Column::TABLE_ID)];
    let (mut cursor, is_matched) = btree.cursor(&key).unwrap().unwrap();

    let (_, values) = cursor.next().unwrap();

    assert!(is_matched);

    let (values, _) = values_codec.decode(&values).unwrap();
    let table = meta::Table::try_from([key, values].concat()).unwrap();

    temp_dir.close().unwrap();

    assert_eq!(table, meta::Column::table());

    Ok(())
}
