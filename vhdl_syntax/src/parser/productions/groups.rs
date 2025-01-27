// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com
/// Parsing of group related rules (LRM §6.9-§6.10)
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub(crate) fn group_declaration(&mut self) {
        self.start_node(GroupDeclaration);
        self.expect_kw(Kw::Group);
        self.identifier();
        self.expect_token(Colon);
        // The `group_constituent_list` is absorbed by the greedy `.name()`!
        self.name();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub(crate) fn group_template_declaration(&mut self) {
        self.start_node(GroupTemplateDeclaration);
        self.expect_kw(Kw::Group);
        self.identifier();
        self.expect_kw(Kw::Is);
        self.expect_token(LeftPar);
        self.entity_class_entry_list();
        self.expect_token(RightPar);
        self.expect_token(SemiColon);
        self.end_node();
    }

    fn entity_class_entry_list(&mut self) {
        self.start_node(EntityClassEntryList);
        self.separated_list(Parser::entity_class_entry, Comma);
        self.end_node();
    }

    fn entity_class_entry(&mut self) {
        self.start_node(EntityClassEntry);
        self.entity_class();
        self.opt_token(BOX);
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::*;
    use crate::parser::Parser;

    #[test]
    fn group_declaration() {
        check(
            Parser::group_declaration,
            "group G1 : my_group_template (s1);",
            "\
GroupDeclaration
  Keyword(Group)
  Identifier 'G1'
  Colon
  Name
    Identifier 'my_group_template'
    RawTokens
      LeftPar
      Identifier 's1'
      RightPar
  SemiColon
",
        );
    }

    #[test]
    fn group_template_declaration() {
        check(
            Parser::group_template_declaration,
            "group G1_TMPL is (label);",
            "\
GroupTemplateDeclaration
  Keyword(Group)
  Identifier 'G1_TMPL'
  Keyword(Is)
  LeftPar
  EntityClassEntryList
    EntityClassEntry
      Keyword(Label)
  RightPar
  SemiColon
",
        );

        check(
            Parser::group_template_declaration,
            "group G2_TMPL is (signal, signal <>, label <>, group);",
            "\
GroupTemplateDeclaration
  Keyword(Group)
  Identifier 'G2_TMPL'
  Keyword(Is)
  LeftPar
  EntityClassEntryList
    EntityClassEntry
      Keyword(Signal)
    Comma
    EntityClassEntry
      Keyword(Signal)
      BOX
    Comma
    EntityClassEntry
      Keyword(Label)
      BOX
    Comma
    EntityClassEntry
      Keyword(Group)
  RightPar
  SemiColon
",
        );
    }
}
