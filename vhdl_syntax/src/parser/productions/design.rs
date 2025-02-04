//! Parsing of design files, and abstract design units.
//! The concrete design units (entity, architecture, ...) live in their own file.
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2024, Lukas Scheller lukasscheller@icloud.com
use crate::match_next_token;
use crate::parser::Parser;
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
        match_next_token!(self,
            Keyword(Kw::Entity) => self.entity(),
            Keyword(Kw::Configuration) => todo!(),
            Keyword(Kw::Package) => {
                let is_pkg_inst_decl = self.next_is_seq([
                    Keyword(Kw::Package),
                    Identifier,
                    Keyword(Kw::Is),
                    Keyword(Kw::New),
                ]);
                let is_pkg_body = self.next_is_seq([Keyword(Kw::Package), Keyword(Kw::Body)]);

                if is_pkg_inst_decl {
                    self.package_instantiation_declaration();
                } else if is_pkg_body {
                    self.package_body();
                } else {
                    self.package_declaration();
                }
            },
            Keyword(Kw::Context) => todo!(),
            Keyword(Kw::Architecture) => todo!()
        );
        self.end_node();
    }

    pub fn context_clause(&mut self) {
        self.start_node(NodeKind::ContextClause);
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
        self.end_node();
    }

    pub fn library_clause(&mut self) {
        self.start_node(NodeKind::LibraryClause);
        self.expect_kw(Kw::Library);
        self.identifier_list();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn use_clause(&mut self) {
        self.start_node(NodeKind::UseClause);
        self.expect_kw(Kw::Use);
        self.name_list();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn context_reference(&mut self) {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{test_utils::check, Parser};

    #[test]
    fn parse_simple_entity() {
        check(
            Parser::design_file,
            "\
entity my_ent is
begin
end my_ent;

entity my_ent2 is
begin
end entity;
",
            "\
DesignFile
  DesignUnit
    ContextClause
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
    ContextClause
    EntityDeclaration
      Keyword(Entity)
      Identifier 'my_ent2'
      Keyword(Is)
      EntityHeader
      Keyword(Begin)
      Keyword(End)
      Keyword(Entity)
      SemiColon
",
        );
    }

    #[test]
    fn parse_entity_with_context_clause() {
        check(
            Parser::design_file,
            "\
            library ieee;
            use ieee.std_logic_1164.all;

            entity my_ent is
            begin
            end my_ent;
        ",
            "\
DesignFile
  DesignUnit
    ContextClause
      LibraryClause
        Keyword(Library)
        IdentifierList
          Identifier 'ieee'
        SemiColon
      UseClause
        Keyword(Use)
        NameList
          Name
            Identifier 'ieee'
            SelectedName
              Dot
              Identifier 'std_logic_1164'
            SelectedName
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
",
        );
    }

    #[test]
    fn parse_use_clause() {
        check(
            Parser::use_clause,
            "use lib1.lib2.lib3.all;",
            "\
UseClause
  Keyword(Use)
  NameList
    Name
      Identifier 'lib1'
      SelectedName
        Dot
        Identifier 'lib2'
      SelectedName
        Dot
        Identifier 'lib3'
      SelectedName
        Dot
        Keyword(All)
  SemiColon
",
        );
    }
}
