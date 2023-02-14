use super::notebook::Scope;
use nanoid::nanoid;
use pyo3::{prelude::*, types::PyDict};
use rustpython_parser::{
    ast::{AliasData, ExprContext, ExprKind, Located, StmtKind},
    error::ParseError,
    parser,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, error::Error};
use tracing::{info, log::warn};

pub type Dependencies = HashSet<String>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    pub metadata: CellMetadata,
    pub uuid: String,
    pub cell_type: CellType,
    pub content: String,
    pub dependencies: Dependencies,

    #[serde(skip)]
    locals: Option<Py<PyDict>>,
}

impl Cell {
    pub fn new(
        cell_type: CellType,
        content: String,
        scope: &mut Scope,
    ) -> Result<Self, Box<dyn Error>> {
        let mut cell = Self {
            metadata: CellMetadata { collapsed: false },
            uuid: nanoid!(30),
            cell_type,
            content,
            locals: Some(Python::with_gil(|py| PyDict::new(py).into())),
            dependencies: Dependencies::new(),
        };

        cell.build_dependencies(scope)?;

        Ok(cell)
    }

    pub fn new_reactive(content: &str, scope: &mut Scope) -> Result<Self, Box<dyn Error>> {
        Self::new(CellType::ReactiveCode, String::from(content), scope)
    }

    // pub fn eval(&mut self, kernel: &mut Kernel) {
    //     let locals = self.locals.as_mut().unwrap();
    //     kernel.eval(&self.content, locals);
    // }

    pub fn build_dependencies(&mut self, scope: &mut Scope) -> Result<(), ParseError> {
        match self.cell_type {
            CellType::ReactiveCode | CellType::NonReactiveCode => self.code_dependencies(scope),
            CellType::Markdown => todo!(),
        }
    }

    fn code_dependencies(&mut self, scope: &mut Scope) -> Result<(), ParseError> {
        let ast = parser::parse_program(&self.content, "<input>")?;

        for statement in ast.iter() {
            match &statement.node {
                StmtKind::Import { names } => self.import_dependencies(names, scope),
                StmtKind::Assign { targets, value, .. } => {
                    for target in targets {
                        self.handle_expr_node(&target.node, scope);
                    }
                    self.handle_expr_node(&value.node, scope);
                }
                StmtKind::Expr { value } => self.handle_expr_node(&value.node, scope),
                _ => warn!("Unsupported statement: {:#?}", statement),
            };
        }

        Ok(())
    }

    fn import_dependencies(&mut self, names: &Vec<Located<AliasData>>, scope: &mut Scope) {
        info!("Import statement: {:#?}", names);
        for name in names {
            if let Some(alias) = &name.node.asname {
                info!("alias: {:#?}", alias);
                scope.insert(alias.to_string(), self.uuid.clone());
            } else {
                let import_name = name.node.name.to_string();
                info!("name: {:#?}", import_name);
                scope.insert(import_name, self.uuid.clone());
            }
        }
    }

