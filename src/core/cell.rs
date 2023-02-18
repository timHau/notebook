use super::{notebook::Scope, statement_pos::StatementPos};
use nanoid::nanoid;
use pyo3::{prelude::*, types::PyDict};
use rustpython_parser::{
    ast::{AliasData, ExprContext, ExprKind, Located, StmtKind},
    error::ParseError,
    parser,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};
use tracing::{info, log::warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CellType {
    NonReactiveCode,
    ReactiveCode,
    Markdown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    pub metadata: CellMetadata,
    pub uuid: String,
    pub cell_type: CellType,
    pub content: String,
    pub dependencies: HashSet<String>,
    pub dependents: HashSet<String>,

    #[serde(skip)]
    pub statements: Vec<StatementPos>,

    #[serde(skip)]
    pub locals: Option<Py<PyDict>>,

    #[serde(skip)]
    pub bindings: HashSet<String>,

    #[serde(skip)]
    ignore_bindings: HashSet<String>,
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
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            statements: Vec::new(),
            bindings: HashSet::new(),
            ignore_bindings: HashSet::new(),
        };

        cell.build_dependencies(scope)?;

        Ok(cell)
    }

    pub fn new_reactive(content: &str, scope: &mut Scope) -> Result<Self, Box<dyn Error>> {
        Self::new(CellType::ReactiveCode, String::from(content), scope)
    }

    pub fn build_dependents(&self, cells: &mut HashMap<String, Cell>) {
        for uuid in self.dependencies.iter() {
            if let Some(dependency) = cells.get_mut(uuid) {
                dependency.dependents.insert(self.uuid.clone());
            }
        }
    }

    pub fn build_dependencies(&mut self, scope: &mut Scope) -> Result<(), ParseError> {
        match self.cell_type {
            CellType::ReactiveCode | CellType::NonReactiveCode => self.code_dependencies(scope),
            CellType::Markdown => Ok(warn!("TODO check Markdown cell")),
        }
    }

    fn code_dependencies(&mut self, scope: &mut Scope) -> Result<(), ParseError> {
        let ast = parser::parse_program(&self.content, "<input>")?;

        for statement in ast.iter() {
            self.handle_stmt_node(&statement, scope);
        }

        Ok(())
    }

    fn import_dependencies(&mut self, names: &[Located<AliasData>], scope: &mut Scope) {
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

    fn handle_stmt_node(&mut self, statement: &Located<StmtKind>, scope: &mut Scope) {
        let start = statement.location;
        let end = statement.end_location.unwrap_or(start);
        let statement_pos = match &statement.node {
            StmtKind::Expr { .. } => StatementPos::eval_from(&start, &end),
            _ => StatementPos::exec_from(&start, &end),
        };

        let mut has_intersection = false;
        for pos in self.statements.iter() {
            if pos.intersects(&statement_pos) {
                has_intersection = true;
                break;
            }
        }
        if !has_intersection {
            self.statements.push(statement_pos);
        }

        match &statement.node {
            StmtKind::Import { names } => self.import_dependencies(&names, scope),

            StmtKind::Assign { targets, value, .. } => {
                for target in targets.iter() {
                    self.handle_expr_node(&target.node, scope);
                }
                self.handle_expr_node(&value.node, scope);
            }

            StmtKind::Expr { value } => self.handle_expr_node(&value.node, scope),

            StmtKind::AugAssign { target, value, .. } => {
                self.handle_expr_node(&target.node, scope);
                self.handle_expr_node(&value.node, scope);
            }

            StmtKind::Return { value } => {
                if let Some(value) = value {
                    self.handle_expr_node(&value.node, scope);
                }
            }

            StmtKind::If { test, body, orelse } => {
                self.handle_expr_node(&test.node, scope);
                for statement in body {
                    self.handle_stmt_node(&statement, scope);
                }
                for statement in orelse {
                    self.handle_stmt_node(&statement, scope);
                }
            }

            StmtKind::Match { subject, cases } => {
                self.handle_expr_node(&subject.node, scope);
                for case in cases {
                    // self.handle_expr_node(&case.pattern.node, scope);
                    // self.handle_expr_node(&case.guard.node, scope);
                    for statement in &case.body {
                        self.handle_stmt_node(&statement, scope);
                    }
                }
            }

            StmtKind::FunctionDef {
                name, body, args, ..
            }
            | StmtKind::AsyncFunctionDef {
                name, body, args, ..
            } => {
                for arg in args.args.iter() {
                    // arguments of a function might be named the same as bindings in another cell
                    // we do not want to add these to the dependencies
                    self.ignore_bindings.insert(arg.node.arg.to_string());
                }

                scope.insert(name.to_string(), self.uuid.clone());
                for statement in body {
                    self.handle_stmt_node(&statement, scope);
                }
            }

            StmtKind::AnnAssign {
                target,
                annotation,
                value,
                ..
            } => {
                self.handle_expr_node(&target.node, scope);
                self.handle_expr_node(&annotation.node, scope);
                if let Some(value) = value {
                    self.handle_expr_node(&value.node, scope);
                }
            }

            StmtKind::While { test, body, orelse } => {
                println!("statement: {:#?}", statement);
                self.handle_expr_node(&test.node, scope);
                for statement in body {
                    self.handle_stmt_node(&statement, scope);
                }
                for statement in orelse {
                    self.handle_stmt_node(&statement, scope);
                }
            }

            StmtKind::For { body, orelse, .. } => {
                // self.handle_expr_node(&target.node, scope);
                // self.handle_expr_node(&iter.node, scope);
                for statement in body {
                    self.handle_stmt_node(&statement, scope);
                }
                for statement in orelse {
                    self.handle_stmt_node(&statement, scope);
                }
            }

            StmtKind::ClassDef {
                name,
                bases,
                body,
                decorator_list,
                ..
            } => {
                scope.insert(name.to_string(), self.uuid.clone());
                for base in bases {
                    self.handle_expr_node(&base.node, scope);
                }
                for statement in body {
                    self.handle_stmt_node(&statement, scope);
                }
                for decorator in decorator_list {
                    self.handle_expr_node(&decorator.node, scope);
                }
            }

            // StmtKind::Delete { targets } => todo!(),
            // StmtKind::AsyncFor { target, iter, body, orelse, type_comment } => todo!(),
            // StmtKind::With { items, body, type_comment } => todo!(),
            // StmtKind::AsyncWith { items, body, type_comment } => todo!(),
            // StmtKind::Raise { exc, cause } => todo!(),
            // StmtKind::Try { body, handlers, orelse, finalbody } => todo!(),
            // StmtKind::Assert { test, msg } => todo!(),
            // StmtKind::ImportFrom { module, names, level } => todo!(),
            // StmtKind::Global { names } => todo!(),
            StmtKind::Nonlocal { .. } => {}
            StmtKind::Pass => {}
            StmtKind::Break => {}
            StmtKind::Continue => {}
            _ => warn!("Unsupported statement: {:#?}", statement),
        };
    }

    fn handle_expr_node(&mut self, expr: &ExprKind, scope: &mut Scope) {
        match expr {
            ExprKind::Name { id, ctx } => self.handle_name_dep(id, ctx, scope),

            ExprKind::BinOp { left, right, .. } => {
                self.handle_expr_node(&left.node, scope);
                self.handle_expr_node(&right.node, scope);
            }

            ExprKind::List { elts, .. } | ExprKind::Tuple { elts, .. } | ExprKind::Set { elts } => {
                for elt in elts {
                    self.handle_expr_node(&elt.node, scope);
                }
            }

            ExprKind::Constant { .. } => {}

            ExprKind::UnaryOp { operand, .. } => {
                self.handle_expr_node(&operand.node, scope);
            }

            ExprKind::BoolOp { values, .. } | ExprKind::JoinedStr { values } => {
                for value in values {
                    self.handle_expr_node(&value.node, scope);
                }
            }

            ExprKind::NamedExpr { target, value } => {
                self.handle_expr_node(&target.node, scope);
                self.handle_expr_node(&value.node, scope);
            }

            ExprKind::IfExp { test, body, orelse } => {
                self.handle_expr_node(&test.node, scope);
                self.handle_expr_node(&body.node, scope);
                self.handle_expr_node(&orelse.node, scope);
            }

            ExprKind::Compare {
                left, comparators, ..
            } => {
                self.handle_expr_node(&left.node, scope);
                for comparator in comparators {
                    self.handle_expr_node(&comparator.node, scope);
                }
            }

            ExprKind::Subscript { value, slice, .. } => {
                self.handle_expr_node(&value.node, scope);
                self.handle_expr_node(&slice.node, scope);
            }

            ExprKind::Slice { lower, upper, step } => {
                if let Some(lower) = lower {
                    self.handle_expr_node(&lower.node, scope);
                }
                if let Some(upper) = upper {
                    self.handle_expr_node(&upper.node, scope);
                }
                if let Some(step) = step {
                    self.handle_expr_node(&step.node, scope);
                }
            }

            ExprKind::FormattedValue {
                value, format_spec, ..
            } => {
                self.handle_expr_node(&value.node, scope);
                if let Some(format_spec) = format_spec {
                    self.handle_expr_node(&format_spec.node, scope);
                }
            }

            ExprKind::Dict { keys, values } => {
                for key in keys {
                    self.handle_expr_node(&key.node, scope);
                }
                for value in values {
                    self.handle_expr_node(&value.node, scope);
                }
            }

            ExprKind::ListComp { elt, generators }
            | ExprKind::SetComp { elt, generators }
            | ExprKind::GeneratorExp { elt, generators } => {
                self.handle_expr_node(&elt.node, scope);
                for generator in generators {
                    self.handle_expr_node(&generator.iter.node, scope);
                    for if_expr in &generator.ifs {
                        self.handle_expr_node(&if_expr.node, scope);
                    }
                }
            }

            ExprKind::DictComp {
                key,
                value,
                generators,
            } => {
                self.handle_expr_node(&key.node, scope);
                self.handle_expr_node(&value.node, scope);
                for generator in generators {
                    self.handle_expr_node(&generator.iter.node, scope);
                    for if_expr in &generator.ifs {
                        self.handle_expr_node(&if_expr.node, scope);
                    }
                }
            }

            ExprKind::Lambda { body, .. } => {
                self.handle_expr_node(&body.node, scope);
            }

            ExprKind::Await { value }
            | ExprKind::YieldFrom { value }
            | ExprKind::Attribute { value, .. }
            | ExprKind::Starred { value, .. } => self.handle_expr_node(&value.node, scope),

            ExprKind::Call { func, args, .. } => {
                self.handle_expr_node(&func.node, scope);
                for arg in args {
                    self.handle_expr_node(&arg.node, scope)
                }
            }

            ExprKind::Yield { value } => {
                if let Some(value) = value {
                    self.handle_expr_node(&value.node, scope);
                }
            }
        }
    }

    fn handle_name_dep(&mut self, id: &str, ctx: &ExprContext, scope: &mut Scope) {
        if self.ignore_bindings.contains(id) {
            return;
        }

        match ctx {
            ExprContext::Load => {
                if let Some(dep) = scope.get(id) {
                    if dep != &self.uuid {
                        self.dependencies.insert(dep.to_string());
                    }
                }
            }
            ExprContext::Store => {
                // Given cell 1) a = 1 and cell 2) while True:\n  a += 1,
                if let Some(dep) = scope.get(id) {
                    if dep != &self.uuid {
                        self.dependencies.insert(dep.to_string());
                    }
                }
                scope.insert(id.to_string(), self.uuid.clone());
                self.bindings.insert(id.to_string());
            }
            ExprContext::Del => {}
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CellMetadata {
    pub collapsed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::topology::Topology;
    use std::error::Error;

    #[test]
    fn test_trivial_code_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let mut cell = Cell::new_reactive("a = 1", &mut scope)?;
        cell.build_dependencies(&mut scope)?;

        let expect = HashSet::new();

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

    #[test]
    fn test_tuple_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = (a, c)", &mut scope)?;
        let cell_3 = Cell::new_reactive("c = 2", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_set_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = {a, c}", &mut scope)?;
        let cell_3 = Cell::new_reactive("c = 2", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_unary_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = -a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_boolop_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let cell_2 = Cell::new_reactive("b = 2", &mut scope)?;
        let mut cell_3 = Cell::new_reactive("c = a and b", &mut scope)?;

        cell_3.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

        Ok(assert_eq!(cell_3.dependencies, expect))
    }

    #[test]
    fn test_namedexpr_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("(b := a)", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_ifexpr_dependencies_1() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let cell_2 = Cell::new_reactive("b = 2", &mut scope)?;
        let mut cell_3 = Cell::new_reactive("c = a if b else 3", &mut scope)?;

        cell_3.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

        Ok(assert_eq!(cell_3.dependencies, expect))
    }

    #[test]
    fn test_ifexpr_dependencies_2() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let cell_2 = Cell::new_reactive("b = 2", &mut scope)?;
        let mut cell_3 = Cell::new_reactive("c = a if 3 else b", &mut scope)?;

        cell_3.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

        Ok(assert_eq!(cell_3.dependencies, expect))
    }

    #[test]
    fn test_compare_dependencies_1() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let cell_2 = Cell::new_reactive("b = 2", &mut scope)?;
        let mut cell_3 = Cell::new_reactive("c = a < b", &mut scope)?;

        cell_3.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

        Ok(assert_eq!(cell_3.dependencies, expect))
    }

    #[test]
    fn test_compare_dependencies_2() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let cell_2 = Cell::new_reactive("b = 2", &mut scope)?;
        let mut cell_3 = Cell::new_reactive("c = a >= b", &mut scope)?;

        cell_3.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

        Ok(assert_eq!(cell_3.dependencies, expect))
    }

    #[test]
    fn test_slice_lower_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("c = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("a = [1, 2, 3]\nb = a[c:]", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_slice_upper_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("c = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("a = [1, 2, 3]\nb = a[:c]", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_slice_step_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("c = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("a = [1, 2, 3]\nb = a[0:c:2]", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_formattedvalue_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = f'{a}'", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_joinedstr_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = f'{a}' + 'a'", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_dict_dependencies_1() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = {a: 1}", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_dict_dependencies_2() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = {1: a}", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_listcomp_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = [a for _ in [1, 2, 3]]", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_setcomp_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = {a for _ in [1, 2, 3]}", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_listcomp_if_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = [a for _ in [1, 2, 3] if c > 1]", &mut scope)?;
        let cell_3 = Cell::new_reactive("c = 2", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_listcomp_mult_if_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 =
            Cell::new_reactive("b = [a for _ in [1, 2, 3] if c > 1 if d > 2]", &mut scope)?;
        let cell_3 = Cell::new_reactive("c = 2", &mut scope)?;
        let cell_4 = Cell::new_reactive("d = 3", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([
            cell_1.uuid.to_string(),
            cell_3.uuid.to_string(),
            cell_4.uuid.to_string(),
        ]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_dictcomp_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = {i: a for i in [1, 2, 3]}", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_dictcomp_dependencies_2() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = {a: 1 for i in [1, 2, 3]}", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_dictcomp_if_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = {2: 1 for i in [1, 2, 3] if a > 0}", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_lambda_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = lambda x: a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_call_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = lambda x: 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = a(1)", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_call_dependencies_2() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = lambda x: 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = a(c)", &mut scope)?;
        let cell_3 = Cell::new_reactive("c = 1", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_generatorexp_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = (a for _ in [1, 2, 3])", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_await_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = await a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_yield_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = yield a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_yieldfrom_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = yield from a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_starred_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b = [*a]", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_build_dependents() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = b + 1", &mut scope)?;
        let cell_2 = Cell::new_reactive("b = 1", &mut scope)?;

        let mut topology = Topology::from_vec(vec![&cell_1, &cell_2], &mut scope)?;
        topology.build(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);
        let c_2 = topology.cells.get(&cell_2.uuid).unwrap();
        Ok(assert_eq!(c_2.dependents, expect))
    }

    #[test]
    fn test_build_dependents_2() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = b + c", &mut scope)?;
        let cell_2 = Cell::new_reactive("b = 1", &mut scope)?;
        let cell_3 = Cell::new_reactive("c = 2", &mut scope)?;

        let mut topology = Topology::from_vec(vec![&cell_1, &cell_2, &cell_3], &mut scope)?;
        topology.build(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);
        let c_2 = topology.cells.get(&cell_2.uuid).unwrap();
        let c_3 = topology.cells.get(&cell_3.uuid).unwrap();
        assert_eq!(c_2.dependents, expect);
        Ok(assert_eq!(c_3.dependents, expect))
    }

    #[test]
    fn test_augassign_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b += a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_fndef_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("def b(c, d): return a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_asyncfndef_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("async def b(c, d): return a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_annassign_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("b: int = a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_while_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("while a: pass", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_while_dependencies_2() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("while True:\n  a += 1", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_forloop_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = [1, 2, 3]", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("for i in a: pass", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_forloop_dependencies_2() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = [1, 2, 3]", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("for i in [4,5,6]: a", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_forloop_dependencies_3() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = [1, 2, 3]", &mut scope)?;
        let mut cell_2 = Cell::new_reactive("for i in [4,5,6]:\n  for j in a: j", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_class_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let mut cell_2 = Cell::new_reactive(
            "class b:\n  def __init__(self):\n    self.a = a",
            &mut scope,
        )?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_class_dependencies_2() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("class a:\n  def __init__(self): pass", &mut scope)?;
        let mut cell_2 = Cell::new_reactive(
            "class b:\n  def __init__(self):\n    self.a = a",
            &mut scope,
        )?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }

    #[test]
    fn test_class_dependencies_inheritance() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("class a:\n  def __init__(self): pass", &mut scope)?;
        let mut cell_2 =
            Cell::new_reactive("class b(a):\n  def __init__(self):\n    pass", &mut scope)?;

        cell_2.build_dependencies(&mut scope)?;

        let expect = HashSet::from([cell_1.uuid.to_string()]);

        Ok(assert_eq!(cell_2.dependencies, expect))
    }
}
