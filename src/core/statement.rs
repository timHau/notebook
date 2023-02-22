use base64::{engine::general_purpose, Engine as _};
use rustpython_parser::ast::Location;
use tracing::info;

use super::kernel_client::ExecutionType;

#[derive(Debug, Clone)]
pub struct Statement {
    pub execution_type: ExecutionType,
    pub content: String,
}

impl Statement {
    pub fn new_exec(start: &Location, end: &Location, content: &str) -> Self {
        let content = Self::extract_content(start, end, content);
        Self {
            execution_type: ExecutionType::Exec,
            content,
        }
    }

    pub fn new_eval(start: &Location, end: &Location, content: &str) -> Self {
        let content = Self::extract_content(start, end, content);
        Self {
            execution_type: ExecutionType::Eval,
            content,
        }
    }

    pub fn new_definition(start: &Location, end: &Location, content: &str) -> Self {
        let content = Self::extract_content(start, end, content);
        let content_bytes = content.as_bytes();
        let content = general_purpose::STANDARD.encode(content_bytes);
        Self {
            execution_type: ExecutionType::Definition,
            content,
        }
    }

    fn extract_content(start: &Location, end: &Location, content: &str) -> String {
        let (row_start, row_end) = (start.row() - 1, end.row() - 1);

        // extract content from all_content
        let content = content
            .lines()
            .skip(row_start)
            .take(row_end - row_start + 1)
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        content.join("\n")
    }
}
