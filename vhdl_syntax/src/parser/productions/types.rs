// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn type_declaration(&mut self) {
        self.start_node(TypeDeclaration);
        self.expect_kw(Kw::Type);
        self.identifier();
        if self.opt_token(SemiColon) {
            self.end_node();
            return;
        }
        self.expect_kw(Kw::Is);
        self.type_definition();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn type_definition(&mut self) {
        match_next_token!(self,
            Keyword(Kw::Range) => todo!(),
            Keyword(Kw::Access) => {
                self.start_node(AccessTypeDefinition);
                self.skip();
                self.subtype_indication();
                self.end_node();
            },
            Keyword(Kw::Protected) => todo!(),
            Keyword(Kw::File) => todo!(),
            Keyword(Kw::Array) => todo!(),
            Keyword(Kw::Record) => todo!(),
            LeftPar => todo!()
        )
    }

    pub fn subtype_declaration(&mut self) {
        self.start_node(SubtypeDeclaration);
        self.expect_kw(Kw::Subtype);
        self.identifier();
        self.expect_kw(Kw::Is);
        self.subtype_indication();
        self.expect_token(SemiColon);
        self.end_node();
    }
}
