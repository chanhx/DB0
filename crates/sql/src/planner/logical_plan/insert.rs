use {
    super::Node,
    crate::{
        catalog::DatabaseCatalog,
        error::{Error, Result},
        parser::ast::{Identifier, InsertSource},
        planner::{Insert, PhysicalNode, Planner},
    },
};

impl<'a, D: DatabaseCatalog> Planner<'a, D> {
    pub(super) fn build_insert(
        &self,
        table: String,
        columns: Option<Vec<Identifier>>,
        source: InsertSource,
    ) -> Result<Node> {
        let catalog = self.db_catalog();
        let table_id = catalog
            .get_table_id(&table)
            .ok_or(Error::RelationNotExist { name: table })?;

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
            table_id,
            columns,
            values,
        })))
    }
}
