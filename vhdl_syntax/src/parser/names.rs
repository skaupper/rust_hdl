// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::Name;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn designator(&mut self) {
        self.expect_one_of_tokens([Identifier, StringLiteral, CharacterLiteral]);
    }

    pub fn name(&mut self) {
        self.start_node(Name);
        // TODO
        self.designator();
        self.end_node();
    }
}
