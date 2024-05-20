//! Lexer for qcc
use crate::ast::Token;
use crate::error::{QccErrorKind, Result};

use std::fmt;

#[derive(Clone, PartialEq)]
pub(crate) struct Location {
    path: String, // TODO: immutable std::rc::Rc<>
    row: usize,
    col: usize,
}

impl Location {
    pub(crate) fn new(path: &str, row: usize, col: usize) -> Self {
        Self {
            path: path.into(),
            row,
            col,
        }
    }

    pub(crate) fn path(&self) -> String {
        self.path.clone()
    }

    #[inline]
    pub(crate) fn row(&self) -> usize {
        self.row
    }

    #[inline]
    pub(crate) fn col(&self) -> usize {
        self.col
    }
}

impl Default for Location {
    fn default() -> Self {
        Self {
            path: "unknown".into(),
            row: 0,
            col: 0,
        }
    }
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

/// `Pointer` is a movable reference into a buffer.
#[derive(Debug)]
pub(crate) struct Pointer {
    /// start of the line, as seen in source, points first non-whitespace char
    start: usize,
    /// index at previous recognized token
    prev: usize,
    /// index to resume reading from
    current: usize,
    /// end of line
    end: usize,
}

impl Pointer {
    /// Create a new `Pointer`.
    pub(crate) fn new() -> Self {
        Self {
            start: 0,
            prev: 0,
            current: 0,
            end: 0,
        }
    }

    /// Align all indices to `Pointer::start`
    pub(crate) fn backward(&self) -> Self {
        Self {
            start: self.start,
            prev: self.start,
            current: self.start,
            end: self.start,
        }
    }

    /// Move all indices forward to `Pointer::end`.
    pub(crate) fn forward(&self) -> Self {
        Self {
            start: self.end,
            prev: self.end,
            current: self.end,
            end: self.end,
        }
    }

    /// Move `Pointer::prev` forward to `Pointer::current`.
    pub(crate) fn reset(&self) -> Self {
        Self {
            start: self.start,
            prev: self.current,
            current: self.current,
            end: self.end,
        }
    }

    /// Get the entire range start `Pointer::start` to `Pointer::end`.
    pub(crate) fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

/// It maintains a pointer to the source buffer, one pointing to the current
/// byte, and a const pointing to end.
#[derive(Debug)]
pub(crate) struct Lexer {
    /// stores the entire buffer this is never over-written, only read
    buffer: Vec<u8>,
    /// stores tracking indices pointing to buffer
    ptr: Pointer,
    /// location of current token
    pub(crate) location: Location,
    /// stores current token
    pub(crate) token: Option<Token>,
}

impl Lexer {
    pub(crate) fn new(buffer: Vec<u8>, path: String) -> Self {
        Self {
            buffer,
            ptr: Pointer::new(),
            location: Location {
                path: path.to_string(),
                row: 0,
                col: 0,
            },
            token: None,
        }
    }

    /// Returns a string in `buffer` given valid indices. This is prone to panic
    /// if out of range indices are provided.
    pub(crate) fn slice(&self, lhs: usize, rhs: usize) -> String {
        let mut sliced: String = Default::default();
        for byte in &self.buffer[lhs..rhs] {
            sliced.push(*byte as char);
        }
        sliced
    }

    /// Returns the previous pointing character in buffer.
    fn previous(&self) -> u8 {
        self.buffer[self.ptr.prev]
    }

    /// Returns the current pointing character in buffer.
    fn current(&self) -> u8 {
        self.buffer[self.ptr.current]
    }

    /// Returns the digit as a string after trimming whitespaces.
    pub(crate) fn digit(&self) -> Option<f64> {
        let digit = self.identifier().replace(" ", "").parse::<f64>();
        if digit.is_err() {
            return None;
        }
        Some(digit.unwrap())
    }

    /// Returns current identifier contained in `self.prev` and `self.current`.
    pub(crate) fn identifier(&self) -> String {
        self.slice(self.ptr.prev, self.ptr.current)
    }

    /// Utility function to dump vector of bytes in string format.
    pub(crate) fn dump(&self) {
        print!("> ");
        for byte in &self.buffer[self.ptr.prev..self.ptr.current] {
            print!("{}", std::ascii::escape_default(*byte));
        }
        println!();
    }

    /// Dumps a range of characters from buffer. It may panic if out of bounds
    /// indices are provided.
    pub(crate) fn dump_range(&self, start: usize, end: usize) {
        print!("> ");
        for byte in &self.buffer[start..end] {
            print!("{}", std::ascii::escape_default(*byte));
        }
        println!();
    }

    /// Returns the entire line which is being parsed at the moment.
    pub(crate) fn line(&self) -> String {
        self.slice(self.ptr.start, self.ptr.end)
    }

    #[inline]
    /// Checks if the current token is what is given.
    pub(crate) fn is_token(&self, token: Token) -> bool {
        !self.token.is_none() && self.token == Some(token)
    }

    /// Checks if the current token is any of the given slice of tokens.
    pub(crate) fn is_any_token(&self, tokens: &[Token]) -> bool {
        !self.token.is_none()
            && (|lexer: &Self, tokens| -> bool {
                let mut result = false;
                for &token in tokens {
                    result |= self.is_token(token);
                }
                result
            })(self, tokens)
    }

    /// Checks if current token is not in the given slice of tokens.
    pub(crate) fn is_none_token(&self, tokens: &[Token]) -> bool {
        !self.token.is_none()
            && (|lexer: &Self, tokens| -> bool {
                let mut result = true;
                for &token in tokens {
                    result &= !self.is_token(token);
                }
                result
            })(self, tokens)
    }

