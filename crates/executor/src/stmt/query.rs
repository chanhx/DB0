use {
    crate::Executor,
    access::{btree::BTree, Codec},
    bound_ast::Query,
    def::{meta, storage::Decoder, Value},
    snafu::{prelude::*, ResultExt},
    std::collections::HashMap,
    storage::buffer::{BufferManager, FileNode},
};

#[derive(Debug, Snafu)]
pub enum Error {
    Access {
        #[snafu(backtrace)]
        source: access::btree::error::Error,
    },
}

type Result<T> = std::result::Result<T, Error>;

impl Executor {
    // leave alone planning for now
    pub(crate) fn select(&self, stmt: Query, manager: &BufferManager) -> Result<Vec<Vec<Value>>> {
        let Query { targets, tables } = stmt;

        let row_values = tables
            .iter()
            .map(|&table| {
                let file_node = FileNode::new(meta::TABLESPACE_ID_DEFAULT, self.database, table);

                let (key_codec, values_codec) = {
                    let binder = self.binder.read().unwrap();
                    // TODO: there should be some information about primary keys in metadata
                    let mut columns = binder.get_columns(table);
                    let v_columns = columns.split_off(1);
                    let k_columns = columns;

                    (Codec::new(k_columns), Codec::new(v_columns))
                };

                let btree = BTree::new(key_codec, 100, file_node, &manager);

                // FIXME: generate search key by key columns
                let (cursor, _) = btree
                    .search(&vec![Value::Null])
                    .context(AccessSnafu)?
                    .unwrap();

                Ok((
                    table,
                    cursor
                        .into_iter()
                        .map(|entry| {
                            let (value, _) = values_codec.decode(&entry.1).unwrap();
                            [entry.0, value].concat()
                        })
                        .collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let mut values = vec![];

        // FIXME: it should be the Cartesian Product of all tables, leave it alone for now
        for table in tables {
            values = row_values[&table]
                .to_owned()
                .into_iter()
                .map(|row| {
                    targets
                        .iter()
                        .map(|t| row.get(t.column as usize - 1).unwrap().clone())
                        .collect()
                })
                .collect();

            break;
        }

        Ok(values)
    }
}
