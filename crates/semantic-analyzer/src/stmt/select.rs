use {
    crate::Analyzer,
    ast::{expr::Expression, ColumnRef, Spanned, TableFactor, TargetElem},
    bound_ast::{Query, QueryTarget, Statement},
    core::cmp::Ordering,
    def::TableId,
    snafu::prelude::*,
    std::collections::HashMap,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display(r#"table "{}" does not exist"#, name))]
    TableNotExists {
        name: Spanned<String>,
    },

    #[snafu(display(r#"table name "{}" specified more than once"#, name))]
    DuplicateTable {
        name: Spanned<String>,
    },

    #[snafu(display(r#"column "{}" does not exists"#, column_ref))]
    ColumnNotExists {
        column_ref: Spanned<String>,
    },

    #[snafu(display(r#"column reference "{}" is ambiguous"#, column_ref))]
    AmbiguousColumnReference {
        column_ref: Spanned<String>,
    },

    Unsupported,
}

type Result<T> = std::result::Result<T, Error>;

impl Analyzer<'_> {
    pub(crate) fn analyze_select(&self, query: ast::Query) -> Result<Statement> {
        let ast::Query {
            distinct,
            targets,
            from,
            cond,
        } = query;

        let mut tables = HashMap::new();

        for table in from {
            let (table, alias) = match table.factor {
                TableFactor::Table { name, alias } => (name, alias),
                _ => return Err(UnsupportedSnafu.build()),
            };

            let table_id = self.binder.get_table_id(1, table.0.clone()).ok_or(
                TableNotExistsSnafu {
                    name: table.clone(),
                }
                .build(),
            )?;

            if tables
                .insert(alias.unwrap_or(table.clone()).0.clone(), table_id)
                .is_some()
            {
                return Err(DuplicateTableSnafu { name: table }.build());
            }
        }

        let targets = targets
            .into_iter()
            .map(|t| match t {
                TargetElem::Expr {
                    expr: Expression::Column(column),
                    alias,
                } => self.bind_column_ref(column, &tables),
                _ => Err(UnsupportedSnafu.build()),
            })
            .collect::<Result<Vec<_>>>()?;

        let tables = tables.into_values().collect();

        Ok(Statement::Select(Query { targets, tables }))
    }

    fn bind_column_ref(
        &self,
        column: ColumnRef,
        tables: &HashMap<String, TableId>,
    ) -> Result<QueryTarget> {
        if let Some(table) = column.table {
            let table = tables
                .get(&table.0)
                .ok_or(TableNotExistsSnafu { name: table }.build())?
                .to_owned();

            let col = self.binder.get_column(table, column.name.0.clone()).ok_or(
                ColumnNotExistsSnafu {
                    column_ref: column.name,
                }
                .build(),
            )?;

            Ok(QueryTarget {
                table,
                column: col.num,
            })
        } else {
            let columns = tables
                .iter()
                .filter_map(|(_, &table)| {
                    self.binder
                        .get_column(table, column.name.0.clone())
                        .map(|col| QueryTarget {
                            table,
                            column: col.num,
                        })
                })
                .collect::<Vec<_>>();

            match columns.len().cmp(&1) {
                // FIXME: should use the full column reference here
                Ordering::Less => Err(ColumnNotExistsSnafu {
                    column_ref: column.name,
                }
                .build()),
                Ordering::Greater => Err(AmbiguousColumnReferenceSnafu {
                    column_ref: column.name,
                }
                .build()),
                Ordering::Equal => Ok(columns[0]),
            }
        }
    }
}
