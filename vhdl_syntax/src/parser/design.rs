// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2024, Lukas Scheller lukasscheller@icloud.com
use crate::parser::Parser;
/// Parsing of design files, and abstract design units.
/// The concrete design units (entity, architecture, ...) live in their own file.
use crate::syntax::node_kind::NodeKind;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::token_kind::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn design_file(&mut self) {
        self.start_node(NodeKind::DesignFile);
        loop {
            self.design_unit();
            if self.tokenizer.peek_next().is_none() {
                break;
            }
        }
        self.end_node();
    }

    pub fn design_unit(&mut self) {
        if !self.tokenizer.has_next() {
            self.eof_err();
            return;
        }
        self.start_node(NodeKind::DesignUnit);
        self.context_clause();
        match self.tokenizer.peek_next() {
            Some(tok) => match tok.kind() {
                Keyword(Kw::Entity) => self.entity(),
                Keyword(Kw::Configuration) => todo!(),
                Keyword(Kw::Package) => todo!(),
                Keyword(Kw::Context) => todo!(),
                Keyword(Kw::Architecture) => todo!(),
                _ => {
                    self.expect_tokens_err([
                        Keyword(Kw::Entity),
                        Keyword(Kw::Configuration),
                        Keyword(Kw::Package),
                        Keyword(Kw::Context),
                        Keyword(Kw::Architecture),
                    ]);
                    // TODO: resynchronize the parser instead
                    self.tokenizer.next();
                }
            },
            None => self.eof_err(),
        }
        self.end_node();
    }

    pub fn context_clause(&mut self) {
        loop {
            match self.tokenizer.peek_next() {
                Some(tok) => match tok.kind() {
                    Keyword(Kw::Use) => self.use_clause(),
                    Keyword(Kw::Library) => self.library_clause(),
                    Keyword(Kw::Context) => self.context_reference(),
                    _ => break,
                },
                _ => self.eof_err(),
            }
        }
    }

    pub fn library_clause(&mut self) {
        self.start_node(NodeKind::LibraryClause);
        self.expect_token(Keyword(Kw::Library));
        self.identifier_list();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn use_clause(&mut self) {
        self.start_node(NodeKind::UseClause);
        self.expect_token(Keyword(Kw::Use));
        self.name_group_list();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn context_reference(&mut self) {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{CanParse, Parser};
    use crate::tokens;
    use crate::tokens::IntoTokenStream;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_simple_entity() {
        let (entity, _) = tokens! {
            entity my_ent is
            begin
            end my_ent;

            entity my_ent2 is
            begin
            end entity;
        }
        .into_token_stream()
        .parse_syntax(Parser::design_file);
        assert_eq!(
            entity.test_text(),
            "\
DesignFile
  DesignUnit
    EntityDeclaration
      Keyword(Entity)
      Identifier 'my_ent'
      Keyword(Is)
      EntityHeader
      Keyword(Begin)
      Keyword(End)
      Identifier 'my_ent'
      SemiColon
  DesignUnit
    EntityDeclaration
      Keyword(Entity)
      Identifier 'my_ent2'
      Keyword(Is)
      EntityHeader
      Keyword(Begin)
      Keyword(End)
      Keyword(Entity)
      SemiColon
"
        );
    }

    #[test]
    fn parse_entity_with_context_clause() {
        let (design, _) = "\
            library ieee;
            use ieee.std_logic_1164.all;

            entity my_ent is
            begin
            end my_ent;
        "
        .parse_syntax(Parser::design_file);
        assert_eq!(
            design.test_text(),
            "\
DesignFile
  DesignUnit
    LibraryClause
      Keyword(Library)
      IdentifierList
        Identifier 'ieee'
      SemiColon
    UseClause
      Keyword(Use)
      NameGroupList
        NameGroup
          Identifier 'ieee'
          Dot
          Identifier 'std_logic_1164'
          Dot
          Keyword(All)
      SemiColon
    EntityDeclaration
      Keyword(Entity)
      Identifier 'my_ent'
      Keyword(Is)
      EntityHeader
      Keyword(Begin)
      Keyword(End)
      Identifier 'my_ent'
      SemiColon
"
        );
    }

    #[test]
    fn parse_use_clause() {
        let (node, diag) = "use lib1.lib2.lib3.all;".parse_syntax(Parser::use_clause);
        assert_eq!(diag.len(), 0);

        assert_eq!(
            node.test_text(),
            "\
UseClause
  Keyword(Use)
  NameGroupList
    NameGroup
      Identifier 'lib1'
      Dot
      Identifier 'lib2'
      Dot
      Identifier 'lib3'
      Dot
      Keyword(All)
  SemiColon
"
        );
    }
}
