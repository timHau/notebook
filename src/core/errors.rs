use std::{error::Error, fmt};

#[derive(Debug)]
pub enum TopologyErrors {
    CellNotFound,
    CycleDetected,
}

impl fmt::Display for TopologyErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TopologyErrors::CellNotFound => write!(f, "Cell not found"),
            TopologyErrors::CycleDetected => write!(f, "Cycle detected"),
        }
    }
}

impl Error for TopologyErrors {}

#[derive(Debug)]
pub enum NotebookErrors {
    NotYetImplemented,
    CellNotFound,
}

impl fmt::Display for NotebookErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotebookErrors::NotYetImplemented => write!(f, "Not yet implemented"),
            NotebookErrors::CellNotFound => write!(f, "Cell not found"),
        }
    }
}

impl Error for NotebookErrors {}
