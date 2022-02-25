use super::Token;
use util::utf8::{Stream, Position};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexerError {
    Unspecified,
    Utf8Error(Position),
    Unexpected(Position, char),
}

pub struct Lexer {
    stream: Stream,
    next: Result<Token, LexerError>
}

impl Lexer {

    pub fn create(data: Vec<u8>) -> Lexer {
        let mut lexer = Lexer { stream: Stream::create(data), next: Err(LexerError::Unspecified) };
        lexer.next = lexer.scan();
        lexer
    }

    /// Returns the next found token or an LexerError without consuming it.
    /// Calling `peek()` several time consecutively or `get()` after `peek()` will always return
    /// the same result again.
    pub fn peek(&self) -> Result<Token, LexerError> {
        self.next.clone()
    }

    /// Returns the next found token or an LexerError and consumes it (e.g. advances in the text).
    pub fn get(&mut self) -> Result<Token, LexerError> {
        let r = self.next.clone();
        self.next = self.scan();
        r
    }

    fn get_char(&mut self) -> Result< Option<char>, LexerError> {
        match self.stream.get() {
            Err(()) => { return Err( LexerError::Utf8Error( self.pos() ) ) },
            Ok(c) => Ok( c ),
        }
    }

    fn pos(&self) -> Position {
        self.stream.pos()
    }

    fn scan(&mut self) -> Result<Token, LexerError> {
        let ch = loop {
            let ch =   match self.get_char()? {
                Some(c) => c,
                None => return Ok( Token::EndOfFile ),
            };
            match ch {
                ' ' | '\n' | '\t' => { continue; },
                _ => break ch,
            }
        };
        self.scan_char(ch)
    }

    fn scan_char(&mut self, ch: char) -> Result<Token, LexerError> {
        match ch {
            '(' => Ok( Token::LeftParen(self.pos())),
            ')' => Ok( Token::RightParen(self.pos())),
            '{' => Ok( Token::LeftBrace(self.pos())),
            '}' => Ok( Token::RightBrace(self.pos())),
            '[' => Ok( Token::LeftBracket(self.pos())),
            ']' => Ok( Token::RightBracket(self.pos())),
            '~' => Ok( Token::Tilde(self.pos())),
            '!' => Ok( Token::ExclamationMark(self.pos())),
            ';' => Ok( Token::Semicolon(self.pos())),
            ',' => Ok( Token::Comma(self.pos())),
            '#' => Ok( Token::Hash(self.pos())),
            '<' => self.scan_less(),
            '>' => self.scan_greater(),
            '=' => self.scan_equals(),
            '+' => self.scan_plus(),
            '-' => self.scan_minus(),
            '*' => self.scan_star(),
            '/' => self.scan_slash(),
            '&' => self.scan_ampersand(),
            '|' => self.scan_vert(),
            '^' => self.scan_caret(),
            '.' => self.scan_dot(),
            ':' => self.scan_colon(),

            _ => Err( LexerError::Unexpected( self.pos(), ch ) )
        }
    }

