use {
    crate::{
        catalog::{DatabaseCatalog, TableSchema},
        error::{Error, Result},
        parser::ast::{Expr, FromItem, JoinItem, Query, SelectFrom, TargetElem},
        planner::{Filter, Join, Node, Planner, Projection, Scan},
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
        let mut node = self.build_scan(scope, from.item)?;

        for join in from.joins {
            node = self.build_join(scope, node, join)?;
        }

        Ok(node)
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

    fn build_join(&'a self, scope: &mut Scope<'b>, node: Node, join: JoinItem) -> Result<Node> {
        Ok(Node::Join(Join {
            join_type: join.join_type,
            left: Box::new(node),
            right: Box::new(self.build_scan(scope, join.item)?),
        }))
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
