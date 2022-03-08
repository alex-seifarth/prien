/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use super::ast::Expression;
use super::tokens;

trait AstVisitor<T> {
    fn visit_expression(&mut self, expr: &Expression) -> T;
}

struct Ast2Json {
    intent_stack: Vec<String>
}

impl AstVisitor<String> for Ast2Json {

    fn visit_expression(&mut self, expr: &Expression) -> String {
        use super::ast::Expression::*;
        match expr {
            Literal(token) => self.visit_literal(token),
            Binary{lhs, operator, rhs} => self.visit_expr_binary(lhs, operator, rhs),
            Unary{operator, rhs} => self.visit_expr_unary(rhs, operator),
            //_ => "".to_string()
        }
    }
}

impl Ast2Json {

    pub fn new() -> Ast2Json {
        Ast2Json{ intent_stack: vec!["".to_string()]}
    }

    fn visit_expr_binary(&mut self, lhs: &Expression, op: &tokens::Token, rhs: &Expression) -> String {
        self.intent_stack.push(self.intent_stack.last().unwrap().clone() + "    ");
        let lhs_str = self.visit_expression(lhs);
        let rhs_str = self.visit_expression(rhs);
        self.intent_stack.pop();

        let intent = self.intent_stack.last().unwrap().clone() + "  ";
        format!("{{\n{}expression: binary,\n{}operator: {},\n{}lhs: {},\n{}rhs: {}\n{}}}",
            intent, intent, Ast2Json::operator_val(op),
            intent, lhs_str, intent, rhs_str, self.intent_stack.last().unwrap())
    }

    fn visit_expr_unary(&mut self, rhs: &Expression, op: &tokens::Token) -> String {
        self.intent_stack.push(self.intent_stack.last().unwrap().clone() + "    ");
        let rhs_str = self.visit_expression(rhs);
        self.intent_stack.pop();

        let intent = self.intent_stack.last().unwrap().clone() + "  ";
        format!("{{\n{}expression: unary,\n{}operator: {},\n{}rhs: {}\n{}}}",
                intent, intent, Ast2Json::operator_val(op),
                intent, rhs_str, self.intent_stack.last().unwrap())
    }

    fn visit_literal(&mut self, token: &tokens::Token) -> String {
        match token {
            tokens::Token::Integer {value, base, source, ..} => {
                format!("{{type: integer, base: {}, literal: {}, value: {} }}",
                        Ast2Json::integer_base_value(base), source, value).to_string()
            },
            _ => "".to_string(),
        }
    }

    fn operator_val(token: &tokens::Token) -> &str{
        match token {
            tokens::Token::Plus(_)      => "+",
            tokens::Token::Minus(_)     => "-",
            tokens::Token::Star(_)      => "*",
            tokens::Token::Slash(_)     => "/",
            tokens::Token::ExclamationMark(_) => "!",
            tokens::Token::Tilde(_)     => "~",
            tokens::Token::Greater(_)   => ">",
            tokens::Token::Less(_)      => "<",
            tokens::Token::GreaterThan(_) => ">=",
            tokens::Token::LessThan(_)  => "<=",
            tokens::Token::Vert(_)      => "|",
            tokens::Token::Ampersand(_) => "&",
            tokens::Token::LogicOr(_)   => "||",
            tokens::Token::LogicAnd(_)  => "&&",
            tokens::Token::Equals(_)    => "==",
            tokens::Token::Unequal(_)   => "!=",
            tokens::Token::AddAssign(_) => "+=",
            tokens::Token::SubAssign(_) => "-=",
            tokens::Token::MulAssign(_) => "*=",
            tokens::Token::DivAssign(_) => "/=",
            tokens::Token::OrAssign(_)  => "|=",
            tokens::Token::AndAssign(_) => "&=",
            _ => panic!("Unsupported token for an operator"),
         }
    }

    fn integer_base_value(base: &tokens::IntegerBase) -> u8 {
        match base {
            tokens::IntegerBase::Decimal        => 10,
            tokens::IntegerBase::Hexadecimal    => 16,
            tokens::IntegerBase::Binary         => 2,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_expression1() {
        let txt = "(1+3)* 0x4 - -2";
        let mut prs = super::super::parser::Parser::create(txt.to_string().into_bytes());

        let expr = prs.expression().unwrap();
        let mut prt = Ast2Json::new();
        let json = prt.visit_expression(&expr);
        assert!(!json.is_empty());
        println!("{}",json);
    }
}