    fn scan_colon(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok( Some(':') ) => {
                self.stream.advance();
                Ok( Token::ScopeSep(pos) )
            },
            _ => Ok( Token::Colon(pos) )
        }
    }

    fn scan_dot(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok( Some('.') ) => {
                self.stream.advance();
                Ok( Token::Range(pos) )
            },
            _ => Ok( Token::Dot(pos) )
        }
    }

    fn scan_caret(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok( Some('=') ) => {
                self.stream.advance();
                Ok( Token::EXorAssign(pos) )
            },
            _ => Ok( Token::Caret(pos) )
        }
    }

    fn scan_vert(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok( Some('|') ) => {
                self.stream.advance();
                Ok( Token::LogicOr(pos) )
            },
            Ok( Some('=') ) => {
                self.stream.advance();
                Ok( Token::OrAssign(pos) )
            },
            _ => Ok( Token::Vert(pos) )
        }
    }

    fn scan_ampersand(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok(Some('&')) => {
                self.stream.advance();
                Ok(Token::LogicAnd(pos))
            },
            Ok(Some('=')) => {
                self.stream.advance();
                Ok(Token::AndAssign(pos))
            },
            _ => Ok(Token::Ampersand(pos))
        }
    }

    fn scan_slash(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok(Some('=')) => {
                self.stream.advance();
                Ok(Token::DivAssign(pos))
            },
            _ => Ok(Token::Slash(pos))
        }
    }

    fn scan_star(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok(Some('=')) => {
                self.stream.advance();
                Ok(Token::MulAssign(pos))
            },
            _ => Ok(Token::Star(pos))
        }
    }

    fn scan_minus(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok( Some('=') ) => {
                self.stream.advance();
                Ok( Token::SubAssign(pos) )
            },
            Ok( Some('>') ) => {
                self.stream.advance();
                Ok( Token::RightArrow(pos) )
            },
            _ => Ok( Token::Minus(pos) )
        }
    }

    fn scan_plus(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok( Some('=') ) => {
                self.stream.advance();
                Ok( Token::AddAssign(pos) )
            },
            _ => Ok( Token::Plus(pos) )
        }
    }

    fn scan_equals(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok( Some('>') ) => {
                self.stream.advance();
                Ok( Token::Implies(pos))
            },
            Ok( Some('=')) => {
                self.stream.advance();
                Ok( Token::Equals(pos))
            },
            _ => Ok( Token::Assign(pos))
        }
    }

    fn scan_greater(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok( Some('=')) => {
                self.stream.advance();
                Ok(Token::GreaterThan(pos))
            },
            Ok( Some('>')) => {
                self.stream.advance();
                Ok( Token::ShiftRight(pos))
            },
            _ => Ok(Token::Greater(pos)),
        }
    }

    fn scan_less(&mut self) -> Result<Token, LexerError> {
        let pos = self.pos();
        match self.stream.peek() {
            Ok( Some( '=' )) => {
                self.stream.advance();
                Ok( Token::LessThan(pos))
            },
            Ok( Some( '-' )) => {
                self.stream.advance();
                Ok( Token::LeftArrow(pos))
            },
            Ok( Some( '<' )) => {
                self.stream.advance();
                Ok( Token::ShiftLeft(pos))
            }
            _ => Ok( Token::Less(pos)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_colon() {
        let txt = ":: :";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::ScopeSep(Position { column: 1, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Colon(Position { column: 4, line: 1 })));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_dot() {
        let txt = "..... .";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::Range(Position { column: 1, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Range(Position { column: 3, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Dot(Position { column: 5, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Dot(Position { column: 7, line: 1 })));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_caret() {
        let txt = "^ ^=";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::Caret(Position { column: 1, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::EXorAssign(Position { column: 3, line: 1 })));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_vert() {
        let txt = "|= ||  |";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::OrAssign(Position { column: 1, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::LogicOr(Position { column: 4, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Vert(Position { column: 8, line: 1 })));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_ampersand() {
        let txt = "&& & &=";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::LogicAnd(Position { column: 1, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Ampersand(Position { column: 4, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::AndAssign(Position { column: 6, line: 1 })));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_slash() {
        let txt = "  / /= ";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::Slash(Position { column: 3, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::DivAssign(Position { column: 5, line: 1 })));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_star() {
        let txt = "* *= ";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::Star(Position { column: 1, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::MulAssign(Position { column: 3, line: 1 })));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_minus() {
        let txt = " -= - ->";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::SubAssign(Position { column: 2, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Minus(Position { column: 5, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::RightArrow(Position { column: 7, line: 1 })));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_plus() {
        let txt = " += +";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::AddAssign(Position { column: 2, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Plus(Position { column: 5, line: 1 })));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_equals() {
        let txt = " == => =";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::Equals(Position { column: 2, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Implies(Position { column: 5, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Assign( Position{ column: 8, line: 1} )));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_greater() {
        let txt = " >= > >>";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::GreaterThan(Position { column: 2, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::Greater(Position { column: 5, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::ShiftRight( Position{ column: 7, line: 1} )));
        assert_eq!(lxr.get(),  Ok( Token::EndOfFile));
    }

    #[test]
    fn test_less() {
        let txt = " < <= <- <<";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());
        assert_eq!(lxr.get(), Ok(Token::Less(Position { column: 2, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::LessThan(Position { column: 4, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::LeftArrow(Position { column: 7, line: 1 })));
        assert_eq!(lxr.get(), Ok(Token::ShiftLeft(Position { column: 10, line: 1 })));
        assert_eq!(lxr.get(),  Ok( Token::EndOfFile));
    }

    # [test]
    fn test_single_tokens1() {
        let txt = " ((){ \n{}   [ \n ]\n!~ ,;#";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.peek(), Ok( Token::LeftParen( Position{ column: 2, line: 1 } )));
        assert_eq!(lxr.get(),  Ok( Token::LeftParen( Position{ column: 2, line: 1 } )));
        assert_eq!(lxr.get(),  Ok( Token::LeftParen( Position{ column: 3, line: 1 } )));
        assert_eq!(lxr.get(),  Ok( Token::RightParen( Position{ column: 4, line: 1 } )));
        assert_eq!(lxr.get(),  Ok( Token::LeftBrace( Position{ column: 5, line: 1 } )));
        assert_eq!(lxr.get(),  Ok( Token::LeftBrace( Position{ column: 1, line: 2 } )));
        assert_eq!(lxr.get(),  Ok( Token::RightBrace( Position{ column: 2, line: 2 } )));
        assert_eq!(lxr.get(),  Ok( Token::LeftBracket( Position{ column: 6, line: 2 } )));
        assert_eq!(lxr.get(),  Ok( Token::RightBracket( Position{ column: 2, line: 3 } )));
        assert_eq!(lxr.get(),  Ok( Token::ExclamationMark( Position{ column: 1, line: 4 } )));
        assert_eq!(lxr.get(),  Ok( Token::Tilde( Position{ column: 2, line: 4 } )));
        assert_eq!(lxr.get(),  Ok( Token::Comma( Position{ column: 4, line: 4 } )));
        assert_eq!(lxr.get(),  Ok( Token::Semicolon( Position{ column: 5, line: 4 } )));
        assert_eq!(lxr.get(),  Ok( Token::Hash( Position{ column: 6, line: 4 } )));
        assert_eq!(lxr.get(),  Ok( Token::EndOfFile));
    }


}