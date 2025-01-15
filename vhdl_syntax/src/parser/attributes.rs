// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::match_next_token;
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn attribute_specification(&mut self) {
        self.start_node(AttributeSpecification);
        self.identifier();
        self.expect_token(Keyword(Kw::Of));
        self.entity_specification();
        self.expect_token(Keyword(Kw::Is));
        self.expression();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn entity_specification(&mut self) {
        self.start_node(EntitySpecification);
        self.entity_name_list();
        self.expect_token(Colon);
        self.entity_class();
        self.end_node();
    }

    pub fn entity_name_list(&mut self) {
        self.start_node(EntityNameList);
        match_next_token!(self,
            Keyword(Kw::All) => self.skip(),
            Keyword(Kw::Others) => self.skip(),
            Identifier, StringLiteral, CharacterLiteral => {
                self.start_node(DesignatorList);
                self.separated_list(Parser::designator, Comma);
                self.end_node();
            }
        );
        self.end_node();
    }

    pub fn entity_class(&mut self) {
        self.expect_one_of_tokens([
            Keyword(Kw::Entity),
            Keyword(Kw::Architecture),
            Keyword(Kw::Configuration),
            Keyword(Kw::Procedure),
            Keyword(Kw::Function),
            Keyword(Kw::Package),
            Keyword(Kw::Type),
            Keyword(Kw::Subtype),
            Keyword(Kw::Constant),
            Keyword(Kw::Signal),
            Keyword(Kw::Variable),
            Keyword(Kw::Component),
            Keyword(Kw::Label),
            Keyword(Kw::Literal),
            Keyword(Kw::Units),
            Keyword(Kw::Group),
            Keyword(Kw::File),
            Keyword(Kw::Property),
            Keyword(Kw::Sequence),
        ])
    }

    pub fn entity_designator(&mut self) {
        self.start_node(EntityDesignator);
        self.entity_tag();
        if self.peek_token() == Some(LeftSquare) {
            self.signature();
        }
        self.end_node();
    }

    pub fn entity_tag(&mut self) {
        self.expect_one_of_tokens([Identifier, CharacterLiteral, StringLiteral])
    }
}
