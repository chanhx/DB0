use {
    super::Node,
    crate::{
        error::{Error, Result},
        parser::ast::{Identifier, InsertSource},
        planner::{Insert, PhysicalNode, Planner},
    },
    def::catalog::{DatabaseCatalog, Table},
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
            .map_err(|_| Error::RelationNotExist { name: table })?;

        // TODO need to validate the arguments
        let columns = columns.map(|v| v.into_iter().map(|id| id.0).collect());
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
