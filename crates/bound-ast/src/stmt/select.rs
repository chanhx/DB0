use {crate::ColumnNum, common::pub_fields_struct, def::TableId};

pub_fields_struct! {
    #[derive(Debug, PartialEq, Clone, Copy)]
    struct QueryTarget {
        table: TableId,
        column: ColumnNum,
    }

    #[derive(Debug, PartialEq)]
    struct Query {
        targets: Vec<QueryTarget>,
        tables: Vec<TableId>,
    }
}
