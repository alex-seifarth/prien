/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use super::Decoder;
use std::fmt::{Display, Formatter};

/// Position within a text file.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, column: {}", self.line, self.column)
    }
}

/// A UTF-8 encoded stream of characters readable in forward manner with peek (look-ahead) function.
/// The struct implements next to the 'get()' method to retrieve and consume the next character also
/// the 'Iterator' trait with its 'next()' method. The difference lies in the returned value, the
/// 'get()' method is a little bit more suitable for our later purposes in the lexer.
///
/// #TODO
/// - resynchronization after UTF-8 encoding failures needs to be implemented
pub struct Stream {
    data: Vec<u8>,
    index: usize,
    dec: Decoder,
    pos: Position,
    peeked: Option< Result< Option<char>, () > >,
    error: bool,
}

impl Stream {

    pub fn create(data: Vec<u8>) -> Stream {
        Stream{ data, index: 0, dec: Decoder::new(), pos: Position{ line: 1, column: 0}, peeked: None, error: false }
    }

    /// Returns the current position of the stream.
    /// #Notes
    /// In case of an error in 'get()' the position is the position of the last successfully decoded
    /// character.<p>
    /// After a new line or before the first character is being decoded the column of the position
    /// is 0. This 0 has the meaning of being 'before the first character of the current line'.
    pub fn pos(&self) -> Position {
        self.pos
    }

    /// Returns the next character from the UTF-8 stream data.
    /// # Returns
    /// - Ok( Some( ch ) )      A valid UTF-8 character has been detected, file position had been updated.
    /// - Ok( None )            The file is at its end.
    /// - Err(())               An UTF-8 encoding error occurred.
    /// # Notes
    /// Calling this function while the instance has encountered an error before will panic. It is
    /// necessary to resynchronize it before another call to 'get' can be made.
    pub fn get(&mut self) -> Result< Option<char>, () > {
        if self.error {
            panic!("Instance is in error condition, cannot proceed without resyncing.");
        }
        let val = match self.peeked {
            Some(_) => self.peeked.take().unwrap(),
            None => self.get_next_char(),
        };

        return match val {
            Ok( Some( ch )) => {
                self.advance_position(ch);
                Ok( Some( ch ))
            },
            Ok( None ) => Ok( None ),
            Err(_) => {
                self.error = true;
                Err(())
            }
        }
    }

    /// Consume the next character without returning it.
    /// This method is usually used in conjunction with peek and will panic if the next character
    /// is an error!
    pub fn advance(&mut self) {
        let _ = self.get().unwrap();
    }

    /// Returns the next character without advancing the current read position.
    /// Calling 'peek()' without interleaving calls to 'get()' or 'advance()' will always return
    /// the same result. <p>
    /// A call to 'get()' after a call to 'peek()' will return the same value.<p>
    /// When peek returns an UTF-8 encoding error the stream is NOT in an error condition yet, so
    /// 'get()' maybe called safely, but 'advance()' will panic.
    pub fn peek(&mut self) -> Result< Option<char>, () > {
        if self.peeked.is_none() {
            self.peeked = Some( self.get_next_char() );
        }
        return self.peeked.as_ref().unwrap().clone();
    }

    fn get_next_char(&mut self) -> Result< Option<char>, () > {
        if self.index >= self.data.len() {
            return Ok( None )
        }

        loop {
            let r = self.dec.decode(self.data[self.index]);
            self.index += 1;
            match r {
                Ok( None ) => {
                    if self.index >= self.data.len() {
                        return Err(())
                    }
                },
                Ok( Some( ch )) => return Ok( Some( ch )),
                Err(()) => return Err(()),
            }
        }
    }

    fn advance_position(&mut self, ch: char) {
        match ch {
            '\n' | '\u{0085}' | '\u{2028}' | '\u{2029}' => {
                self.pos.line += 1;
                self.pos.column = 0;
            },
            _ => {
                self.pos.column += 1;
            }
        }
    }

}

impl Iterator for Stream {
    type Item = Result<char, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.get() {
            Ok( Some( ch )) => Some( Ok( ch )),
            Ok( None ) => None,
            Err(()) => Some( Err(()) )
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Stream, Position};

    #[test]
    fn valid_text() {
        let txt = "This is a text. It will be encoded\n as UTF8! Hopefully \u{00f9}";
        let mut utxt = Stream::create(txt.to_string().into_bytes());

        let mut txt_it = txt.chars();
        loop {
            let r = utxt.get().unwrap();
            if r.is_none() {
                assert!(txt_it.next().is_none());
                break;
            }
            assert_eq!(txt_it.next().unwrap(), r.unwrap());
        }
    }

    #[test]
    fn valid_text_iterator() {
        let txt = "This is a text. It will be encoded\n as UTF8! Hopefully \u{00f9}";
        let utxt = Stream::create(txt.to_string().into_bytes());
        let mut txt_it = txt.chars();
        for t in utxt {
            assert!(t.is_ok());
            assert_eq!(t.unwrap(), txt_it.next().unwrap());
        }
    }

    #[test]
    fn position() {
        let txt = "L1 \nL2\n";
        let mut utxt = Stream::create(txt.to_string().into_bytes());

        assert_eq!(utxt.get().unwrap(), Some('L'));
        assert_eq!(utxt.pos(), Position{ line: 1, column: 1});
        assert_eq!(utxt.get().unwrap(), Some('1'));
        assert_eq!(utxt.pos(), Position{ line: 1, column: 2});
        assert_eq!(utxt.get().unwrap(), Some(' '));
        assert_eq!(utxt.pos(), Position{ line: 1, column: 3});
        assert_eq!(utxt.get().unwrap(), Some('\n'));
        assert_eq!(utxt.pos(), Position{ line: 2, column: 0});

        assert_eq!(utxt.get().unwrap(), Some('L'));
        assert_eq!(utxt.pos(), Position{ line: 2, column: 1});
        assert_eq!(utxt.get().unwrap(), Some('2'));
        assert_eq!(utxt.pos(), Position{ line: 2, column: 2});
        assert_eq!(utxt.get().unwrap(), Some('\n'));
        assert_eq!(utxt.pos(), Position{ line: 3, column: 0});

        assert_eq!(utxt.get().unwrap(), None);
    }

    #[test]
    fn valid_peek() {
        let txt = "a!";
        let mut utxt = Stream::create(txt.to_string().into_bytes());

        assert_eq!(utxt.peek().unwrap(), Some('a'));
        assert_eq!(utxt.pos(), Position{ line: 1, column: 0});
        assert_eq!(utxt.get().unwrap(), Some('a'));
        assert_eq!(utxt.pos(), Position{ line: 1, column: 1});

        assert_eq!(utxt.peek().unwrap(), Some('!'));
        assert_eq!(utxt.pos(), Position{ line: 1, column: 1});
        assert_eq!(utxt.peek().unwrap(), Some('!'));
        assert_eq!(utxt.pos(), Position{ line: 1, column: 1});
        assert_eq!(utxt.get().unwrap(), Some('!'));
        assert_eq!(utxt.pos(), Position{ line: 1, column: 2});

        assert_eq!(utxt.peek().unwrap(), None);
        assert_eq!(utxt.pos(), Position{ line: 1, column: 2});
        assert_eq!(utxt.get().unwrap(), None);
    }
}
