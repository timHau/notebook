use nanoid::nanoid;
use rustpython_parser::{
    ast::{ExprKind, Located, StmtKind},
    error::ParseError,
    parser,
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    metadata: CellMetadata,
    pub uuid: String,
    pub cell_type: CellType,
    pub content: String,
    pub pos: usize,
}

impl Cell {
    pub fn new(cell_type: CellType, content: String, pos: usize) -> Self {
        Self {
            metadata: CellMetadata::default(),
            uuid: nanoid!(30),
            cell_type,
            content,
            pos,
        }
    }

    pub fn parse(&self) -> Result<(), ParseError> {
        match self.cell_type {
            CellType::ReactiveCode => self.parse_reactive_code(),
            _ => todo!(),
        }
    }

    fn parse_reactive_code(&self) -> Result<(), ParseError> {
        info!("TODO evaluating reactive code");
        let ast = parser::parse_program(&self.content, "<input>")?;

        for statement in ast.iter() {
            match &statement.node {
                StmtKind::Assign { targets, value, .. } => {
                    Self::parse_assign(targets, value)?;
                }
                _ => todo!(),
            };
        }

        Ok(())
    }

    fn parse_assign(
        targets: &Vec<Located<ExprKind>>,
        value: &Located<ExprKind>,
    ) -> Result<(), ParseError> {
        info!("targets: {:#?}", targets);
        info!("value: {:#?}", value);

        match &value.node {
            ExprKind::Name { id, ctx } => Ok(()),
            _ => Ok(()),
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
