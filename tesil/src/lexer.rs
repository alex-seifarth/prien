use super::Token;
use util::utf8::{Stream, Position};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexerError {
    Unspecified,
    Utf8Error(Position),
    UnexpectedEndOfFile(Position),
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
            '_' | 'a'..='z' | 'A'..='Z' => self.scan_identifier(ch),
            '\'' => self.scan_char_literal(),

            _ => Err( LexerError::Unexpected( self.pos(), ch ) )
        }
    }

    fn scan_char_literal(&mut self) -> Result<Token, LexerError> {
        let start = self.pos();
        return match self.stream.get() {
            Err(_) => Err(LexerError::Utf8Error(start)),
            Ok(None) => Err(LexerError::UnexpectedEndOfFile(start)),
            Ok(Some('\\')) => {
                let ec = self.scan_escaped_char()?;
                self.check_for_char('\'')?;
                return Ok(Token::Char { start, ch: ec })
            },
            Ok(Some(c)) => {
                self.check_for_char('\'')?;
                return Ok(Token::Char { start, ch: c })
            }
        }
    }

    fn scan_escaped_char(&mut self) -> Result<char, LexerError> {
        match self.stream.get() {
            Err( () ) => return Err( LexerError::Utf8Error(self.pos())),
            Ok( None ) => return Err( LexerError::UnexpectedEndOfFile(self.pos())),
            Ok( Some('n') ) => return Ok( '\n' ),
            Ok( Some('t') ) => return Ok( '\t' ),
            Ok( Some('r') ) => return Ok( '\r' ),
            Ok( Some('\\') ) => return Ok( '\\' ),
            Ok( Some('\'') ) => return Ok( '\'' ),
            Ok( Some('"') ) => return Ok( '"' ),
            Ok( Some('u')) | Ok( Some('U')) => {},
            Ok( Some(c) ) => return Err( LexerError::Unexpected(self.pos(), c)),
        };
        self.check_for_char('{')?;

        self.check_for_char('}')?;
        Ok( 'a' )
    }

    fn check_for_char(&mut self, ch: char) -> Result<(), LexerError> {
        match self.stream.get() {
            Err( () ) => return Err( LexerError::Utf8Error(self.pos())),
            Ok( None ) => return Err( LexerError::UnexpectedEndOfFile(self.pos())),
            Ok( Some(c) ) => {
                if c == ch {
                    return Ok( () )
                }
                return Err( LexerError::Unexpected(self.pos(), c))
            },
        };
    }

    fn scan_identifier(&mut self, ch: char) -> Result<Token, LexerError> {
        let start = self.pos();
        let mut v= vec![ch];
        loop {
            let next_char = match self.stream.peek() {
                Err(()) => break,
                Ok(None) => break,
                Ok(Some(c)) => c,
            };
            match next_char {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    self.stream.advance();
                    v.push(next_char);
                },
                _ => break,
            }
        }
        let str : String = v.into_iter().collect();
        match str.as_ref() {
            "import"    => Ok( Token::KwImport(start) ),
            "i8"        => Ok( Token::KwTypeI8(start) ),
            "i16"       => Ok( Token::KwTypeI16(start) ),
            "i32"       => Ok( Token::KwTypeI32(start) ),
            "i64"       => Ok( Token::KwTypeI64(start) ),
            "u8"        => Ok( Token::KwTypeU8(start) ),
            "u16"       => Ok( Token::KwTypeU16(start) ),
            "u32"       => Ok( Token::KwTypeU32(start) ),
            "u64"       => Ok( Token::KwTypeU64(start) ),
            "bool"      => Ok( Token::KwTypeBool(start) ),
            "f32"       => Ok( Token::KwTypeF32(start) ),
            "f64"       => Ok( Token::KwTypeF64(start) ),
            "char"      => Ok( Token::KwTypeChar(start) ),
            "fn"        => Ok( Token::KwFn(start) ),
            "struct"    => Ok( Token::KwStruct(start) ),
            "enum"      => Ok( Token::KwEnum(start) ),
            "type"      => Ok( Token::KwType(start) ),
            "break"     => Ok( Token::KwBreak(start) ),
            "continue"  => Ok( Token::KwContinue(start) ),
            "expect"    => Ok( Token::KwExpect(start) ),
            "let"       => Ok( Token::KwLet(start) ),
            "mut"       => Ok( Token::KwMut(start) ),
            _           => Ok( Token::Identifier {start, source: str, end: self.pos() })
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
            Ok(Some('/')) => {
                self.stream.advance();
                let mut str = vec![];
                loop {
                    match self.stream.peek() {
                        Err(()) => break,
                        Ok(Some('\n')) | Ok(None) => break,
                        Ok(Some(ch)) => {
                            self.stream.advance();
                            str.push(ch);
                        },
                    }
                }
                Ok(Token::Comment{start: pos, comment: str.into_iter().collect()})
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
    fn test_char_literal() {
        let txt = "'a' 'z''\\n' '\\t' '\\r' '\\\\' '\\\'' '\\\"'";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok( Token::Char{ start: Position{ line:1, column: 1}, ch: 'a' } ) );
        assert_eq!(lxr.get(), Ok( Token::Char{ start: Position{ line:1, column: 5}, ch: 'z' } ) );
        assert_eq!(lxr.get(), Ok( Token::Char{ start: Position{ line:1, column: 8}, ch: '\n' } ) );
        assert_eq!(lxr.get(), Ok( Token::Char{ start: Position{ line:1, column: 13}, ch: '\t' } ) );
        assert_eq!(lxr.get(), Ok( Token::Char{ start: Position{ line:1, column: 18}, ch: '\r' } ) );
        assert_eq!(lxr.get(), Ok( Token::Char{ start: Position{ line:1, column: 23}, ch: '\\' } ) );
        assert_eq!(lxr.get(), Ok( Token::Char{ start: Position{ line:1, column: 28}, ch: '\'' } ) );
        assert_eq!(lxr.get(), Ok( Token::Char{ start: Position{ line:1, column: 33}, ch: '"' } ) );

    }

    #[test]
    fn test_comments() {
        let txt = concat!(
            "varname // this is a variable \n",
            "//full line comment\n",
            "!"
        );
        let mut lxr = Lexer::create(txt.to_string().into_bytes());
        assert_eq!(lxr.get(), Ok( Token::Identifier {start: Position{ line: 1, column: 1},
            end: Position{ line: 1, column: 7 }, source: "varname".to_string()}));
        assert_eq!(lxr.get(), Ok( Token::Comment {start: Position{ line: 1, column: 9},
            comment: " this is a variable ".to_string()}));
        assert_eq!(lxr.get(), Ok( Token::Comment {start: Position{ line: 2, column: 1},
            comment: "full line comment".to_string()}));
        assert_eq!(lxr.get(), Ok( Token::ExclamationMark( Position{ line: 3, column: 1})));
        assert_eq!(lxr.get(), Ok( Token::EndOfFile));
    }

    #[test]
    fn test_keywords() {
        let txt = concat!("import i8 i16 i32 i64 u8 u16 u32 u64 \n",
            "bool f32 f64 char fn struct enum\n",
            "type break continue expect let mut");
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::KwImport( Position{ column: 1, line: 1} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeI8( Position{ column: 8, line: 1} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeI16( Position{ column: 11, line: 1} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeI32( Position{ column: 15, line: 1} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeI64( Position{ column: 19, line: 1} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeU8( Position{ column: 23, line: 1} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeU16( Position{ column: 26, line: 1} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeU32( Position{ column: 30, line: 1} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeU64( Position{ column: 34, line: 1} )));

        assert_eq!(lxr.get(), Ok(Token::KwTypeBool( Position{ column: 1, line: 2} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeF32( Position{ column: 6, line: 2} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeF64( Position{ column: 10, line: 2} )));
        assert_eq!(lxr.get(), Ok(Token::KwTypeChar( Position{ column: 14, line: 2} )));
        assert_eq!(lxr.get(), Ok(Token::KwFn( Position{ column: 19, line: 2} )));
        assert_eq!(lxr.get(), Ok(Token::KwStruct( Position{ column: 22, line: 2} )));
        assert_eq!(lxr.get(), Ok(Token::KwEnum( Position{ column: 29, line: 2} )));

        assert_eq!(lxr.get(), Ok(Token::KwType( Position{ column: 1, line: 3} )));
        assert_eq!(lxr.get(), Ok(Token::KwBreak( Position{ column: 6, line: 3} )));
        assert_eq!(lxr.get(), Ok(Token::KwContinue( Position{ column: 12, line: 3} )));
        assert_eq!(lxr.get(), Ok(Token::KwExpect( Position{ column: 21, line: 3} )));
        assert_eq!(lxr.get(), Ok(Token::KwLet( Position{ column: 28, line: 3} )));
        assert_eq!(lxr.get(), Ok(Token::KwMut( Position{ column: 32, line: 3} )));
    }

    #[test]
    fn test_identifier() {
        let txt = "my_identifier _anotherOne1 Zzz\n";
        let mut lxr = Lexer::create(txt.to_string().into_bytes());

        assert_eq!(lxr.get(), Ok(Token::Identifier{start: Position{ line: 1, column: 1},
            source: "my_identifier".to_string(), end: Position{ line:1, column: 13}} ) );
        assert_eq!(lxr.get(), Ok(Token::Identifier{start: Position{ line: 1, column: 15},
            source: "_anotherOne1".to_string(), end: Position{ line:1, column: 26}} ) );
        assert_eq!(lxr.get(), Ok(Token::Identifier{start: Position{ line: 1, column: 28},
            source: "Zzz".to_string(), end: Position{ line:1, column: 30}} ) );
    }

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