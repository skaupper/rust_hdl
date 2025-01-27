// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2024, Lukas Scheller lukasscheller@icloud.com
/// Parsing of packages (LRM §4.7-§4.9)
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::token_kind::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn package_declaration(&mut self) {
        self.start_node(PackageDeclaration);
        self.expect_token(Keyword(Kw::Package));
        self.identifier();
        self.expect_token(Keyword(Kw::Is));
        self.package_header();
        self.package_declarative_part();
        self.expect_token(Keyword(Kw::End));
        self.opt_token(Keyword(Kw::Package));
        self.opt_identifier();
        self.expect_token(SemiColon);
        self.end_node();
    }

    fn package_header(&mut self) {
        self.start_node(PackageHeader);
        if self.opt_generic_clause() && self.opt_generic_map_aspect() {
            self.expect_token(SemiColon);
        }
        self.end_node();
    }

    pub(crate) fn package_instantiation_declaration(&mut self) {
        self.start_node(PackageInstantiationDeclaration);
        self.expect_tokens([
            Keyword(Kw::Package),
            Identifier,
            Keyword(Kw::Is),
            Keyword(Kw::New),
        ]);
        self.name();
        self.opt_generic_map_aspect();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub(crate) fn package_body(&mut self) {
        self.start_node(PackageBody);
        self.expect_kw(Kw::Package);
        self.expect_kw(Kw::Body);

        self.package_body_declarative_part();

        self.expect_kw(Kw::End);
        if self.next_is(Keyword(Kw::Package)) {
            self.expect_kw(Kw::Package);
            self.expect_kw(Kw::Body);
        }
        self.opt_identifier();
        self.expect_token(SemiColon);
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::check;
    use crate::parser::Parser;

    #[test]
    #[ignore]
    fn dummy() {
        check(Parser::package_declaration, "", "");
    }
}
