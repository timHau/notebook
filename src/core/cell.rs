use super::{kernel_client::ExecutionType, notebook::Scope, statement::Statement};
use nanoid::nanoid;
use rustpython_parser::{
    ast::{AliasData, ExprContext, ExprKind, Located, StmtKind},
    error::ParseError,
    parser,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CellType {
    NonReactiveCode,
    ReactiveCode,
    Markdown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalValue {
    pub value: Value,
    pub local_type: ExecutionType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    pub metadata: CellMetadata,
    pub uuid: String,
    pub cell_type: CellType,
    pub content: String,

    #[serde(skip)]
    pub locals: HashMap<String, LocalValue>,

    #[serde(skip)]
    pub bindings: HashSet<String>,

    #[serde(skip)]
    pub required: HashSet<String>,

    #[serde(skip)]
    ignore_bindings: HashSet<String>,

    #[serde(skip)]
    pub statements: Vec<Statement>,
}

impl Cell {
    pub fn new(
        cell_type: CellType,
        content: String,
        scope: &mut Scope,
    ) -> Result<Self, ParseError> {
        let mut cell = Self {
            metadata: CellMetadata { collapsed: false },
            uuid: nanoid!(30),
            cell_type,
            content,
            locals: HashMap::new(),
            bindings: HashSet::new(),
            ignore_bindings: HashSet::new(),
            required: HashSet::new(),
            statements: Vec::new(),
        };

        cell.setup_local_vars(scope)?;

        Ok(cell)
    }

    pub fn new_reactive(content: &str, scope: &mut Scope) -> Result<Self, ParseError> {
        Self::new(CellType::ReactiveCode, String::from(content), scope)
    }

    fn unbind_all(&mut self) {
        self.bindings.clear();
        self.ignore_bindings.clear();
        self.required.clear();
        self.statements.clear();
    }

    pub fn update_content(&mut self, content: &str, scope: &mut Scope) -> Result<(), ParseError> {
        // remove old bindings from global scope
        for binding in self.bindings.iter() {
            scope.remove(binding);
        }

        match self.cell_type {
            CellType::ReactiveCode | CellType::NonReactiveCode => {
                // remove all local bindings
                self.unbind_all();
                // update content
                self.content = content.to_string();
                // rebind all new local bindings
                self.setup_local_vars(scope)
            }
            CellType::Markdown => Ok(warn!("TODO check Markdown cell")),
        }
    }

    pub fn setup_local_vars(&mut self, scope: &mut Scope) -> Result<(), ParseError> {
        match self.cell_type {
            CellType::ReactiveCode | CellType::NonReactiveCode => {
                let ast = parser::parse_program(&self.content, "<input>")?;

                for statement in ast.iter() {
                    self.handle_stmt_node(&statement, scope, true);
                }

                Ok(())
            }
            CellType::Markdown => Ok(warn!("TODO check Markdown cell")),
        }
    }

    fn import_dependencies(&mut self, names: &[Located<AliasData>], scope: &mut Scope) {
        for name in names {
            if let Some(alias) = &name.node.asname {
                scope.insert(alias.to_string(), self.uuid.clone());
                self.bindings.insert(alias.to_string());
            } else {
                let import_name = name.node.name.to_string();
                self.bindings.insert(import_name.clone());
                scope.insert(import_name, self.uuid.clone());
            }
        }
    }

    fn handle_stmt_node(&mut self, stmtKind: &Located<StmtKind>, scope: &mut Scope, is_root: bool) {
        if is_root {
            let start = stmtKind.location;
            let end = stmtKind.end_location.unwrap_or(start);
            let statement = match &stmtKind.node {
                StmtKind::Expr { .. } => Statement::new_eval(&start, &end, &self.content),

                StmtKind::Import { .. }
                | StmtKind::ImportFrom { .. }
                | StmtKind::FunctionDef { .. }
                | StmtKind::ClassDef { .. }
                | StmtKind::AsyncFunctionDef { .. } => {
                    Statement::new_definition(&start, &end, &self.content)
                }

                _ => Statement::new_exec(&start, &end, &self.content),
            };
            self.statements.push(statement);
        }

        // println!("statement: {:#?}", statement);
        match &stmtKind.node {
            StmtKind::Import { names } | StmtKind::ImportFrom { names, .. } => {
                self.import_dependencies(&names, scope)
            }

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
                    self.handle_stmt_node(&statement, scope, false);
                }
                for statement in orelse {
                    self.handle_stmt_node(&statement, scope, false);
                }
            }

            StmtKind::Match { subject, cases } => {
                self.handle_expr_node(&subject.node, scope);
                for case in cases {
                    // self.handle_expr_node(&case.pattern.node, scope);
                    // self.handle_expr_node(&case.guard.node, scope);
                    for statement in &case.body {
                        self.handle_stmt_node(&statement, scope, false);
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
                    self.handle_stmt_node(&statement, scope, false);
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
                println!("statement: {:#?}", stmtKind);
                self.handle_expr_node(&test.node, scope);
                for statement in body {
                    self.handle_stmt_node(&statement, scope, false);
                }
                for statement in orelse {
                    self.handle_stmt_node(&statement, scope, false);
                }
            }

            StmtKind::For { body, orelse, .. } => {
                // self.handle_expr_node(&target.node, scope);
                // self.handle_expr_node(&iter.node, scope);
                for statement in body {
                    self.handle_stmt_node(&statement, scope, false);
                }
                for statement in orelse {
                    self.handle_stmt_node(&statement, scope, false);
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
                    self.handle_stmt_node(&statement, scope, false);
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
            // StmtKind::Global { names } => todo!(),
            StmtKind::Nonlocal { .. } => {}
            StmtKind::Pass => {}
            StmtKind::Break => {}
            StmtKind::Continue => {}
            _ => warn!("Unsupported statement: {:#?}", stmtKind),
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
                self.required.insert(id.to_string());
            }
            ExprContext::Store => {
                // Example where we need the if
                // Given cell 1) a = 1 and cell 2) while True:\n  a += 1,
                if let Some(dep) = scope.get(id) {
                    if dep != &self.uuid {
                        self.required.insert(id.to_string());
                    }
                } else {
                    scope.insert(id.to_string(), self.uuid.clone());
                    self.bindings.insert(id.to_string());
                }
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

    #[test]
    fn setup_local_vars() {
        let mut scope = Scope::new();
        let cell = Cell::new_reactive("a = 1\nb = 2\nc = 3", &mut scope).unwrap();

        let expected_bindings = vec!["a", "b", "c"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(cell.bindings, expected_bindings);

        let expected_scope = vec![
            ("a", cell.uuid.clone()),
            ("b", cell.uuid.clone()),
            ("c", cell.uuid.clone()),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        assert_eq!(scope, expected_scope);
    }

    #[test]
    fn test_required_vars() {
        let mut scope = Scope::new();
        let cell_1 = Cell::new_reactive("a = 1\nb = 2\nc = 3", &mut scope).unwrap();

        let expected_required_1 = HashSet::new();
        assert_eq!(cell_1.required, expected_required_1);

        let expected_bindings_1 = vec!["a", "b", "c"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(cell_1.bindings, expected_bindings_1);

        let cell_2 = Cell::new_reactive("a + b + c", &mut scope).unwrap();

        let expected_required_2 = vec!["a", "b", "c"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(cell_2.required, expected_required_2);

        let expected_bindings_2 = HashSet::new();
        assert_eq!(cell_2.bindings, expected_bindings_2);
    }
}
