use {
    crate::Analyzer,
    ast::{
        expr::{Expression, Literal},
        InsertSource, Spanned,
    },
    bound_ast::{InsertStmt, Statement},
    core::cmp::Ordering,
    def::{meta, SqlType, Value},
    snafu::prelude::*,
    std::collections::HashSet,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display(r#"table "{}" does not exist"#, name))]
    TableNotExists {
        name: Spanned<String>,
    },

    #[snafu(display(r#"column "{}" of table "{}" does not exists"#, name, table))]
    ColumnNotExists {
        name: Spanned<String>,
        table: Spanned<String>,
    },

    #[snafu(display(r#"column "{}" specified more than once"#, column))]
    DuplicateColumn {
        column: Spanned<String>,
    },

    #[snafu(display(
        r#"null value in column "{}" of table "{}" violates not-null constraint"#,
        column,
        table
    ))]
    NullValue {
        column: String,
        table: Spanned<String>,
    },

    // #[snafu(display(r#"invalid input syntax for type {}: "{}""#, sql_type, value))]
    #[snafu(display("invalid input syntax for type {}", sql_type))]
    InvalidInput {
        sql_type: SqlType,
        // value: Literal,
    },

    #[snafu(display("INSERT has more expressions than target columns",))]
    TooManyExpressions,

    #[snafu(display("INSERT has more target columns than expressions",))]
    TooManyTargets,

    #[snafu(display("{} out of range", sql_type))]
    ValueOutOfRange {
        sql_type: SqlType,
    },

    #[snafu(display("value is too long for type {}({})", sql_type, type_len))]
    ValueTooLong {
        sql_type: SqlType,
        type_len: u16,
    },

    Unsupported,
}

type Result<T> = std::result::Result<T, Error>;

impl Analyzer {
    pub(crate) fn analyze_insert(&self, stmt: ast::InsertStmt) -> Result<Statement> {
        let ast::InsertStmt {
            table,
            targets,
            source,
        } = stmt;

        let (table_id, targets) = {
            let binder = self.binder.read().unwrap();
            let table_id = binder
                .get_table_id(meta::SCHEMA_ID_PUBLIC, table.0.clone())
                .ok_or(
                    TableNotExistsSnafu {
                        name: table.clone(),
                    }
                    .build(),
                )?;

            let columns = binder.get_columns(table_id);

            let targets = match targets {
                Some(targets) => {
                    let mut target_set = HashSet::new();
                    let targets = targets
                        .into_iter()
                        .map(
                            |target| match binder.get_column(table_id, target.0.clone()) {
                                Some(col) => {
                                    if !target_set.insert(col.num) {
                                        return Err(DuplicateColumnSnafu { column: target }.build());
                                    }

                                    Ok(col)
                                }
                                None => Err(ColumnNotExistsSnafu {
                                    name: target,
                                    table: table.clone(),
                                }
                                .build()),
                            },
                        )
                        .collect::<Result<Vec<_>>>()?;

                    // check if any non-null constraint is violated
                    if let Some(col) = columns
                        .iter()
                        .filter(|col| !target_set.contains(&col.num))
                        .find(|col| !col.is_nullable)
                    {
                        return Err(NullValueSnafu {
                            column: col.name.clone(),
                            table,
                        }
                        .build());
                    }

                    targets
                }
                None => columns,
            };

            (table_id, targets)
        };

        let source = match source {
            InsertSource::FromQuery(_) => return Err(UnsupportedSnafu.build()),
            InsertSource::Values(exprs) => exprs
                .into_iter()
                .map(|exprs| {
                    match exprs.len().cmp(&targets.len()) {
                        Ordering::Greater => return Err(TooManyExpressionsSnafu.build()),
                        Ordering::Less => return Err(TooManyTargetsSnafu.build()),
                        _ => {}
                    }

                    exprs
                        .into_iter()
                        .enumerate()
                        .map(|(i, expr)| {
                            let target = targets.get(i).unwrap();

                            match expr {
                                Expression::Literal(literal) => cast_value(literal, target),
                                _ => Err(UnsupportedSnafu.build()),
                            }
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<_>>>()?,
        };

        let targets = targets.into_iter().map(|target| target.num).collect();

        Ok(Statement::Insert(InsertStmt {
            table: table_id,
            targets,
            source,
        }))
    }
}

fn cast_value(literal: Literal, target: &meta::Column) -> Result<Value> {
    let sql_type = target.type_id.clone();

    Ok(match (literal, &sql_type) {
        (Literal::Null, _) => Value::Null,

        (Literal::Boolean(v), SqlType::Boolean) => Value::Boolean(v),

        (Literal::Float(v), SqlType::Double) => Value::Double(v),
        (Literal::Float(v), SqlType::Float) => {
            if v > f32::MAX as f64 || v < f32::MIN as f64 {
                return Err(ValueOutOfRangeSnafu { sql_type }.build());
            }
            Value::Float(v as f32)
        }

        (Literal::Int(v), SqlType::BigInt) => Value::BigInt(v),
        (Literal::Int(v), SqlType::Int) => {
            if v < i32::MIN as i64 {
                return Err(ValueOutOfRangeSnafu { sql_type }.build());
            }
            Value::Int(v as i32)
        }
        (Literal::Int(v), SqlType::SmallInt) => {
            if v < i16::MIN as i64 {
                return Err(ValueOutOfRangeSnafu { sql_type }.build());
            }
            Value::SmallInt(v as i16)
        }
        (Literal::Int(v), SqlType::TinyInt) => {
            if v < i8::MIN as i64 {
                return Err(ValueOutOfRangeSnafu { sql_type }.build());
            }
            Value::TinyInt(v as i8)
        }

        (Literal::Uint(v), SqlType::BigUint) => Value::BigUint(v),
        (Literal::Uint(v), SqlType::Uint) => {
            if v > u32::MAX as u64 {
                return Err(ValueOutOfRangeSnafu { sql_type }.build());
            }
            Value::Uint(v as u32)
        }
        (Literal::Uint(v), SqlType::SmallUint) => {
            if v > u16::MAX as u64 {
                return Err(ValueOutOfRangeSnafu { sql_type }.build());
            }
            Value::SmallUint(v as u16)
        }
        (Literal::Uint(v), SqlType::TinyUint) => {
            if v < u8::MIN as u64 {
                return Err(ValueOutOfRangeSnafu { sql_type }.build());
            }
            Value::TinyUint(v as u8)
        }

        (Literal::String(v), SqlType::Varchar) | (Literal::String(v), SqlType::Char) => {
            if v.len() < target.type_len as usize {
                Value::String(v)
            } else {
                return Err(ValueTooLongSnafu {
                    sql_type: target.type_id.clone(),
                    type_len: target.type_len,
                }
                .build());
            }
        }

        _ => return Err(InvalidInputSnafu { sql_type }.build()),
    })
}
