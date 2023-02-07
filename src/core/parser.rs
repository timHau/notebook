use super::cell::{Cell, CellType};
use rustpython_parser::{
    ast::{ExprKind, Located, StmtKind},
    error::ParseError,
    parser,
};
use tracing::info;

pub struct Parser {}

impl Parser {
    pub fn parse(cell: &Cell) -> Result<(), ParseError> {
        match cell.cell_type {
            CellType::ReactiveCode => Self::parse_reactive_code(cell),
            CellType::NonReactiveCode => Self::parse_non_reactive_code(),
            CellType::Markdown => Self::parse_markdown(),
        }
    }

    fn parse_reactive_code(cell: &Cell) -> Result<(), ParseError> {
        info!("TODO evaluating reactive code");
        let ast = parser::parse_program(&cell.content, "<input>")?;

        for statement in ast.iter() {
            match &statement.node {
                StmtKind::Assign { targets, value, .. } => {
                    Self::parse_assign(targets, value);
                }
                _ => todo!(),
            }
        }

        Ok(())
    }

    fn parse_assign(
        targets: &Vec<Located<ExprKind>>,
        value: &Located<ExprKind>,
    ) -> Result<(), ParseError> {
        for target in targets.iter() {
            info!("target: {:#?}", target);
        }
        Ok(())
    }

    fn parse_non_reactive_code() -> Result<(), ParseError> {
        info!("TODO evaluating non-reactive code");
        Ok(())
    }

    fn parse_markdown() -> Result<(), ParseError> {
        info!("TODO evaluating markdown");
        Ok(())
    }
}
