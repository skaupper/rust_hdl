// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2024, Lukas Scheller lukasscheller@icloud.com
/// Parsing of different names
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::{Keyword as Kw, TokenKind::*, TokenStream};

impl<T: TokenStream> Parser<T> {
    fn external_name(&mut self) {
        self.start_node(ExternalName);
        self.expect_token(LtLt);

        while !self.next_is(GtGt) {
            self.push_next();
        }

        self.expect_token(GtGt);
        self.end_node();
    }

    fn prefix(&mut self) {
        if self.next_is(LtLt) {
            self.external_name();
        } else {
            self.expect_one_of_tokens([Identifier, StringLiteral, CharacterLiteral]);
        }
    }

    fn bracket_group(&mut self) {
        self.start_node(BracketGroup);
        self.expect_token(LeftSquare);
        let mut bracket_count = 1;

        while bracket_count > 0 {
            if self.next_is(LeftSquare) {
                bracket_count += 1;
                self.start_node(BracketGroup);
                self.push_next();
            } else if self.next_is(RightSquare) {
                bracket_count -= 1;
                self.push_next();
                self.end_node();
            } else {
                self.push_next();
            }
        }
    }

    fn paren_group(&mut self) {
        self.start_node(ParenGroup);
        self.expect_token(LeftPar);
        self.start_node(ParenChildGroup);
        let mut paren_count = 1;

        while paren_count > 0 {
            if self.next_is(LeftPar) {
                paren_count += 1;
                self.start_node(ParenGroup);
                self.push_next();
                self.start_node(ParenChildGroup);
            } else if self.next_is(RightPar) {
                paren_count -= 1;
                self.end_node();
                self.push_next();
                self.end_node();
            } else if self.next_is(Comma) {
                self.end_node();
                self.push_next();
                self.start_node(ParenChildGroup);
            } else {
                self.push_next();
            }
        }
    }

    fn name_tail(&mut self) {
        if self.opt_token(Dot) {
            let parsed_token = self.opt_tokens([
                Identifier,
                StringLiteral,
                CharacterLiteral,
                Keyword(Kw::All),
            ]);
            if let None = parsed_token {
                return;
            }
            self.name_tail();
        } else if self.opt_token(Tick) {
            self.identifier();
            if self.next_is(LeftPar) {
                self.paren_group();
            }
            self.name_tail();
        } else if self.next_is(LeftSquare) {
            self.bracket_group();
            self.name_tail();
        } else if self.next_is(LeftPar) {
            self.paren_group();
            self.name_tail();
        }
    }

    pub fn name_group(&mut self) {
        self.start_node(NameGroup);
        self.prefix();
        self.name_tail();
        self.end_node();
    }

    pub fn name_group_list(&mut self) {
        self.start_node(NameGroupList);
        self.separated_list(Parser::name_group, Comma);
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{test_utils::check, Parser};

    #[test]
    fn parse_name() {
        check(
            Parser::name_group,
            "lib1.fn('a', 1+45, sig).vector(100 downto -10).all",
            "\
NameGroup
  Identifier 'lib1'
  Dot
  Identifier 'fn'
  ParenGroup
    LeftPar
    ParenChildGroup
      CharacterLiteral
    Comma
    ParenChildGroup
      AbstractLiteral
      Plus
      AbstractLiteral
    Comma
    ParenChildGroup
      Identifier 'sig'
    RightPar
  Dot
  Identifier 'vector'
  ParenGroup
    LeftPar
    ParenChildGroup
      AbstractLiteral
      Keyword(Downto)
      Minus
      AbstractLiteral
    RightPar
  Dot
  Keyword(All)
",
        );
    }
}
