use {
    crate::Analyzer,
    ast::{ColumnConstraint, Identifier, Span, Spanned, TableConstraint},
    bound_ast::{Column, ColumnNum, CreateTableStmt, Statement},
    def::meta,
    snafu::prelude::*,
    std::collections::HashMap,
};

#[derive(Debug, Snafu)]
pub enum Error {
    DuplicateColumn {
        name: Identifier,
    },

    #[snafu(display("multiple primary keys for table {} are not allowed", table))]
    MultiplePrimaryKey {
        span: Span,
        table: String,
    },

    #[snafu(display("column {} named in key does not exist", name))]
    UndefinedColumn {
        span: Span,
        name: String,
    },
}

impl Analyzer {
    pub(crate) fn analyze_create_table(
        &self,
        stmt: ast::CreateTableStmt,
    ) -> Result<Statement, Error> {
        let ast::CreateTableStmt {
            if_not_exists,
            name,
            table_schema,
        } = stmt;

        let mut column_nums = HashMap::new();

        let mut primary_key = None;
        let mut unique_constraints = vec![];

        let columns = table_schema
            .columns
            .into_iter()
            .enumerate()
            .map(|(i, column)| {
                let col_name = column.name.to_string();
                if column_nums
                    .insert(col_name.clone(), i as ColumnNum)
                    .is_some()
                {
                    return Err(DuplicateColumnSnafu { name: column.name }.build());
                }

                let mut is_nullable = true;

                // check multiple primary keys
                for Spanned(constraint, span) in column.constraints {
                    match constraint {
                        ColumnConstraint::PrimaryKey if primary_key.is_some() => {
                            return Err(Error::MultiplePrimaryKey {
                                span,
                                table: name.0.clone(),
                            });
                        }
                        ColumnConstraint::PrimaryKey => {
                            primary_key = Some(vec![i as ColumnNum]);
                            is_nullable = false;
                        }
                        ColumnConstraint::NotNull => {
                            is_nullable = false;
                        }
                        _ => {}
                    }
                }

                Ok(Column {
                    name: col_name,
                    data_type: column.data_type,
                    is_nullable,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        // check if there are conflicts with column constraints or undefined columns
        for Spanned(constraint, span) in table_schema.constraints {
            match constraint {
                TableConstraint::PrimaryKey(_) if primary_key.is_some() => {
                    return Err(MultiplePrimaryKeySnafu {
                        span,
                        table: name.0,
                    }
                    .build());
                }

                TableConstraint::PrimaryKey(columns) => {
                    primary_key = Some(collect_column_nums(columns, &column_nums)?);
                }

                TableConstraint::Unique(columns) => {
                    let columns = collect_column_nums(columns, &column_nums)?;

                    unique_constraints.push(columns);
                }
            }
        }

        Ok(Statement::CreateTable(CreateTableStmt {
            if_not_exists,
            schema: meta::SCHEMA_ID_PUBLIC,
            name: name.to_string(),
            columns,
            primary_key,
            unique_constraints,
        }))
    }
}

fn collect_column_nums(
    columns: Vec<Identifier>,
    column_nums: &HashMap<String, ColumnNum>,
) -> Result<Vec<ColumnNum>, Error> {
    columns
        .iter()
        .map(|col| {
            column_nums
                .get(&col.0)
                .cloned()
                .ok_or(Error::UndefinedColumn {
                    span: col.1.clone(),
                    name: col.0.clone(),
                })
        })
        .collect()
}
