use {
    super::error::{self, Result},
    access::{BTree, Codec},
    def::{
        meta::{self, MetaTable},
        storage::Encoder,
        DatabaseId, Value,
    },
    snafu::prelude::*,
    std::{fs, path::Path},
    storage::{
        buffer::{BufferManager, FileNode},
        DEFAULT_PAGE_SIZE,
    },
};

pub fn create_meta_tables(data_dir: &Path) -> Result<()> {
    let capacity = 100;
    let manager = BufferManager::new(capacity, DEFAULT_PAGE_SIZE, data_dir.to_path_buf());

    let database_id = 1;

    let file_node = FileNode::global_meta(meta::Database::TABLE_ID);
    create_directory(data_dir, &file_node)?;
    create_global_tables(&manager, database_id, file_node)?;

    let file_node = FileNode::new(meta::TABLESPACE_ID_DEFAULT, database_id, 1);
    create_directory(data_dir, &file_node)?;
    init_database(&manager, database_id)?;

    manager.flush_pages().context(error::StorageSnafu)?;

    Ok(())
}

fn create_directory(data_dir: &Path, file_node: &FileNode) -> Result<()> {
    let mut path = data_dir.to_path_buf();
    path.extend(file_node.file_path().parent().unwrap());

    fs::create_dir_all(&path).unwrap();

    Ok(())
}

fn create_global_tables(
    manager: &BufferManager,
    database_id: DatabaseId,
    file_node: FileNode,
) -> Result<()> {
    let db = meta::Database {
        id: database_id,
        name: "test".to_string(),
        space_id: meta::TABLESPACE_ID_GLOBAL,
    };
    let mut kv: Vec<Value> = db.into();
    let values = kv.split_off(1);
    let key = kv;

    let mut columns = meta::Database::columns();
    let v_columns = columns.split_off(1);
    let k_columns = columns;
    let key_codec = Codec::new(k_columns);

    BTree::<Codec>::init(file_node, manager).context(error::AccessSnafu)?;
    let mut btree = BTree::new(key_codec, 30, file_node, manager);

    let values_codec = Codec::new(v_columns);
    let values = values_codec.encode(&values).unwrap();

    btree.insert(&key, &values).unwrap();

    Ok(())
}

fn init_database(manager: &BufferManager, database_id: DatabaseId) -> Result<()> {
    {
        let tables = [meta::Table::table(), meta::Column::table()];

        let file_node = FileNode::new(
            meta::TABLESPACE_ID_DEFAULT,
            database_id,
            meta::Table::TABLE_ID,
        );

        let (key_codec, values_codec) = {
            let mut columns = meta::Table::columns();
            let v_columns = columns.split_off(1);
            let k_columns = columns;

            (Codec::new(k_columns), Codec::new(v_columns))
        };

        BTree::<Codec>::init(file_node, manager).context(error::AccessSnafu)?;
        let mut btree = BTree::new(key_codec, 30, file_node, manager);

        for table in tables {
            let mut kv: Vec<Value> = table.into();
            let values = kv.split_off(1);
            let key = kv;

            let values = values_codec.encode(&values).unwrap();
            btree.insert(&key, &values).unwrap();
        }
    }

    {
        let columns = [meta::Table::columns(), meta::Column::columns()];

        let file_node = FileNode::new(
            meta::TABLESPACE_ID_DEFAULT,
            database_id,
            meta::Column::TABLE_ID,
        );

        let (key_codec, values_codec) = {
            let mut columns = meta::Column::columns();
            let v_columns = columns.split_off(2);
            let k_columns = columns;

            (Codec::new(k_columns), Codec::new(v_columns))
        };

        BTree::<Codec>::init(file_node, manager).context(error::AccessSnafu)?;
        let mut btree = BTree::new(key_codec, 30, file_node, manager);

        columns.into_iter().flatten().for_each(|column| {
            let mut kv: Vec<Value> = column.into();
            let values = kv.split_off(2);
            let key = kv;

            let values = values_codec.encode(&values).unwrap();
            btree.insert(&key, &values).unwrap();
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use {super::*, def::storage::Decoder, tempfile::tempdir};

    #[test]
    fn it_works() -> Result<()> {
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

        let btree = BTree::new(key_codec, 100, file_node, &manager);

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
}
