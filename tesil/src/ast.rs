/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use super::tokens::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Binary{ lhs: Box<Expression>, operator: Token, rhs: Box<Expression> },
    Unary{ operator: Token, rhs: Box<Expression> },
    Literal(Token),
}