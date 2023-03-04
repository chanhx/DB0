use {
    access::{BTree, Codec},
    core::{default::Default, ops::Bound::Excluded},
    def::{
        meta::{self, MetaTable},
        storage::Decoder,
        DatabaseId, SchemaId, TableId, Value,
    },
    snafu::prelude::*,
    std::collections::BTreeMap,
    storage::buffer::{BufferManager, FileNode},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("error reading metadata"))]
    MetaData {
        #[snafu(backtrace)]
        source: meta::error::Error,
    },
}

type Result<T> = std::result::Result<T, Error>;

pub struct Binder {
    database: DatabaseId,
    tables: BTreeMap<(SchemaId, String), meta::Table>,
    columns: BTreeMap<(TableId, String), meta::Column>,
}

impl Binder {
    pub fn new(database: DatabaseId, manager: &BufferManager) -> Result<Self> {
        let mut binder = Self {
            database,
            tables: Default::default(),
            columns: Default::default(),
        };

        binder.build_index(manager)?;

        Ok(binder)
    }

    pub fn get_table_id(&self, schema_id: SchemaId, table: String) -> Option<TableId> {
        self.tables.get(&(schema_id, table)).map(|tbl| tbl.id)
    }

    pub fn get_columns(&self, table_id: TableId) -> Vec<meta::Column> {
        self.columns
            .range((
                Excluded((table_id, "".to_string())),
                Excluded((table_id + 1, "".to_string())),
            ))
            .into_iter()
            .map(|(_, column)| column.clone())
            .collect()
    }

    pub fn get_column(&self, table_id: TableId, name: String) -> Option<meta::Column> {
        self.columns.get(&(table_id, name)).cloned()
    }

    fn build_index(&mut self, manager: &BufferManager) -> Result<()> {
        let tables = self.load_tables(manager)?;
        tables.into_iter().for_each(|tbl| {
            self.tables.insert((tbl.schema_id, tbl.name.clone()), tbl);
        });

        let columns = self.load_columns(manager)?;
        columns.into_iter().for_each(|col| {
            self.columns.insert((col.table_id, col.name.clone()), col);
        });

        Ok(())
    }

    fn load_tables(&self, manager: &BufferManager) -> Result<Vec<meta::Table>> {
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

        let btree = BTree::new(key_codec, 100, file_node, manager);

        let key = vec![Value::Uint(TableId::MIN)];
        let (cursor, _) = btree.search(&key).unwrap().unwrap();

        cursor
            .into_iter()
            .map(|entry| {
                let (value, _) = values_codec.decode(&entry.1).unwrap();
                let values = [entry.0, value].concat();

                meta::Table::try_from(values).context(MetaDataSnafu)
            })
            .collect()
    }

    fn load_columns(&self, manager: &BufferManager) -> Result<Vec<meta::Column>> {
        let file_node = FileNode::new(
            meta::TABLESPACE_ID_DEFAULT,
            self.database,
            meta::Column::TABLE_ID,
        );

        let (key_codec, values_codec) = {
            let mut columns = meta::Column::columns();
            // TODO: remove hard code
            let v_columns = columns.split_off(2);
            let k_columns = columns;

            (Codec::new(k_columns), Codec::new(v_columns))
        };

        let btree = BTree::new(key_codec, 100, file_node, manager);

        let key = vec![Value::Uint(TableId::MIN), Value::SmallInt(i16::MIN)];
        let (cursor, _) = btree.search(&key).unwrap().unwrap();

        cursor
            .into_iter()
            .map(|entry| {
                let (value, _) = values_codec.decode(&entry.1).unwrap();
                let values = [entry.0, value].concat();

                meta::Column::try_from(values).context(MetaDataSnafu)
            })
            .collect()
    }

    pub fn update_table(&mut self, table: meta::Table) {
        self.tables
            .insert((table.schema_id, table.name.clone()), table);
    }

    pub fn update_columns(&mut self, columns: Vec<meta::Column>) {
        columns.into_iter().for_each(|col| {
            self.columns.insert((col.table_id, col.name.clone()), col);
        })
    }
}
