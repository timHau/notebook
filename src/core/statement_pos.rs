use itertools::Itertools;
use rustpython_parser::ast::Location;

#[derive(Debug, Clone)]
pub enum ExecutionType {
    Exec,
    Eval,
}

#[derive(Debug, Clone)]
pub struct StatementPos {
    pub row_start: usize,
    pub row_end: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub execution_type: ExecutionType,
}

impl StatementPos {
    pub fn exec_from(start: &Location, end: &Location) -> Self {
        Self {
            row_start: start.row() - 1,
            row_end: end.row() - 1,
            col_start: start.column(),
            col_end: end.column(),
            execution_type: ExecutionType::Exec,
        }
    }

    pub fn eval_from(start: &Location, end: &Location) -> Self {
        Self {
            row_start: start.row() - 1,
            row_end: end.row() - 1,
            col_start: start.column(),
            col_end: end.column(),
            execution_type: ExecutionType::Eval,
        }
    }

    pub fn intersects(&self, other: &StatementPos) -> bool {
        let row = self.row_start..=self.row_end;
        let col = self.col_start..=self.col_end;

        (row.contains(&other.row_start)
            && (col.contains(&other.col_start) || col.contains(&other.col_end)))
            || (row.contains(&other.row_start)
                && (col.contains(&other.col_start) || col.contains(&other.col_end)))
    }

    pub fn extract_code(&self, code: &str) -> String {
        let lines = code
            .lines()
            .skip(self.row_start)
            .take(self.row_end - self.row_start + 1)
            .map(|c| c.to_string())
            .collect::<Vec<_>>();

        lines.iter().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_intersection() {
        // 1......2.1.....2
        let pos_1 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 0,
            row_end: 0,
            col_start: 0,
            col_end: 10,
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 0,
            row_end: 0,
            col_start: 8,
            col_end: 15,
        };
        assert!(pos_1.intersects(&pos_2));
        assert!(pos_2.intersects(&pos_1));
    }

    #[test]
    fn test_pos_no_intersection() {
        let pos_1 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 0,
            row_end: 0,
            col_start: 10,
            col_end: 10,
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 11,
            row_end: 13,
            col_start: 15,
            col_end: 15,
        };
        assert!(!pos_1.intersects(&pos_2));
        assert!(!pos_2.intersects(&pos_1));
    }

    #[test]
    fn test_pos_same_pos() {
        let pos_1 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 0,
            row_end: 0,
            col_start: 10,
            col_end: 10,
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 0,
            row_end: 0,
            col_start: 10,
            col_end: 10,
        };
        assert!(pos_1.intersects(&pos_2));
        assert!(pos_2.intersects(&pos_1));
    }

    #[test]
    fn test_pos_no_intersection_same_col() {
        // 1........1
        // 2........2
        let pos_1 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 0,
            row_end: 10,
            col_start: 0,
            col_end: 0,
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 12,
            row_end: 15,
            col_start: 0,
            col_end: 0,
        };
        assert!(!pos_1.intersects(&pos_2));
        assert!(!pos_2.intersects(&pos_1));
    }

    #[test]
    fn test_pos_no_intersection_same_row() {
        // 1........1.2...2
        let pos_1 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 0,
            row_end: 0,
            col_start: 0,
            col_end: 10,
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 0,
            row_end: 0,
            col_start: 12,
            col_end: 15,
        };
        assert!(!pos_1.intersects(&pos_2));
        assert!(!pos_2.intersects(&pos_1));
    }

    #[test]
    fn test_extract_code() {
        let code = "a = 1
b = 2
c = 3
d = 4
";
        let pos = StatementPos {
            execution_type: ExecutionType::Exec,
            row_start: 1,
            row_end: 2,
            col_start: 0,
            col_end: 5,
        };

        assert_eq!(pos.extract_code(code), "b = 2\nc = 3".to_string());
    }
}
