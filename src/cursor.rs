use crate::line_map::Line;
use std::str::Chars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

pub struct Cursor<'a> {
    lines: &'a [Line],
    line_idx: usize,
    col_idx: usize,
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    // Update new to accept anything that can be viewed as a slice of Lines
    pub fn new(lines: &'a [Line]) -> Self {
        // Initialize the char iterator from the first line's content if present,
        // otherwise use an empty iterator from a static empty str.
        let start = lines.get(0).map(|l| l.content.as_str()).unwrap_or("");
        Self {
            lines,
            line_idx: 0,
            col_idx: 0,
            chars: start.chars(),
        }
    }

    /// Returns the current character without advancing.
    pub fn peek(&self) -> Option<char> {
        // Optimization: Cloning the iterator is cheap and lets us peek O(1)
        let ch = self.chars.clone().next();

        match ch {
            Some(c) => Some(c),
            None => {
                // If iterator is empty, we are at the end of the line.
                // We virtually inject a newline if there are more lines remaining.
                if self.line_idx < self.lines.len() {
                    Some('\n')
                } else {
                    None
                }
            }
        }
    }

    /// Advances to the next character.
    pub fn advance(&mut self) -> Option<char> {
        match self.chars.next() {
            Some(c) => {
                self.col_idx += 1;
                Some(c)
            }
            None => {
                // End of current line content; check if we can move to next line
                if self.line_idx < self.lines.len() {
                    // Move to next line
                    self.line_idx += 1;
                    self.col_idx = 0;

                    // Load the next line's iterator
                    self.chars = self
                        .lines
                        .get(self.line_idx)
                        .map(|l| l.content.as_str())
                        .unwrap_or("")
                        .chars();

                    // Return the virtual newline we just passed over
                    Some('\n')
                } else {
                    None
                }
            }
        }
    }

    /// Returns the current location for error reporting
    pub fn loc(&self) -> Location {
        let actual_line_num = self
            .lines
            .get(self.line_idx)
            .map(|l| l.idx)
            .unwrap_or_else(|| {
                if self.line_idx > 0 && !self.lines.is_empty() {
                    self.lines[self.lines.len() - 1].idx + 1
                } else {
                    0
                }
            });

        Location {
            line: actual_line_num,
            col: self.col_idx,
        }
    }

    pub fn eat_while<F>(&mut self, mut predicate: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if predicate(ch) {
                result.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        result
    }
}
