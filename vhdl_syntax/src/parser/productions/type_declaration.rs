// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com
/// Parsing of type and subtype declarations (LRM §6.2)
use crate::parser::diagnostics::ParserDiagnostic;
use crate::parser::diagnostics::ParserError::*;
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn type_declaration(&mut self) {
        self.start_node(TypeDeclaration);
        self.expect_token(Keyword(Kw::Type));
        self.identifier();
        if self.opt_token(Keyword(Kw::Is)) {
            self.type_definition();
        }
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn subtype_declaration(&mut self) {
        self.start_node(TypeDeclaration);
        self.expect_token(Keyword(Kw::Type));
        self.identifier();
        self.expect_token(Keyword(Kw::Is));
        self.subtype_indication();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn type_definition(&mut self) {
        let max_length = match self.lookahead([SemiColon]) {
            Ok((_, len)) => len,
            Err(_) => {
                // TODO: The type definition is not fully parseable, what to do now?
                self.diagnostics.push(ParserDiagnostic::new(
                    self.builder.current_pos(),
                    LookaheadFailed(SemiColon),
                ));
                return;
            }
        };

        self.start_node(TypeDefinition);
        match_next_token!(self,
            Keyword(Kw::Record),Keyword(Kw::Array) => self.composite_type_definition(),
            LeftPar,Keyword(Kw::Range) => self.scalar_type_definition(max_length),
            Keyword(Kw::Access) => self.access_type_definition(),
            Keyword(Kw::File) => self.file_type_definition(),
            Keyword(Kw::Protected) => self.protected_type_definition()
        );
        self.end_node();
    }

    pub fn access_type_definition(&mut self) {
        self.start_node(AccessTypeDefinition);
        self.expect_kw(Kw::Access);
        self.subtype_indication();
        self.end_node();
    }

    pub fn file_type_definition(&mut self) {
        self.start_node(FileTypeDefinition);
        self.expect_kw(Kw::File);
        self.expect_kw(Kw::Of);
        self.name();
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::check;
    use crate::parser::Parser;

    #[test]
    fn incomplete_type_declaration() {
        check(
            Parser::type_declaration,
            "type incomplete_type;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'incomplete_type'
  SemiColon
",
        );
    }

    #[test]
    fn file_type_declaration() {
        check(
            Parser::type_declaration,
            "type IntegerFile is file of integer;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'IntegerFile'
  Keyword(Is)
  TypeDefinition
    FileTypeDefinition
      Keyword(File)
      Keyword(Of)
      Name
        Identifier 'integer'
  SemiColon
",
        );

        check(
            Parser::type_declaration,
            "type sl_file is file of ieee.std_logic_1164.std_ulogic;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'sl_file'
  Keyword(Is)
  TypeDefinition
    FileTypeDefinition
      Keyword(File)
      Keyword(Of)
      Name
        Identifier 'ieee'
        SelectedName
          Dot
          Identifier 'std_logic_1164'
        SelectedName
          Dot
          Identifier 'std_ulogic'
  SemiColon
",
        );
    }

    #[test]
    fn access_type_definition() {
        check(
            Parser::type_declaration,
            "type str_ptr_t is access string;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'str_ptr_t'
  Keyword(Is)
  TypeDefinition
    AccessTypeDefinition
      Keyword(Access)
      Identifier 'string'
  SemiColon
",
        );
    }
}
