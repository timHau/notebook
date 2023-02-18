use itertools::Itertools;
use rustpython_parser::ast::Location;

#[derive(Debug, Clone)]
pub enum ExecutionType {
    Exec,
    Eval,
}

#[derive(Debug, Clone)]
pub struct StatementPos {
    pub row: [usize; 2], // row_start, row_end
    pub col: [usize; 2], // col_start, col_end
    pub execution_type: ExecutionType,
}

impl StatementPos {
    pub fn exec_from(start: &Location, end: &Location) -> Self {
        Self {
            row: [start.row() - 1, end.row() - 1],
            col: [start.column(), end.column()],
            execution_type: ExecutionType::Exec,
        }
    }

    pub fn eval_from(start: &Location, end: &Location) -> Self {
        Self {
            row: [start.row() - 1, end.row() - 1],
            col: [start.column(), end.column()],
            execution_type: ExecutionType::Eval,
        }
    }

    pub fn intersects(&self, other: &StatementPos) -> bool {
        let row = self.row[0]..=self.row[1];
        let col = self.col[0]..=self.col[1];

        (row.contains(&other.row[0])
            && (col.contains(&other.col[0]) || col.contains(&other.col[1])))
            || (row.contains(&other.row[0])
                && (col.contains(&other.col[0]) || col.contains(&other.col[1])))
    }

    pub fn extract_code(&self, code: &str) -> String {
        let lines = code
            .lines()
            .skip(self.row[0])
            .take(self.row[1] - self.row[0] + 1)
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
            row: [0, 0],
            col: [0, 10],
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row: [0, 0],
            col: [8, 15],
        };
        assert!(pos_1.intersects(&pos_2));
        assert!(pos_2.intersects(&pos_1));
    }

    #[test]
    fn test_pos_no_intersection() {
        let pos_1 = StatementPos {
            execution_type: ExecutionType::Exec,
            row: [0, 0],
            col: [10, 10],
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row: [11, 13],
            col: [15, 15],
        };
        assert!(!pos_1.intersects(&pos_2));
        assert!(!pos_2.intersects(&pos_1));
    }

    #[test]
    fn test_pos_same_pos() {
        let pos_1 = StatementPos {
            execution_type: ExecutionType::Exec,
            row: [0, 0],
            col: [10, 10],
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row: [0, 0],
            col: [10, 10],
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
            row: [0, 10],
            col: [0, 0],
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row: [12, 15],
            col: [0, 0],
        };
        assert!(!pos_1.intersects(&pos_2));
        assert!(!pos_2.intersects(&pos_1));
    }

    #[test]
    fn test_pos_no_intersection_same_row() {
        // 1........1.2...2
        let pos_1 = StatementPos {
            execution_type: ExecutionType::Exec,
            row: [0, 0],
            col: [0, 10],
        };
        let pos_2 = StatementPos {
            execution_type: ExecutionType::Exec,
            row: [0, 0],
            col: [12, 15],
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
            row: [1, 2],
            col: [0, 5],
        };

        assert_eq!(pos.extract_code(code), "b = 2\nc = 3".to_string());
    }
}
