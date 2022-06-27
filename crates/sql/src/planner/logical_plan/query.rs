use {
    crate::{
        catalog::{DatabaseCatalog, TableSchema},
        error::{Error, Result},
        parser::ast::{Expr, FromItem, Query, SelectFrom, TargetElem},
        planner::{Filter, JoinItem, Node, Planner, Projection, Scan},
    },
    std::collections::HashMap,
};

impl<'b, 'a: 'b, D: DatabaseCatalog> Planner<'a, D> {
    pub fn build_query_plan(&self, query: Query) -> Result<Node> {
        let mut scope = Scope::default();

        let node = query
            .from
            .map(|from| self.build_from_clause(&mut scope, from))
            .transpose()?;

        let node = query
            .cond
            .map(|expr| build_filter(expr, node))
            .transpose()?;

        build_projection(&mut scope, query.distinct, query.targets, node)
    }

    fn build_from_clause(&'a self, scope: &mut Scope<'b>, from: SelectFrom) -> Result<Node> {
        let node = self.build_scan(scope, from.item)?;

        let joined_nodes = from
            .joins
            .into_iter()
            .map(|j| {
                Ok(JoinItem {
                    join_type: j.join_type,
                    node: self.build_scan(scope, j.item)?,
                    cond: j.cond,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(if joined_nodes.len() > 0 {
            Node::LogicalJoin {
                initial_node: Box::new(node),
                joined_nodes,
            }
        } else {
            node
        })
    }

    fn build_scan(&'a self, scope: &mut Scope<'b>, item: FromItem) -> Result<Node> {
        Ok(match item {
            FromItem::Table { name, alias } => {
                let catalog = self.db_catalog();
                let table_id = catalog
                    .get_table_id(&name.0)
                    .ok_or(Error::RelationNotExist {
                        name: name.to_string(),
                    })?;
                let table = catalog.get_table_schema(table_id)?;

                scope.tables.insert(name.to_string(), table);
                if let Some(alias) = alias {
                    scope.table_aliases.insert(alias.0, table);
                }

                Node::Scan(Scan {
                    table_id,
                    projection: None,
                })
            }
            FromItem::SubQuery { .. } => unimplemented!("subquery is not supported now"),
        })
    }
}

fn build_filter(predict: Expr, input: Option<Node>) -> Result<Node> {
    let filter = Filter {
        input: input.map(|input| Box::new(input)),
        predict,
    };

    Ok(Node::Filter(filter))
}

fn build_projection<'a>(
    _scope: &mut Scope<'a>,
    distinct: bool,
    targets: Vec<TargetElem>,
    input: Option<Node>,
) -> Result<Node> {
    // TODO: validate targets with scope
    let projection = Projection {
        input: input.map(|input| Box::new(input)),
        distinct,
        targets,
    };

    Ok(Node::Projection(projection))
}

#[derive(Default)]
struct Scope<'a> {
    table_aliases: HashMap<String, &'a TableSchema>,
    tables: HashMap<String, &'a TableSchema>,
}
