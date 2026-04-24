use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub byte_range: Range<usize>,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

impl Span {
    pub fn from_byte_range(source: &str, byte_range: Range<usize>) -> Self {
        let (start_line, start_col) = line_col(source, byte_range.start);
        let (end_line, end_col) = line_col(source, byte_range.end);
        Self {
            byte_range,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }
}

fn line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line: u32 = 1;
    let mut col: u32 = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_covers_byte_range_and_line_col() {
        let s = "hello\nworld\n";
        let sp = Span::from_byte_range(s, 6..11);
        assert_eq!(sp.start_line, 2);
        assert_eq!(sp.start_col, 1);
        assert_eq!(sp.end_line, 2);
        assert_eq!(sp.end_col, 6);
        assert_eq!(sp.byte_range, 6..11);
    }
}
