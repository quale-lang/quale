//! Lexer for qcc
use crate::ast::Token;
use std::fmt;

pub(crate) struct Location {
    path: String,
    row: usize,
    col: usize,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let basename = *self.path.split('/').collect::<Vec<_>>().last().unwrap();
        write!(f, "@{}:{}:{}", basename, self.row, self.col)
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Location")
            .field("path", &self.path)
            .field("row", &self.row)
            .field("col", &self.col)
            .finish()
    }
}

/// It maintains a pointer to the source buffer, one pointing to the current
/// byte, and a const pointing to end.
#[derive(Debug)]
pub(crate) struct Lexer<'a> {
    // stores the entire buffer this is never over-written, only read
    buffer: &'a Vec<u8>,
    // stores the next index in buffer, from where to resume reading buffer
    start: usize,
    end: usize,
    prev: usize,
    current: usize,
    location: Location,
    last_token: Option<Token>, // stores last token
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(buffer: &'a Vec<u8>, path: &'a String) -> Self {
        Self {
            buffer,
            start: 0,
            end: 0,
            prev: 0,
            current: 0,
            location: Location {
                path: path.to_string(),
                row: 0,
                col: 0,
            },
            last_token: None,
        }
    }

    /// Returns current identifier based on `self.prev` and `self.current`.
    pub(crate) fn identifier(&self) -> String {
        let mut identifier: String = Default::default();
        for byte in &self.buffer[self.prev..self.current - 1] {
            identifier.push(*byte as char);
        }
        return identifier;
    }

    /// Utility function to dump vector of bytes in string format.
    pub(crate) fn dump(&self) {
        print!("> ");
        for byte in &self.buffer[self.prev..self.current] {
            print!("{}", std::ascii::escape_default(*byte));
        }
        println!();
    }

    /// Returns the next token wrapped. If EOF is reached it returns None.
    /// In order to find next token, we start looking first in `self.line`, if
    /// it is empty then we need next line. Note, `next_line` trims the newline
    /// character at end, so we must keep calling `next_line` until a non-empty
    /// `self.line` is returned.
    pub(crate) fn next_token(&mut self) -> Option<Token> {
        while self.current == self.end
            || self.buffer[self.start..].starts_with(&[0x2f, 0x2f])
            || self.buffer[self.start..self.end] == [0xa]
        {
            self.next_line()?;
        }

        // Skip all whitespaces
        while self.buffer[self.current] == 0x20 {
            self.current += 1;
            self.location.col += 1;
        }

        /* print!("{}\t", self.location); */
        self.prev = self.current;
        while !self.buffer[self.current].is_ascii_whitespace() {
            self.current += 1;
        }
        self.current += 1; // skip whitespace

        if self.identifier() == "fn" {
            println!("Found a function at {}", self.location);
        }
        /* self.dump(); */

        self.last_token = Some(Token::Identifier);
        self.location.col += self.current - self.prev;
        Some(Token::Identifier)
    }

    /// Consumes last set token and moves onto the next token in buffer.
    pub(crate) fn consume(&mut self, token: Token) {
        if let Some(last_token) = &self.last_token {
            // Failure on consuming EOF (None). Should it be?
            assert_eq!(token, *last_token);
            self.prev = self.current;
        }
    }

    /// Reads the next line updating `self.line_start` and `self.line_end`.
    fn next_line(&mut self) -> Option<()> {
        if self.buffer[self.end..].is_empty() {
            return None;
        }

        self.start = self.end;
        self.prev = self.start;
        self.current = self.prev;

        while self.buffer[self.end] != /* newline */0xa {
            self.end += 1;
        }
        self.end += 1;

        self.location.row += 1;
        self.location.col = 1;

        Some(())
    }
}
