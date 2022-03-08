/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use super::tokens::Token;
use super::lexer::Lexer;
use crate::Expression;
use super::ast;

#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    MissingToken(String)
}

/// Parser for TESIL language files producing the corresponding TESIL AST.
pub struct Parser {
    lexer: Lexer,
}

/// Checks whether next token matches one of the given patterns and returns it as 'Some(token)'
/// or `None` if no match is found.
/// Usage: use inside of lexer method as: `matches(self, Token::LeftParen(_), Token::RightParen(_))`
macro_rules! matches {
    ($self:ident, $($pats:pat),*) => {
        match $self.lexer.peek() {
            $(Ok($pats) => Some($self.lexer.get()),)*
            _ => None
        }
    }
}

macro_rules! check_token {
    ($self:ident, $pat:pat, $msg:expr) => {
        match $self.lexer.peek() {
            Ok($pat) => {
                let _ = $self.lexer.get();
                Ok(())
            },
            _ => {
                Err(ParseError::MissingToken($msg))
            }
        }
    }
}

impl Parser {

    pub fn create(data: Vec<u8>) -> Parser {
        let lexer = Lexer::create( data );
        Parser{ lexer }
    }
    //
    // fn eof(&mut self) -> bool {
    //     self.lexer.peek() == Ok( Token::EndOfFile )
    // }

    pub fn expression(&mut self) -> Result<ast::Expression, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<ast::Expression, ParseError> {
        let mut expr = self.comparison()?;
        while let Some(tk) = matches!(self, Token::Equals(_), Token::Unequal(_)) {
            expr = ast::Expression::Binary {lhs: Box::new(expr), operator: tk.unwrap(),
                rhs: Box::new( self.comparison()?) }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<ast::Expression, ParseError> {
        let mut expr = self.term()?;
        while let Some(tk) = matches!(self, Token::Greater(_), Token::GreaterThan(_),
                Token::Less(_), Token::LessThan(_)) {
            expr = ast::Expression::Binary {lhs: Box::new(expr), operator: tk.unwrap(),
                rhs: Box::new( self.term()?) }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<ast::Expression, ParseError> {
        let mut expr = self.factor()?;
        while let Some(tk) = matches!(self, Token::Minus(_), Token::Plus(_) ) {
            expr = ast::Expression::Binary {lhs: Box::new(expr), operator: tk.unwrap(),
                                            rhs: Box::new( self.factor()?) }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<ast::Expression, ParseError> {
        let mut expr = self.unary()?;
        while let Some(tk) = matches!(self, Token::Star(_), Token::Slash(_)) {
            expr = ast::Expression::Binary {lhs: Box::new(expr), operator: tk.unwrap(),
                                            rhs: Box::new(self.unary()?) }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<ast::Expression, ParseError> {
        if let Some(tk) =
                matches!(self, Token::Minus(_),Token::ExclamationMark(_), Token::Tilde(_)) {
            return Ok( ast::Expression::Unary {operator: tk.unwrap(), rhs: Box::new(self.unary()?) } )
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<ast::Expression, ParseError> {
        if let Some(tk) = matches!(self, Token::Integer{..},
                Token::FloatNumber {..}, Token::String{..}, Token::Char {..}, Token::KwFalse(_),
                Token::KwTrue(_)) {
            return Ok( Expression::Literal(tk.unwrap()))
        }
        else if let Ok(Token::LeftParen(pos)) = self.lexer.peek() {
            self.advance();
            let expr = self.expression()?;
            check_token!(self, Token::RightParen(_),
                format!("Missing closing parentheses for opening parentheses ({}).", pos))?;
            return Ok( expr )
        }
        Err(ParseError::MissingToken(format!("Expected literal ({}).", self.lexer.pos())))
    }

    fn advance(&mut self) {
        let _ = self.lexer.get().unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use util::utf8::Position;
    use super::super::tokens::IntegerBase;

    fn position(line: u32, column: u32) -> Position {
        Position{ line, column }
    }

    #[test]
    fn test_expression_factor() {
        let txt = "1*2 \"ajb\"/\"bca\"";
        let mut prs = Parser::create(txt.to_string().into_bytes());

        assert_eq!(prs.factor(), Ok( Expression::Binary {
            lhs: Box::new(Expression::Literal(
                Token::Integer{start: position(1,1), end: position(1,1),
                                               source: "1".to_string(), value: 1, base: IntegerBase::Decimal })),
            operator: Token::Star(position(1, 2)),
            rhs: Box::new(Expression::Literal(
                Token::Integer{start: position(1,3), end: position(1,3),
                    source: "2".to_string(), value: 2, base: IntegerBase::Decimal })),
        }));

        assert_eq!(prs.factor(), Ok( Expression::Binary {
            lhs: Box::new(Expression::Literal(
                Token::String{start: position(1,5), end: position(1,9), source: "ajb".to_string() })),
            operator: Token::Slash(position(1, 10)),
            rhs: Box::new(Expression::Literal(
                Token::String{start: position(1,11), end: position(1,15), source: "bca".to_string()})),
        }));
    }

    #[test]
    fn test_expression_unary() {
        let txt = "1245 (2.3) !false ~22 -42";
        let mut prs = Parser::create(txt.to_string().into_bytes());

        assert_eq!(prs.unary(), Ok( Expression::Literal(
            Token::Integer{start: position(1,1), end: position(1, 4),
                source:"1245".to_string(), value: 1245, base: IntegerBase::Decimal })));
        assert_eq!(prs.unary(), Ok( Expression::Literal(
            Token::FloatNumber{start: position(1, 7), end: position(1,9),
                source:"2.3".to_string(), value: 2.3 })));
        assert_eq!(prs.unary(), Ok( Expression::Unary {
            operator: Token::ExclamationMark(position(1, 12)),
            rhs: Box::new(Expression::Literal( Token::KwFalse(position(1,13)) ))}));
        assert_eq!(prs.unary(), Ok( Expression::Unary {
            operator: Token::Tilde(position(1, 19)),
            rhs: Box::new(Expression::Literal(
                Token::Integer{start: position(1,20), end: position(1,21),
                    source:"22".to_string(), value: 22, base: IntegerBase::Decimal}))}));
        assert_eq!(prs.unary(), Ok( Expression::Unary {
            operator: Token::Minus(position(1, 23)),
            rhs: Box::new(Expression::Literal(
                Token::Integer{start: position(1,24), end: position(1,25),
                    source:"42".to_string(), value: 42, base: IntegerBase::Decimal}))}));
    }
}
