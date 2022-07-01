use {
    crate::{
        common::{Span, Spanned},
        parser::ast::{self, ColumnConstraint, Identifier, TableConstraint},
        planner::{physical_plan::CreateTable, Error, Result},
    },
    def::catalog::{ColumnDef, UniqueConstraint},
    std::collections::HashSet,
};

pub fn build_create_table_plan(
    if_not_exists: bool,
    name: String,
    ast_columns: Vec<ast::Column>,
    constraints: Vec<Spanned<TableConstraint>>,
) -> Result<CreateTable> {
    let mut ids = HashSet::new();

    let mut columns = vec![];
    let mut primary_key_columns = None;
    let mut unique_contraints = vec![];

    for column in ast_columns {
        // check duplicate columns
        let id = &column.name;
        if !ids.insert(id.to_string()) {
            return Err(Error::DuplicateColumn {
                span: id.1.clone(),
                details: format!(r#"column "{}" specified more than once"#, id),
            });
        }

        let mut is_nullable = true;

        // check multiple primary keys
        for (constraint, span) in column.constraints {
            match constraint {
                ColumnConstraint::PrimaryKey => {
                    check_duplicate_primary_key(&name, primary_key_columns.is_some(), span)?;

                    primary_key_columns = Some(vec![id.to_string()]);
                }
                ColumnConstraint::NotNull => {
                    is_nullable = false;
                }
                _ => {}
            }
        }

        columns.push(ColumnDef {
            name: id.to_string(),
            data_type: column.data_type,
            is_nullable,
            comment: None,
        });
    }

    // check if there are conflicts with column constraints or undefined columns
    for (constraint, span) in constraints {
        match constraint {
            TableConstraint::PrimaryKey(columns) => {
                check_duplicate_primary_key(&name, primary_key_columns.is_some(), span)?;
                check_undefined_column(&ids, &columns)?;

                primary_key_columns = Some(columns.iter().map(|col| col.to_string()).collect());
            }

            TableConstraint::Unique(columns) => {
                check_undefined_column(&ids, &columns)?;

                let columns = columns
                    .iter()
                    .map(|col| col.to_string())
                    .collect::<Vec<_>>();

                unique_contraints.push(UniqueConstraint {
                    name: format!("{}_{}_key", name, columns.join("_")),
                    columns,
                });
            }
        }
    }

    Ok(CreateTable {
        if_not_exists,
        name,
        columns,
        primary_key_columns,
        unique_contraints,
    })
}

fn check_duplicate_primary_key(name: &str, primary_key_exists: bool, span: Span) -> Result<()> {
    if primary_key_exists {
        return Err(Error::MultiplePrimaryKey {
            span,
            details: format!(r#"multiple primary keys for table "{name}" are not allowed"#),
        });
    }

    Ok(())
}

fn check_undefined_column(ids: &HashSet<String>, columns: &Vec<Identifier>) -> Result<()> {
    if let Some(id) = columns.iter().find(|id| !ids.contains(&id.0)) {
        return Err(Error::UndefinedColumn {
            span: id.1.clone(),
            details: format!(r#"column "{}" named in key does not exist "#, id.0),
        });
    }

    Ok(())
}