    /// Returns the next token wrapped. If EOF is reached it returns None.
    /// In order to find next token, we start looking first in `self.line`, if
    /// it is empty then we need next line. Note, `next_line` trims the newline
    /// character at end, so we must keep calling `next_line` until a non-empty
    /// `self.line` is returned.
    pub(crate) fn next_token(&mut self) -> Result<Option<Token>> {
        // Skip all leading whitespaces and trailing newlines.
        while self.buffer[self.ptr.current].is_ascii_whitespace() {
            self.ptr.current += 1;
            self.location.col += 1;

            // If only whitespaces are present, ask for next line.
            if self.ptr.current >= self.ptr.end {
                if self.next_line() == None {
                    self.token = None;
                    return Ok(self.token);
                }

                if self.ptr.prev == self.ptr.end {
                    self.token = None;
                    return Ok(self.token);
                }

                return self.next_token();
            }
        }

        while self.ptr.current >= self.ptr.end
            || self.buffer[self.ptr.start..].starts_with(&['/' as u8, '/' as u8])
            || self.buffer[self.ptr.range()] == ['\n' as u8]
            || self.buffer[self.ptr.current..].starts_with(&['/' as u8, '/' as u8])
        {
            // TODO: FromResidual trait impl (but nightly) to use ?
            // TODO: == None blob should be rechecked because bug was present
            // because of no return of self.next_token after a new line was
            // fetched.
            if self.next_line() == None {
                self.token = None;
                return Ok(self.token);
            }

            // If there is no EOF then only fetch next line as long as
            // everything is already lexemed.
            if self.ptr.prev == self.ptr.end {
                self.token = None;
                return Ok(self.token);
            }

            // FIXME: stackoverflow, too much recursion, see bug
            // only-whitespace-no-eof.ql
            return self.next_token();
        }

        self.ptr = self.ptr.reset();

        let single_token = match self.current().into() {
            '#' => Token::Hash,
            '[' => Token::OBracket,
            ']' => Token::CBracket,
            '{' => Token::OCurly,
            '}' => Token::CCurly,
            '(' => Token::OParenth,
            ')' => Token::CParenth,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            '!' => Token::Bang,
            '=' => Token::Assign,
            '+' => Token::Add,
            '-' => Token::Sub,
            '*' => Token::Mul,
            '/' => Token::Div,
            _ => Token::Multi,
        };

        // if a single character token is found
        if single_token != Token::Multi {
            self.ptr.current += 1;
            self.token = Some(single_token);

            if single_token != Token::Sub {
                return Ok(self.token);
            }
        }

        if single_token == Token::Sub {
            while self.buffer[self.ptr.current].is_ascii_whitespace() {
                self.ptr.current += 1;
                self.location.col += 1;
            }
        }

        if self.current().is_ascii_digit() {
            self.ptr.current += 1;

            // lexing quantum bit
            if self.current() == 'q' as u8 {
                while self.current() != ')' as u8 {
                    self.ptr.current += 1;
                }
                self.ptr.current += 1;
                return Ok(Some(Token::Qbit));
            }

            while self.current().is_ascii_digit() || self.current() == '.' as u8 {
                self.ptr.current += 1;
            }
            self.token = Some(Token::Digit);
            return Ok(self.token);
        }

        // If Sub isn't consumed by now, it was a standalone Sub.
        if single_token == Token::Sub {
            return Ok(self.token);
        }

        if self.current().is_ascii_alphanumeric() || self.current() == '_' as u8 {
            self.ptr.current += 1;
            while self.current().is_ascii_alphanumeric() || self.current() == '_' as u8 {
                self.ptr.current += 1;
            }
            self.token = match self.identifier().as_str() {
                "fn" => Some(Token::Function),
                "return" => Some(Token::Return),
                "const" => Some(Token::Const),
                "extern" => Some(Token::Extern),
                "module" => Some(Token::Module),
                "let" => Some(Token::Let),
                _ => Some(Token::Identifier),
            };
            return Ok(self.token);
        }

        self.ptr.current += 1;
        self.token = Some(Token::Identifier);

        Ok(self.token)
    }

    /// Get the current token.
    pub(crate) fn token(&mut self) -> Result<Option<Token>> {
        // TODO: This shouldn't be used, next_token should be used instead.
        // This gives a wrong meaning to what is happening. next_token is more
        // verbose.
        Ok(self.next_token()?)
    }

    /// Consumes last set token and moves onto the next token in buffer.
    pub(crate) fn consume(&mut self, token: Token) -> Result<()> {
        // TODO: use Lexer::is_token?
        if let Some(last_token) = &self.token {
            assert_eq!(
                token, *last_token,
                "
Internal Compiler Error: Lexer failed {}
Please report this bug to {}",
                self.location, "https://github.com/quale-lang/quale/issues"
            );
            self.location.col += self.ptr.current - self.ptr.prev;
            self.ptr = self.ptr.reset();
            self.token = self.next_token()?;
        }
        Ok(())
    }

    /// Reads the next line updating `self.line_start` and `self.line_end`.
    fn next_line(&mut self) -> Option<()> {
        if self.buffer[self.ptr.end..].is_empty() {
            return None;
        }

        self.ptr = self.ptr.forward();

        while self.buffer[self.ptr.end] != '\n' as u8 {
            if self.ptr.end == self.buffer.len() - 1 {
                self.location.row += 1;

                return Some(());
            }
            // Move Ptr::start to first non-whitespace char.
            if self.buffer[self.ptr.start].is_ascii_whitespace() {
                self.ptr.start += 1;
            }
            self.ptr.end += 1;
        }
        self.ptr.end += 1;

        self.location.row += 1;
        self.location.col = 1;

        Some(())
    }
}
