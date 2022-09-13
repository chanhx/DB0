use {
    super::{Error, Generator, Result},
    crate::{physical_plan::CreateTable, LogicalNode, PhysicalNode},
    def::catalog::{ColumnDef, UniqueConstraint},
    parser::{
        ast::{
            ddl::{ColumnConstraint, CreateTableStmt, TableConstraint},
            Identifier,
        },
        Span,
    },
    std::collections::HashSet,
};

impl Generator {
    pub(super) fn generate_create_table_plan(&self, stmt: CreateTableStmt) -> Result<LogicalNode> {
        let CreateTableStmt {
            if_not_exists,
            name,
            table_schema,
        } = stmt;

        let mut ids = HashSet::new();

        let mut primary_key_columns = None;
        let mut unique_contraints = vec![];

        let columns = columns
            .into_iter()
            .map(|column| {
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
                            self.check_duplicate_primary_key(
                                &name.0,
                                primary_key_columns.is_some(),
                                span,
                            )?;

                            primary_key_columns = Some(vec![id.to_string()]);
                        }
                        ColumnConstraint::NotNull => {
                            is_nullable = false;
                        }
                        _ => {}
                    }
                }

                Ok(ColumnDef {
                    name: id.to_string(),
                    data_type: column.data_type,
                    is_nullable,
                    comment: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        // check if there are conflicts with column constraints or undefined columns
        for (constraint, span) in constraints {
            match constraint {
                TableConstraint::PrimaryKey(columns) => {
                    self.check_duplicate_primary_key(&name.0, primary_key_columns.is_some(), span)?;
                    self.check_undefined_column(&ids, &columns)?;

                    primary_key_columns = Some(columns.iter().map(|col| col.to_string()).collect());
                }

                TableConstraint::Unique(columns) => {
                    self.check_undefined_column(&ids, &columns)?;

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

        Ok(LogicalNode::Physical(PhysicalNode::CreateTable(
            CreateTable {
                if_not_exists,
                name: name.to_string(),
                columns,
                primary_key_columns,
                unique_contraints,
            },
        )))
    }

    fn check_duplicate_primary_key(
        &self,
        name: &str,
        primary_key_exists: bool,
        span: Span,
    ) -> Result<()> {
        if primary_key_exists {
            return Err(Error::MultiplePrimaryKey {
                span,
                details: format!(r#"multiple primary keys for table "{name}" are not allowed"#),
            });
        }

        Ok(())
    }

    fn check_undefined_column(
        &self,
        ids: &HashSet<String>,
        columns: &Vec<Identifier>,
    ) -> Result<()> {
        if let Some(id) = columns.iter().find(|id| !ids.contains(&id.0)) {
            return Err(Error::UndefinedColumn {
                span: id.1.clone(),
                details: format!(r#"column "{}" named in key does not exist "#, id.0),
            });
        }

        Ok(())
    }
}
