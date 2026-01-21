use crate::line_map::Lines; // Assuming Lines is in your lib.rs or main.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

pub struct Cursor<'a> {
    lines: &'a Lines,
    line_idx: usize,
    col_idx: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(lines: &'a Lines) -> Self {
        Self {
            lines,
            line_idx: 0,
            col_idx: 0,
        }
    }

    /// Returns the current character without advancing.
    /// Returns None if we are at the end of the file.
    pub fn peek(&self) -> Option<char> {
        self.lines.get(self.line_idx).and_then(|line| {
            // Check if we are within the bounds of the current line string
            if self.col_idx < line.content.len() {
                line.content.chars().nth(self.col_idx)
            } else {
                // If we are at the end of the line, we treat the newline as a logical '\n'
                // This simplifies the Lexer logic so it doesn't have to manually check line boundaries.
                // However, we must ensure we don't loop forever on the last line.
                if self.line_idx < self.lines.len() {
                    Some('\n')
                } else {
                    None
                }
            }
        })
    }

    /// Advances to the next character.
    /// Returns the character we just passed over.
    pub fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;

        if ch == '\n' {
            self.line_idx += 1;
            self.col_idx = 0;
        } else {
            self.col_idx += 1;
        }

        Some(ch)
    }

    /// Returns the current location for error reporting
    pub fn loc(&self) -> Location {
        // Map the internal 0-based index to the actual file line number defined in Line struct
        let actual_line_num = self
            .lines
            .get(self.line_idx)
            .map(|l| l.idx)
            .unwrap_or_else(|| {
                if self.line_idx > 0 {
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

    /// Helper: Advance while a condition is true (useful for whitespace/comments)
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

use crate::ast1::Span; // Assuming AST1 is in a module named ast1
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // --- Literals ---
    // We store the raw string here; parsing "10u" into value+suffix
    // happens in the conversion to AST or during parsing.
    Ident(String),  // "speed", "ground_type"
    Int(String),    // "10", "0b1010", "5u"
    Float(String),  // "10.0", "0.5f"
    String(String), // "On ground\n" (content only)

    // --- Keywords ---
    Let,    // "let"
    If,     // "if"
    Elif,   // "elif"
    Else,   // "else"
    Yield,  // "yield"
    Return, // "return"
    True,   // "true"
    False,  // "false"

    // --- Symbols ---
    Eq,     // "="
    EqEq,   // "=="
    Bang,   // "!"
    BangEq, // "!="
    Plus,   // "+"
    Minus,  // "-"
    Star,   // "*"
    Slash,  // "/"

    Lt,  // "<"
    Le,  // "<="
    Gt,  // ">"
    Ge,  // ">="
    And, // "&&"
    Or,  // "||"

    LParen, // "("
    RParen, // ")"
    LBrace, // "{"
    RBrace, // "}"
    Comma,  // ","
    Colon,  // ":"
    Semi,   // ";"

    // --- Special ---
    Eof,           // End of file
    Error(String), // Lexer error (e.g., "Unknown char")
}

// Helper to make debugging/error messages nicer
impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Ident(s) => write!(f, "Ident({})", s),
            TokenKind::Int(s) => write!(f, "Int({})", s),
            TokenKind::Float(s) => write!(f, "Float({})", s),
            TokenKind::String(s) => write!(f, "String(\"{}\")", s),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::Eof => write!(f, "<EOF>"),
            // ... (others can be inferred or added as needed)
            _ => write!(f, "{:?}", self),
        }
    }
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(cursor: Cursor<'a>) -> Self {
        Self { cursor }
    }

    /// The main entry point to get the next token.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        // Start span recording
        let start_loc = self.cursor.loc();

        // Peek at the next character to decide what to do
        let ch = match self.cursor.peek() {
            Some(c) => c,
            None => {
                return Token {
                    kind: TokenKind::Eof,
                    span: Span {
                        line: start_loc.line,
                        col: start_loc.col,
                    },
                };
            }
        };

        // 1. Identifiers and Keywords (start with a-z or _)
        if is_ident_start(ch) {
            let raw = self.cursor.eat_while(is_ident_char);
            let kind = match raw.as_str() {
                "let" => TokenKind::Let,
                "if" => TokenKind::If,
                "elif" => TokenKind::Elif,
                "else" => TokenKind::Else,
                "yield" => TokenKind::Yield,
                "return" => TokenKind::Return,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                _ => TokenKind::Ident(raw),
            };
            return self.make_token(kind, start_loc);
        }

        // 2. Numbers (start with 0-9)
        if ch.is_ascii_digit() {
            return self.scan_number(start_loc);
        }

        // 3. String Literals (start with ")
        if ch == '"' {
            return self.scan_string(start_loc);
        }

        // 4. Symbols and Operators
        // We advance the cursor here because we know we are consuming at least one char.
        self.cursor.advance();

        let kind = match ch {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semi,
            ':' => TokenKind::Colon,

            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => {
                // Check for comments "//"
                if self.cursor.peek() == Some('/') {
                    // Consume until end of line
                    self.cursor.eat_while(|c| c != '\n');
                    // Recursively call next_token to skip the comment entirely
                    return self.next_token();
                } else {
                    TokenKind::Slash
                }
            }

            '=' => {
                if self.match_char('=') {
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if self.match_char('=') {
                    TokenKind::BangEq
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if self.match_char('=') {
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.match_char('=') {
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                if self.match_char('&') {
                    TokenKind::And
                } else {
                    TokenKind::Error(format!("Expected '&&', found '&'"))
                }
            }
            _ => TokenKind::Error(format!("Unexpected character: '{}'", ch)),
        };

        self.make_token(kind, start_loc)
    }

    // --- Helpers ---

    fn make_token(&self, kind: TokenKind, start_loc: Location) -> Token {
        // We calculate the end column based on where the cursor is NOW (after consuming).
        // Note: For a real span, you might want separate start/end types,
        // but for now we just track where it started.
        Token {
            kind,
            span: Span {
                line: start_loc.line,
                col: start_loc.col,
            },
        }
    }

    fn skip_whitespace(&mut self) {
        self.cursor.eat_while(|c| c.is_whitespace());
    }

    /// Check if the next char matches expected. If so, consume it.
    fn match_char(&mut self, expected: char) -> bool {
        if self.cursor.peek() == Some(expected) {
            self.cursor.advance();
            true
        } else {
            false
        }
    }

    fn scan_string(&mut self, start_loc: Location) -> Token {
        self.cursor.advance(); // Skip opening quote

        let mut value = String::new();
        while let Some(ch) = self.cursor.peek() {
            if ch == '"' {
                break;
            }
            if ch == '\n' {
                // Optional: Error on multiline strings if not supported
            }
            value.push(self.cursor.advance().unwrap());
        }

        // Consume closing quote
        if self.cursor.peek() == Some('"') {
            self.cursor.advance();
            self.make_token(TokenKind::String(value), start_loc)
        } else {
            self.make_token(TokenKind::Error("Unterminated string".into()), start_loc)
        }
    }

    fn scan_number(&mut self, start_loc: Location) -> Token {
        let mut raw = String::new();
        let mut is_float = false;

        // 1. Consume integer part (or hex/bin prefix logic could go here)
        raw.push_str(&self.cursor.eat_while(|c| c.is_ascii_digit() || c == '_'));

        // 2. Check for fractional part
        if self.cursor.peek() == Some('.') {
            // Lookahead to ensure it's not a method call or range (e.g. 1..10)
            // But since your language uses standard floats, we assume `.` followed by digit is float.
            // For safety, let's just peek one more.
            // (Simpler logic: just consume dot)
            is_float = true;
            raw.push(self.cursor.advance().unwrap()); // consume '.'
            raw.push_str(&self.cursor.eat_while(|c| c.is_ascii_digit() || c == '_'));
        }

        // 3. Check for Exponent (Scientific notation) '1e10'
        if let Some(ch) = self.cursor.peek() {
            if ch == 'e' || ch == 'E' {
                is_float = true;
                raw.push(self.cursor.advance().unwrap()); // consume 'e'
                if let Some(sign) = self.cursor.peek() {
                    if sign == '+' || sign == '-' {
                        raw.push(self.cursor.advance().unwrap());
                    }
                }
                raw.push_str(&self.cursor.eat_while(|c| c.is_ascii_digit()));
            }
        }

        // 4. Check for Suffixes (f, u, i)
        // This is where we decide the final token kind based on your AST rules.
        if let Some(ch) = self.cursor.peek() {
            if ch == 'f' {
                is_float = true; // '10f' is a float
                raw.push(self.cursor.advance().unwrap());
            } else if ch == 'u' || ch == 'i' {
                // '10u' is an int. '10.5u' is technically invalid in many langs,
                // but we will let the parser reject it or just tokenize as Float+suffix?
                // Your AST says IntLit handles 'u'/'i'. FloatLit handles 'f'.
                // If we already marked it as is_float (due to dot), and we see 'u',
                // we'll include it in raw, but return Float token (Parser will error "invalid suffix for float").
                raw.push(self.cursor.advance().unwrap());
            }
        }

        if is_float {
            self.make_token(TokenKind::Float(raw), start_loc)
        } else {
            self.make_token(TokenKind::Int(raw), start_loc)
        }
    }
}

// --- Char Predicates ---

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
