// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn block_statement(&mut self) {
        self.start_node(BlockStatement);
        self.opt_label();
        self.expect_kw(Kw::Block);
        if self.opt_token(LeftPar) {
            self.condition();
            self.expect_token(RightPar);
        }
        self.opt_token(Keyword(Kw::Is));
        self.block_header();
        self.block_declarative_part();
        self.expect_kw(Kw::Begin);
        self.block_statement_part();
        self.expect_tokens([Keyword(Kw::End), Keyword(Kw::Block)]);
        self.opt_identifier();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn block_header(&mut self) {
        todo!()
    }

    pub fn block_declarative_part(&mut self) {
        todo!()
    }

    pub fn block_statement_part(&mut self) {
        todo!()
    }
}
