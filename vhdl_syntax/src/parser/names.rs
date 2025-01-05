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
    // §8.1: name                   ::=   simple_name
    //                                  | operator_symbol
    //                                  | character_literal
    //                                  | selected_name
    //                                  | indexed_name
    //                                  | slice_name
    //                                  | attribute_name
    //                                  | external_name
    //       prefix                 ::=   name
    //                                  | function_call
    // §8.2: simple_name            ::=   identifier
    // §8.3: selected_name          ::=   prefix . suffix
    //       suffix                 ::=   simple_name
    //                                  | character_literal
    //                                  | operator_symbol
    //                                  | "all"
    // §8.4: indexed_name           ::=   prefix ( expression { , expression } )
    // §8.5: slice_name             ::=   prefix ( discrete_range )
    // §8.6: attribute_name         ::=   prefix [ signature ] ' attribute_designator [ ( expression ) ]
    //       attribute_designator   ::=   attribute_simple_name
    // §8.7: external_name          ::=   external_constant_name
    //                                  | external_signal_name
    //                                  | external_variable_name

    fn name_terminals(&mut self) {
        self.expect_one_of_tokens([Identifier, StringLiteral, CharacterLiteral]);
    }

    pub fn name(&mut self) {
        self.start_node(Name);
        if self.opt_token(LtLt) {
            // external_name
            todo!();
        }

        self.name_terminals();
        self.builder.embed_node(NamePrefix);

        loop {
            match self.peek_token() {
                Some(Dot) => {
                    self.expect_token(Dot);
                    if !self.opt_token(Keyword(Kw::All)) {
                        self.name_terminals()
                    }
                    self.builder.embed_node(SelectedName);
                }
                Some(LeftSquare) => {
                    todo!()
                }
                Some(Tick) => {
                    todo!()
                }
                Some(LeftPar) => {
                    todo!()
                }
                _ => break,
            }
        }

        self.end_node();
    }

    pub fn selected_name(&mut self) {
        self.start_node(SelectedName);

        self.name_terminals();
        self.builder.embed_node(NamePrefix);

        while self.opt_token(Dot) {
            if !self.opt_token(Keyword(Kw::All)) {
                self.name_terminals();
            }
            self.builder.embed_node(SelectedName);
        }

        self.end_node();
    }

    pub fn selected_name_list(&mut self) {
        self.start_node(SelectedNameList);
        self.separated_list(Parser::selected_name, Comma);
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{CanParse, Parser};

    #[test]
    fn parse_name() {
        let text = "lib1.lib2.lib3.all";
        let (node, diag) = text.parse_syntax(Parser::name);
        println!("{}", node.test_text());
        println!("{}", diag.len());
    }

    #[test]
    fn parse_selected_name() {
        let text = "lib1.lib2.lib3.all";
        let (node, diag) = text.parse_syntax(Parser::selected_name);
        println!("{}", node.test_text());
        println!("{}", diag.len());
    }
    #[test]
    fn parse_use_clause() {
        let text = "use lib1.lib2.lib3.all;";
        let (node, diag) = text.parse_syntax(Parser::use_clause);

        println!("{}", node.test_text());
        println!("{}", diag.len());
    }
}
