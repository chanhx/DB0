use {
    super::{Generator, LogicalNode},
    crate::{Error, Result, Scan},
    def::catalog::Table,
    parser::ast::{
        dml::{FromItem, Query, SelectFrom, TargetElem},
        expr::Expression,
    },
    std::collections::HashMap,
};

impl Generator {
    pub fn build_query_plan(&self, query: Query) -> Result<LogicalNode> {
        let mut scope = Scope::<'b, D::T>::new();

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

    fn build_from_clause(
        &'a self,
        scope: &mut Scope<'b, D::T>,
        from: SelectFrom,
    ) -> Result<LogicalNode> {
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
            LogicalNode::Join {
                initial_node: Box::new(node),
                joined_nodes,
            }
        } else {
            node
        })
    }

    fn build_scan(&'a self, scope: &mut Scope<'b, D::T>, item: FromItem) -> Result<LogicalNode> {
        Ok(match item {
            FromItem::Table { name, alias } => {
                let catalog = self.db_catalog();
                let table = catalog
                    .get_table(&name.0)
                    .map_err(|_| Error::RelationNotExists {
                        name: name.to_string(),
                    })?;

                scope.tables.insert(name.to_string(), table);
                if let Some(alias) = alias {
                    scope.table_aliases.insert(alias.0, table);
                }

                LogicalNode::Scan(Scan {
                    table_id: table.id(),
                    projection: None,
                })
            }
            FromItem::SubQuery { .. } => unimplemented!("subquery is not supported now"),
        })
    }
}

fn build_filter(predict: Expression, input: Option<LogicalNode>) -> Result<LogicalNode> {
    Ok(LogicalNode::Filter {
        input: input.map(|input| Box::new(input)),
        predict,
    })
}

fn build_projection<'a, T: Table>(
    _scope: &mut Scope<'a, T>,
    distinct: bool,
    targets: Vec<TargetElem>,
    input: Option<LogicalNode>,
) -> Result<LogicalNode> {
    Ok(LogicalNode::Projection {
        input: input.map(|input| Box::new(input)),
        distinct,
        targets,
    })
}

#[derive(Default)]
struct Scope<'a, T: Table> {
    table_aliases: HashMap<String, &'a T>,
    tables: HashMap<String, &'a T>,
}

impl<'a, T: Table> Scope<'a, T> {
    fn new() -> Self {
        Self {
            table_aliases: HashMap::new(),
            tables: HashMap::new(),
        }
    }
}
