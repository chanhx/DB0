use {
    crate::Executor,
    access::{btree::BTree, Codec},
    bound_ast::{Column, CreateTableStmt},
    def::{
        meta::{self, MetaTable},
        storage::Encoder,
        TableId, Value,
    },
    snafu::prelude::*,
    storage::buffer::{BufferManager, FileNode},
};

#[derive(Debug, Snafu)]
pub enum Error {
    Access {
        #[snafu(backtrace)]
        source: access::btree::error::Error,
    },

    #[snafu(display("table '{}' already exists", name))]
    TableExists { name: String },
}

type Result<T> = std::result::Result<T, Error>;

impl Executor {
    pub(crate) fn create_table(
        &self,
        stmt: CreateTableStmt,
        manager: &BufferManager,
    ) -> Result<Vec<Vec<Value>>> {
        let CreateTableStmt {
            if_not_exists,
            schema,
            name,
            columns,
            primary_key,
            unique_constraints,
        } = stmt;

        // check if table with the same name exists in meta table `table`
        if self.table_exists(&name) {
            return Err(TableExistsSnafu { name }.build());
        };

        // create a new record in `table` table
        // TODO: generate an id for new table
        let table_id = 123;
        let table = meta::Table {
            id: table_id,
            name,
            schema_id: schema,
        };
        self.create_table_record(table.clone(), manager)?;

        // create new records in `column` table
        let columns = transform_columns(columns, table_id);
        self.create_column_records(columns.clone(), manager)?;

        {
            let mut binder = self.binder.try_write().unwrap();
            binder.update_table(table);
            binder.update_columns(columns);
        }

        // create table file
        // TODO: determine table space by schema and database
        let space_id = meta::TABLESPACE_ID_DEFAULT;
        let file_node = FileNode::new(space_id, self.database, table_id);
        BTree::<Codec>::init(file_node, &manager).context(AccessSnafu)?;

        Ok(vec![vec![Value::Uint(1)]])
    }

    fn table_exists(&self, table: &str) -> bool {
        // TODO: use index and cache
        false
    }

    // TODO: just a reference of `table` should be enough
    fn create_table_record(&self, table: meta::Table, manager: &BufferManager) -> Result<()> {
        let file_node = FileNode::new(
            meta::TABLESPACE_ID_DEFAULT,
            self.database,
            meta::Table::TABLE_ID,
        );

        let (key_codec, values_codec) = {
            let mut columns = meta::Table::columns();
            let v_columns = columns.split_off(1);
            let k_columns = columns;

            (Codec::new(k_columns), Codec::new(v_columns))
        };

        let mut btree = BTree::new(key_codec, 30, file_node, manager);

        let mut kv: Vec<Value> = table.into();
        let values = kv.split_off(1);
        let key = kv;

        let values = values_codec.encode(&values).unwrap();
        btree.insert(&key, &values).context(AccessSnafu)
    }

    fn create_column_records(
        &self,
        columns: Vec<meta::Column>,
        manager: &BufferManager,
    ) -> Result<()> {
        let file_node = FileNode::new(
            meta::TABLESPACE_ID_DEFAULT,
            self.database,
            meta::Column::TABLE_ID,
        );

        let (key_codec, values_codec) = {
            let mut columns = meta::Column::columns();
            let v_columns = columns.split_off(2);
            let k_columns = columns;

            (Codec::new(k_columns), Codec::new(v_columns))
        };

        let mut btree = BTree::new(key_codec, 30, file_node, manager);

        columns.into_iter().for_each(|column| {
            let mut kv: Vec<Value> = column.into();
            let values = kv.split_off(2);
            let key = kv;

            let values = values_codec.encode(&values).unwrap();
            btree.insert(&key, &values).unwrap();
        });

        Ok(())
    }
}

fn transform_columns(columns: Vec<Column>, table_id: TableId) -> Vec<meta::Column> {
    columns
        .into_iter()
        .enumerate()
        .map(|(i, col)| {
            let (type_id, type_len) = col.data_type.value_repr();

            meta::Column {
                table_id,
                num: i as i16 + 1,
                name: col.name,
                type_id,
                type_len,
                is_nullable: col.is_nullable,
            }
        })
        .collect()
}

// #[cfg(test)]
// mod tests {
//     use {super::*, def::DataType, storage::DEFAULT_PAGE_SIZE, tempfile::tempdir};

//     #[test]
//     fn it_works() -> Result<()> {
//         let executor = Executor { database: 1 };

//         let temp_dir = tempdir().unwrap();
//         let path = temp_dir.path();
//         let mut manager = BufferManager::new(100, DEFAULT_PAGE_SIZE, path.to_path_buf());

//         let columns = vec![Column {
//             name: "abc".to_string(),
//             data_type: DataType::Int,
//             is_nullable: false,
//         }];

//         let stmt = CreateTableStmt {
//             if_not_exists: true,
//             schema: meta::SCHEMA_ID_PUBLIC,
//             name: "test".to_string(),
//             columns,
//             primary_key: None,
//             unique_constraints: vec![],
//         };

//         executor.create_table(stmt, &mut manager)?;

//         temp_dir.close().unwrap();

//         Ok(())
//     }
// }
