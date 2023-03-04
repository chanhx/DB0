use {
    crate::Executor,
    access::{btree::BTree, Codec},
    bound_ast::InsertStmt,
    def::{
        meta::{self},
        storage::Encoder,
        Value,
    },
    snafu::{prelude::*, ResultExt},
    storage::buffer::{BufferManager, FileNode},
};

#[derive(Debug, Snafu)]
pub enum Error {
    Encoding {
        #[snafu(backtrace)]
        source: access::codec::Error,
    },

    Access {
        #[snafu(backtrace)]
        source: access::btree::error::Error,
    },
}

type Result<T> = std::result::Result<T, Error>;

impl Executor {
    pub(crate) fn insert(
        &self,
        stmt: InsertStmt,
        manager: &BufferManager,
    ) -> Result<Vec<Vec<Value>>> {
        let InsertStmt {
            table,
            targets,
            source,
        } = stmt;

        let file_node = FileNode::new(meta::TABLESPACE_ID_DEFAULT, self.database, table);

        let columns_count;
        let (key_codec, values_codec) = {
            let binder = self.binder.read().unwrap();
            // TODO: there should be some information about primary keys in metadata
            let mut columns = binder.get_columns(table);
            columns_count = columns.len();

            let v_columns = columns.split_off(1);
            let k_columns = columns;

            (Codec::new(k_columns), Codec::new(v_columns))
        };

        let mut btree = BTree::new(key_codec, 100, file_node, &manager);

        let mut new_rows_count = 0;

        let mut values = vec![Value::Null; columns_count];
        for row in source {
            row.into_iter().zip(targets.iter()).for_each(|(v, &i)| {
                values[i as usize - 1] = v;
            });

            // TODO: no need to clone `values`, should update the `Encoder` trait
            let mut tmp_values = values.clone();
            // FIXME: remove hard code
            let values = tmp_values.split_off(1);
            let key = tmp_values;

            let values = values_codec.encode(&values).context(EncodingSnafu)?;

            // TODO: should distinguish whether this is an insert or an update
            btree.insert(&key, &values).context(AccessSnafu)?;
            new_rows_count += 1;
        }

        Ok(vec![vec![Value::Uint(new_rows_count)]])
    }
}
