use super::kernel::Kernel;
use nanoid::nanoid;
use pyo3::{prelude::*, types::PyDict};
use rustpython_parser::{
    ast::{AliasData, ExprContext, ExprKind, Located, StmtKind},
    error::ParseError,
    parser,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, vec};
use tracing::{info, log::warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    pub metadata: CellMetadata,
    pub uuid: String,
    pub cell_type: CellType,
    pub content: String,
    pub pos: usize,
    pub dependencies: Vec<String>,

    #[serde(skip)]
    locals: Option<Py<PyDict>>,
    #[serde(skip)]
    pub bindings: HashSet<String>,
}

impl Cell {
    pub fn new(cell_type: CellType, content: String, pos: usize) -> Self {
        Self {
            metadata: CellMetadata { collapsed: false },
            uuid: nanoid!(30),
            cell_type,
            content,
            pos,
            locals: Some(Python::with_gil(|py| PyDict::new(py).into())),
            dependencies: vec![],
            bindings: HashSet::new(),
        }
    }

    pub fn eval(&mut self, kernel: &mut Kernel) {
        let locals = self.locals.as_mut().unwrap();
        kernel.eval(&self.content, locals);
    }

    pub fn build_dependencies(&mut self, cells: &Vec<Cell>) -> Result<Vec<String>, ParseError> {
        match self.cell_type {
            CellType::ReactiveCode | CellType::NonReactiveCode => self.code_dependencies(cells),
            CellType::Markdown => todo!(),
        }
    }

    fn code_dependencies(&mut self, cells: &Vec<Cell>) -> Result<Vec<String>, ParseError> {
        let ast = parser::parse_program(&self.content, "<input>")?;

        let mut dependencies = vec![];
        for statement in ast.iter() {
            match &statement.node {
                StmtKind::Import { names } => {
                    self.import_dependencies(names, cells, &mut dependencies)
                }
                StmtKind::Assign { targets, value, .. } => {
                    self.assign_dependencies(targets, value, cells, &mut dependencies)
                }
                _ => warn!("Unsupported statement: {:#?}", statement),
            };
        }

        Ok(dependencies)
    }

    fn assign_dependencies(
        &mut self,
        targets: &Vec<Located<ExprKind>>,
        value: &Located<ExprKind>,
        cells: &Vec<Cell>,
        dep_topology: &mut Vec<String>,
    ) {
        for target in targets {
            match &target.node {
                ExprKind::Name { id, ctx, .. } => {
                    self.handle_name_dep(id, cells, ctx, dep_topology)
                }
                _ => warn!("Unsupported assign target: {:#?}", target),
            }
        }

        match &value.node {
            ExprKind::Call { func, args, .. } => {
                match &func.node {
                    ExprKind::Attribute { value, .. } => match &value.node {
                        ExprKind::Name { id, ctx, .. } => {
                            self.handle_name_dep(id, cells, ctx, dep_topology)
                        }
                        _ => warn!("Unsupported value assign value: {:#?}", value),
                    },
                    _ => warn!("Unsupported func assign value: {:#?}", value),
                }

                for arg in args {
                    match &arg.node {
                        ExprKind::Constant { .. } => {}
                        ExprKind::BinOp { left, right, .. } => {}
                        _ => warn!("Unsupported arg assign value: {:#?}", value),
                    }
                }
            }
            ExprKind::Name { id, ctx, .. } => self.handle_name_dep(id, cells, ctx, dep_topology),
            _ => warn!("Unsupported assign value: {:#?}", value),
        }
    }

    fn import_dependencies(
        &mut self,
        names: &Vec<Located<AliasData>>,
        _cells: &Vec<Cell>,
        _dep_topology: &mut Vec<String>,
    ) {
        info!("Import statement: {:#?}", names);
        for name in names {
            if let Some(alias) = &name.node.asname {
                info!("alias: {:#?}", alias);
                self.bindings.insert(alias.to_string());
            } else {
                let import_name = name.node.name.to_string();
                info!("name: {:#?}", import_name);
                self.bindings.insert(import_name);
            }
        }
    }

    fn handle_name_dep(
        &mut self,
        id: &str,
        cells: &Vec<Cell>,
        ctx: &ExprContext,
        dep_topology: &mut Vec<String>,
    ) {
        match ctx {
            ExprContext::Load => {
                if let Some(dep) = self.find_dep_in_cells(id, cells) {
                    dep_topology.push(dep);
                }
            }
            ExprContext::Store => {
                self.bindings.insert(id.to_string());
            }
            ExprContext::Del => {}
        }
    }

    fn find_dep_in_cells(&self, name: &str, cells: &Vec<Cell>) -> Option<String> {
        for cell in cells.iter() {
            if cell.uuid == self.uuid {
                continue;
            }

            if cell.bindings.contains(name) {
                return Some(cell.uuid.clone());
            }
        }
        None
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

    #[test]
    fn test_trivial_code_dependencies() {
        let mut cell = Cell::new(CellType::ReactiveCode, "a = 1".to_string(), 0);
        let deps = cell.build_dependencies(&vec![]).unwrap();
        let expect: Vec<String> = vec![];
        assert_eq!(deps, expect);
    }

    #[test]
    fn test_assign_code_dependencies() {
        let mut cell_1 = Cell::new(CellType::ReactiveCode, "a = 1".to_string(), 0);
        let mut cell_2 = Cell::new(CellType::ReactiveCode, "b = a".to_string(), 1);
        let _deps_1 = cell_1.build_dependencies(&vec![cell_2.clone()]).unwrap();
        let deps = cell_2.build_dependencies(&vec![cell_1.clone()]).unwrap();
        let expect: Vec<String> = vec![cell_1.uuid.to_string()];
        assert_eq!(deps, expect);
    }
}
