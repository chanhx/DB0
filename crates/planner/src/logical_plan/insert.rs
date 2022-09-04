use {
    super::Node,
    crate::{Error, Insert, PhysicalNode, Planner, Result},
    def::catalog::{DatabaseCatalog, Table},
    parser::ast::{Identifier, InsertSource},
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn build_insert(
        &self,
        table: String,
        columns: Option<Vec<Identifier>>,
        source: InsertSource,
    ) -> Result<Node> {
        let catalog = self.db_catalog();
        let table = catalog
            .get_table(&table)
            .map_err(|_| Error::RelationNotExists { name: table })?;

        // TODO need to validate the arguments
        let columns = columns
            .map(|v| {
                v.into_iter()
                    .map(|id| {
                        table
                            .get_column(id.0.clone())
                            .map(|col| col.id)
                            .map_err(|_| Error::ColumnNotExists {
                                name: id.0,
                                span: id.1,
                            })
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        let values = match source {
            InsertSource::Values(values) => values,
            _ => {
                return Err(Error::Internal(
                    "The `INSERT INTO SELECT` statement is not supported".into(),
                ))
            }
        };

        Ok(Node::Physical(PhysicalNode::Insert(Insert {
            table_id: table.id(),
            columns,
            values,
        })))
    }
}
