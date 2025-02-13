// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn expression(&mut self) {
        self.start_node(Expression);
        // TODO: Expecting a simple expression is just a placeholder
        self.simple_expression();
        self.end_node();
    }

    pub fn simple_expression(&mut self) {
        self.start_node(SimpleExpression);
        // TODO: Expecting these literals is just a placeholder
        self.expect_one_of_tokens([CharacterLiteral, StringLiteral, Identifier, AbstractLiteral]);
        self.end_node();
    }

    pub fn expression_list(&mut self) {
        self.start_node(ExpressionList);
        self.separated_list(Parser::expression, Comma);
        self.end_node();
    }

    pub fn condition(&mut self) {
        self.expression()
    }
}
