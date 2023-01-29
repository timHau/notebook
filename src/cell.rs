use crate::notebook::Notebook;
use nanoid::nanoid;
use num_bigint::BigInt;
use rustpython_parser::{
    ast::{self, Constant, ExprKind, Located},
    parser,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
enum VarValue {
    Int(BigInt),
    Float(f64),
    String(String),
    Bool(bool),
    Complex { real: f64, imag: f64 },
    Ref,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    metadata: CellMetadata,
    pub uuid: String,
    pub cell_type: CellType,
    pub content: String,
    scope: HashMap<String, VarValue>,
    pos: usize,
}

impl Cell {
    pub fn new(cell_type: CellType, content: String, pos: usize) -> Self {
        Self {
            metadata: CellMetadata::default(),
            uuid: nanoid!(30),
            cell_type,
            content,
            scope: HashMap::new(),
            pos,
        }
    }

    pub fn update_content(&mut self, content: &str) {
        self.content = content.to_string();
    }

    fn eval_expr(&mut self, value: &Box<Located<ExprKind>>) {
        println!("value: {:#?}", value.node);
    }

    fn eval_assign(&mut self, targets: &Vec<Located<ExprKind>>, value: &Box<Located<ExprKind>>) {
        println!("---------------------------------------");
        for target in targets.iter() {
            match &target.node {
                ExprKind::Name { id, .. } => match &value.node {
                    ExprKind::BinOp { left, op, right } => {
                        println!("left: {:#?}", left.node);
                        println!("op: {:#?}", op);
                        println!("right: {:#?}", right.node);
                    }
                    ExprKind::Constant { value, .. } => self.handle_constant_assign(id, value),
                    ExprKind::Name { id, ctx } => {
                        println!("id: {:#?}", id);
                        println!("ctx: {:#?}", ctx);
                        match ctx {
                            ast::ExprContext::Store => {}
                            ast::ExprContext::Load => {}
                            ast::ExprContext::Del => {}
                        }
                    }
                    _ => {
                        println!("TODO [eval_assign]");
                        println!("target: {:#?}", target.node);
                        println!("value: {:#?}", value.node);
                    }
                },
                _ => {}
            }
        }
        println!("---------------------------------------");
        println!("scope: {:#?}", self.scope);
    }

    fn handle_constant_assign(&mut self, id: &str, value: &Constant) {
        match value {
            Constant::Int(value) => {
                self.scope
                    .insert(id.to_string(), VarValue::Int(value.clone()));
            }
            Constant::Float(value) => {
                self.scope
                    .insert(id.to_string(), VarValue::Float(value.clone()));
            }
            Constant::Bool(value) => {
                self.scope
                    .insert(id.to_string(), VarValue::Bool(value.clone()));
            }
            Constant::Str(value) => {
                self.scope
                    .insert(id.to_string(), VarValue::String(value.clone()));
            }
            Constant::Complex { real, imag } => {
                self.scope.insert(
                    id.to_string(),
                    VarValue::Complex {
                        real: real.clone(),
                        imag: imag.clone(),
                    },
                );
            }
            Constant::Tuple(value) => {
                println!("value: {:#?}", value);
            }
            _ => {
                println!("[TODO] value: {:#?}", value);
            }
        }
    }

    pub fn eval(&mut self) {
        let ast = parser::parse_program(&self.content, "<input>").unwrap();
        for statement in ast.iter() {
            // println!("statement: {:#?}", statement.node);
            match &statement.node {
                ast::StmtKind::Assign {
                    targets,
                    value,
                    ..
                } => self.eval_assign(targets, value),
                ast::StmtKind::Expr { value } => self.eval_expr(value),
                ast::StmtKind::FunctionDef {
                    ..
                    // name,
                    // args,
                    // body,
                    // decorator_list,
                    // returns,
                    // type_comment,
                } => {
                    // println!("name: {}", name);
                    // println!("args: {:#?}", args);
                    // println!("body: {:#?}", body);
                    // println!("decorator_list: {:#?}", decorator_list);
                    // println!("returns: {:#?}", returns);
                    // println!("type_comment: {:#?}", type_comment);
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CellMetadata {
    pub collapsed: bool,
}

impl Default for CellMetadata {
    fn default() -> Self {
        Self { collapsed: false }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CellType {
    NonReactiveCode,
    ReactiveCode,
    Markdown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Dependent {
    id: String,
}
