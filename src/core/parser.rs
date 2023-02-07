use super::{
    cell::{Cell, CellType},
    topology::Topology,
};
use rustpython_parser::{
    ast::{ExprKind, Located, StmtKind},
    error::ParseError,
    parser,
};
use tracing::info;

pub struct Parser {}

impl Parser {
    pub fn parse(cell: &Cell) -> Result<Topology, ParseError> {
        match cell.cell_type {
            CellType::ReactiveCode => Self::parse_reactive_code(cell),
            CellType::NonReactiveCode => Self::parse_non_reactive_code(),
            CellType::Markdown => Self::parse_markdown(),
        }
    }

    fn parse_reactive_code(cell: &Cell) -> Result<Topology, ParseError> {
        info!("TODO evaluating reactive code");
        let ast = parser::parse_program(&cell.content, "<input>")?;

        let topology = Topology::new();
        for statement in ast.iter() {
            match &statement.node {
                StmtKind::Assign { targets, value, .. } => {
                    let assign_graph = Self::parse_assign(targets, value)?;
                }
                _ => todo!(),
            };
        }

        Ok(topology)
    }

    fn parse_assign(
        targets: &Vec<Located<ExprKind>>,
        value: &Located<ExprKind>,
    ) -> Result<Topology, ParseError> {
        info!("targets: {:#?}", targets);
        info!("value: {:#?}", value);

        match &value.node {
            ExprKind::Name { id, ctx } => {
                let topology = Topology::new();
                Ok(topology)
            }
            _ => Ok(Topology::default()),
        }
    }

    fn parse_non_reactive_code() -> Result<Topology, ParseError> {
        info!("TODO evaluating non-reactive code");
        Ok(Topology::default())
    }

    fn parse_markdown() -> Result<Topology, ParseError> {
        info!("TODO evaluating markdown");
        Ok(Topology::default())
    }
}