    fn handle_expr_node(&mut self, node: &ExprKind, scope: &mut Scope) {
        match node {
            ExprKind::Name { id, ctx } => self.handle_name_dep(id, ctx, scope),
            ExprKind::BinOp { left, right, .. } => {
                self.handle_expr_node(&left.node, scope);
                self.handle_expr_node(&right.node, scope);
            }
            ExprKind::Attribute { value, attr, ctx } => {
                self.handle_expr_node(&value.node, scope);
            }
            ExprKind::List { elts, ctx } => {
                for elt in elts {
                    self.handle_expr_node(&elt.node, scope);
                }
            }
            ExprKind::Constant { .. } => {}
            ExprKind::Call { func, args, .. } => {
                eprintln!("Call: {:#?}", func);
                // match &func.node {
                //     ExprKind::Attribute { value, .. } => match &value.node {
                //         ExprKind::Name { id, ctx, .. } => {
                //             self.handle_name_dep(id, cells, ctx, dep_topology)
                //         }
                //         _ => warn!("Unsupported value assign value: {:#?}", value),
                //     },
                //     _ => warn!("Unsupported func assign value: {:#?}", value),
                // }

                // for arg in args {
                //     match &arg.node {
                //         ExprKind::Constant { .. } => {}
                //         ExprKind::BinOp { left, right, .. } => {}
                //         _ => warn!("Unsupported arg assign value: {:#?}", value),
                //     }
                // }
            }
            // ExprKind::BoolOp { op, values } => todo!(),
            // ExprKind::NamedExpr { target, value } => todo!(),
            // ExprKind::UnaryOp { op, operand } => todo!(),
            // ExprKind::Lambda { args, body } => todo!(),
            // ExprKind::IfExp { test, body, orelse } => todo!(),
            // ExprKind::Dict { keys, values } => todo!(),
            // ExprKind::Set { elts } => todo!(),
            // ExprKind::ListComp { elt, generators } => todo!(),
            // ExprKind::SetComp { elt, generators } => todo!(),
            // ExprKind::DictComp {
            //     key,
            //     value,
            //     generators,
            // } => todo!(),
            // ExprKind::GeneratorExp { elt, generators } => todo!(),
            // ExprKind::Await { value } => todo!(),
            // ExprKind::Yield { value } => todo!(),
            // ExprKind::YieldFrom { value } => todo!(),
            // ExprKind::Compare {
            //     left,
            //     ops,
            //     comparators,
            // } => todo!(),
            // ExprKind::FormattedValue {
            //     value,
            //     conversion,
            //     format_spec,
            // } => todo!(),
            // ExprKind::JoinedStr { values } => todo!(),
            // ExprKind::Subscript { value, slice, ctx } => todo!(),
            // ExprKind::Starred { value, ctx } => todo!(),
            // ExprKind::Tuple { elts, ctx } => todo!(),
            // ExprKind::Slice { lower, upper, step } => todo!(),
            _ => warn!("Unsupported expr node: {:#?}", node),
        }
    }

    fn handle_name_dep(&mut self, id: &str, ctx: &ExprContext, scope: &mut Scope) {
        match ctx {
            ExprContext::Load => {
                if let Some(dep) = scope.get(id) {
                    self.dependencies.insert(dep.to_string());
                }
            }
            ExprContext::Store => {
                scope.insert(id.to_string(), self.uuid.clone());
            }
            ExprContext::Del => {}
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CellMetadata {
    pub collapsed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CellType {
    NonReactiveCode,
    ReactiveCode,
    Markdown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_trivial_code_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let mut cell = Cell::new_reactive("a = 1", &mut scope)?;
        cell.build_dependencies(&mut scope)?;

        let expect: Dependencies = HashSet::new();

        assert_eq!(scope.get("a"), Some(&cell.uuid));
        Ok(assert_eq!(cell.dependencies, expect))
    }

    #[test]
    fn test_assign_code_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        assert_eq!(scope.get("a"), Some(&cell_1.uuid));
        assert_eq!(scope.get("b"), Some(&cell_2.uuid));
        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_assign_add_code_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = a + 1", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        assert_eq!(scope.get("a"), Some(&cell_1.uuid));
        assert_eq!(scope.get("b"), Some(&cell_2.uuid));
        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_assign_add_two_code_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = a + c", &mut scope)?;
        let cell_3 = Cell::new_reactive("c = 1", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

        assert_eq!(scope.get("a"), Some(&cell_1.uuid));
        assert_eq!(scope.get("b"), Some(&cell_2.uuid));
        assert_eq!(scope.get("c"), Some(&cell_3.uuid));
        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_import_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("import numpy as np", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("p = np.pi", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_attr_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("import numpy as np", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("np.pi", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_list_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = [a, c]", &mut scope)?;
        let cell_3 = Cell::new_reactive("c = 2", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }
}
