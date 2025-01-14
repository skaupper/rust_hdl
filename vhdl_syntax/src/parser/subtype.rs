// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::Parser;
use crate::tokens::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    fn end_of_subtype_indication(&mut self) -> bool {
        let possible_end_tokens = [
            SemiColon,
            ColonEq,
            Comma,
            RightPar,
            Bar,
            RightArrow,
            GtGt,
            Pow,
            Times,
            Div,
            Plus,
            Minus,
            Concat,
            Keyword(Kw::Generate),
            Keyword(Kw::Loop),
            Keyword(Kw::Register),
            Keyword(Kw::Bus),
            Keyword(Kw::Open),
            Keyword(Kw::Is),
            Keyword(Kw::Bus),
            Keyword(Kw::Is),
            Keyword(Kw::Mod),
            Keyword(Kw::Rem),
            Keyword(Kw::To),
            Keyword(Kw::Downto),
            Keyword(Kw::Units),
        ];
        let is_end_of_subtype_indication = possible_end_tokens.iter().any(|t| self.next_is(*t));
        is_end_of_subtype_indication
    }

    fn array_element_constraint(&mut self) {
        todo!();
    }

    fn opt_array_element_constraint(&mut self) -> bool {
        todo!();
    }

    fn record_element_constraint(&mut self) {
        todo!();
    }

    fn index_constraint(&mut self) {
        todo!();
    }

    fn range_constraint(&mut self) {
        todo!();
    }

    fn array_constraint(&mut self) {
        match self.peek_nth_token(2) {
            Some(Keyword(Kw::Open)) => self.expect_tokens([LeftPar, Keyword(Kw::Open), RightPar]),
            Some(_) => self.index_constraint(),
            None => {
                self.eof_err();
                return;
            }
        }

        self.opt_array_element_constraint();
    }

    fn record_constraint(&mut self) {
        self.expect_token(LeftPar);
        self.separated_list(Parser::record_element_constraint, Comma);
        self.expect_token(RightPar);
    }

    fn constraint(&mut self) {
        todo!();
    }

    fn opt_constraint(&mut self) -> bool {
        let possible_tokens = [Keyword(Kw::Range), LeftPar];
        let is_constraint = possible_tokens.iter().any(|t| self.next_is(*t));
        if !is_constraint {
            return false;
        }

        self.constraint();
        true
    }

    pub fn subtype_indication(&mut self) {
        self.name_group();
        if self.opt_constraint() || self.end_of_subtype_indication() {
            return;
        }

        self.name_group();
        self.opt_constraint();
    }
}
