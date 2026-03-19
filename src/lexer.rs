use crate::error::{CalcError, Result};

/// Every terminal symbol in the calculator grammar.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Numeric literal — already converted to `f64`.
    Number(f64),
    /// Identifier: variable name or function name.
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    /// `^` or `**` — exponentiation.
    Caret,
    LParen,
    RParen,
    Comma,
    Eq,
    Eof,
}

/// Converts raw source text into a flat [`Token`] list terminated by [`Token::Eof`].
pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for `input`.
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            src: input.as_bytes(),
            pos: 0,
        }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    #[inline]
    fn peek2(&self) -> Option<u8> {
        self.src.get(self.pos + 1).copied()
    }

    #[inline]
    fn advance(&mut self) -> Option<u8> {
        let c = self.src.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }

    fn lex_number(&mut self) -> Result<Token> {
        let start = self.pos;

        if self.peek() == Some(b'0') {
            match self.peek2() {
                Some(b'x' | b'X') => {
                    self.pos += 2;
                    return self.lex_radix(start, 16);
                }
                Some(b'b' | b'B') => {
                    self.pos += 2;
                    return self.lex_radix(start, 2);
                }
                _ => {}
            }
        }

        while self.peek().is_some_and(|b| b.is_ascii_digit() || b == b'_') {
            self.pos += 1;
        }

        if self.peek() == Some(b'.') && self.peek2().is_some_and(|b| b.is_ascii_digit()) {
            self.pos += 1;
            while self.peek().is_some_and(|b| b.is_ascii_digit() || b == b'_') {
                self.pos += 1;
            }
        }

        if self.peek().is_some_and(|b| b == b'e' || b == b'E') {
            self.pos += 1;
            if self.peek().is_some_and(|b| b == b'+' || b == b'-') {
                self.pos += 1;
            }
            let exp_start = self.pos;
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.pos += 1;
            }
            if self.pos == exp_start {
                return Err(CalcError::Lex {
                    pos: start,
                    msg: "empty exponent in scientific notation".into(),
                });
            }
        }

        let raw: String = self.src[start..self.pos]
            .iter()
            .filter(|&&b| b != b'_')
            .map(|&b| b as char)
            .collect();

        raw.parse::<f64>()
            .map(Token::Number)
            .map_err(|_| CalcError::Lex {
                pos: start,
                msg: format!("malformed number '{raw}'"),
            })
    }

    /// Lex a non-decimal integer literal; the prefix bytes have already been
    /// consumed.  `radix` must be `2` or `16`.
    fn lex_radix(&mut self, start: usize, radix: u32) -> Result<Token> {
        let digit_start = self.pos;

        while self.peek().is_some_and(|b| {
            b == b'_'
                || (radix == 16 && b.is_ascii_hexdigit())
                || (radix != 16 && matches!(b, b'0' | b'1'))
        }) {
            self.pos += 1;
        }

        let raw: String = self.src[digit_start..self.pos]
            .iter()
            .filter(|&&b| b != b'_')
            .map(|&b| b as char)
            .collect();

        if raw.is_empty() {
            return Err(CalcError::Lex {
                pos: start,
                msg: format!("empty base-{radix} literal"),
            });
        }

        u64::from_str_radix(&raw, radix)
            .map(|n| {
                #[allow(clippy::cast_precision_loss)]
                Token::Number(n as f64)
            })
            .map_err(|_| CalcError::Lex {
                pos: start,
                msg: format!("invalid base-{radix} literal '{raw}'"),
            })
    }

    /// Tokenise the entire input in one pass.
    ///
    /// # Errors
    ///
    /// Returns [`CalcError::Lex`] on an unrecognised character or malformed
    /// numeric literal.
    pub fn tokenize(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::with_capacity(self.src.len() / 2 + 1);

        loop {
            self.skip_whitespace();

            let Some(b) = self.peek() else {
                tokens.push(Token::Eof);
                break;
            };

            let tok = match b {
                b'0'..=b'9' => self.lex_number()?,

                b'.' if self.peek2().is_some_and(|d| d.is_ascii_digit()) => self.lex_number()?,

                b if b.is_ascii_alphabetic() || b == b'_' => {
                    let start = self.pos;
                    while self
                        .peek()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
                    {
                        self.pos += 1;
                    }
                    let s = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("?");
                    Token::Ident(s.to_owned())
                }

                // Multi-byte UTF-8 identifiers: π, φ, √, etc.
                b if !b.is_ascii() => {
                    let tail =
                        std::str::from_utf8(&self.src[self.pos..]).map_err(|_| CalcError::Lex {
                            pos: self.pos,
                            msg: "invalid UTF-8 sequence".into(),
                        })?;
                    let ch = tail.chars().next().ok_or(CalcError::Lex {
                        pos: self.pos,
                        msg: "unexpected end of input".into(),
                    })?;
                    if ch.is_alphabetic() || ch == '√' {
                        let start = self.pos;
                        self.pos += ch.len_utf8();
                        loop {
                            let rest = std::str::from_utf8(&self.src[self.pos..]).unwrap_or("");
                            match rest.chars().next() {
                                Some(c) if c.is_alphanumeric() || c == '_' => {
                                    self.pos += c.len_utf8();
                                }
                                _ => break,
                            }
                        }
                        let s = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("?");
                        Token::Ident(s.to_owned())
                    } else {
                        return Err(CalcError::Lex {
                            pos: self.pos,
                            msg: format!("unexpected character '{ch}'"),
                        });
                    }
                }

                b'+' => {
                    self.advance();
                    Token::Plus
                }
                b'-' => {
                    self.advance();
                    Token::Minus
                }
                b'*' => {
                    self.advance();
                    if self.peek() == Some(b'*') {
                        self.advance();
                        Token::Caret
                    } else {
                        Token::Star
                    }
                }
                b'/' => {
                    self.advance();
                    Token::Slash
                }
                b'%' => {
                    self.advance();
                    Token::Percent
                }
                b'^' => {
                    self.advance();
                    Token::Caret
                }
                b'(' => {
                    self.advance();
                    Token::LParen
                }
                b')' => {
                    self.advance();
                    Token::RParen
                }
                b',' => {
                    self.advance();
                    Token::Comma
                }
                b'=' => {
                    self.advance();
                    Token::Eq
                }

                other => {
                    return Err(CalcError::Lex {
                        pos: self.pos,
                        msg: format!("unexpected character '{}'", other as char),
                    });
                }
            };

            tokens.push(tok);
        }

        Ok(tokens)
    }
}
