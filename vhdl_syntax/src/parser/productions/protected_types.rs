// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com
/// Parsing of scalar types (LRM §5.6)
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn protected_type_definition(&mut self) {
        self.start_node(ProtectedTypeDefinition);
        match self.peek_nth_token(1) {
            Some(Keyword(Kw::Body)) => self.protected_type_body(),
            Some(_) => self.protected_type_declaration(),
            None => self.eof_err(),
        }
        self.end_node();
    }

    pub fn protected_type_declaration(&mut self) {
        self.start_node(ProtectedTypeDeclaration);
        self.expect_kw(Kw::Protected);

        self.protected_type_declarative_part();

        self.expect_kw(Kw::End);
        self.expect_kw(Kw::Protected);
        self.opt_identifier();
        self.end_node();
    }

    pub fn protected_type_body(&mut self) {
        self.start_node(ProtectedTypeBody);
        self.expect_kw(Kw::Protected);
        self.expect_kw(Kw::Body);

        self.protected_type_body_declarative_part();

        self.expect_kw(Kw::End);
        self.expect_kw(Kw::Protected);
        self.expect_kw(Kw::Body);
        self.opt_identifier();
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::check;
    use crate::parser::Parser;

    #[test]
    fn protected_type_declaration() {
        check(
            Parser::type_declaration,
            "type p_t is protected end protected;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'p_t'
  Keyword(Is)
  TypeDefinition
    ProtectedTypeDefinition
      ProtectedTypeDeclaration
        Keyword(Protected)
        ProtectedTypeDeclarativePart
        Keyword(End)
        Keyword(Protected)
  SemiColon
",
        );

        check(
            Parser::type_declaration,
            "type p_t is protected end protected p_t;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'p_t'
  Keyword(Is)
  TypeDefinition
    ProtectedTypeDefinition
      ProtectedTypeDeclaration
        Keyword(Protected)
        ProtectedTypeDeclarativePart
        Keyword(End)
        Keyword(Protected)
        Identifier 'p_t'
  SemiColon
",
        );

        // TODO: Test protected types with content
    }

    #[test]
    fn protected_type_body() {
        check(
            Parser::type_declaration,
            "type p_t is protected body end protected body;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'p_t'
  Keyword(Is)
  TypeDefinition
    ProtectedTypeDefinition
      ProtectedTypeBody
        Keyword(Protected)
        Keyword(Body)
        ProtectedTypeBodyDeclarativePart
        Keyword(End)
        Keyword(Protected)
        Keyword(Body)
  SemiColon
",
        );
        check(
            Parser::type_declaration,
            "type p_t is protected body end protected body p_t;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'p_t'
  Keyword(Is)
  TypeDefinition
    ProtectedTypeDefinition
      ProtectedTypeBody
        Keyword(Protected)
        Keyword(Body)
        ProtectedTypeBodyDeclarativePart
        Keyword(End)
        Keyword(Protected)
        Keyword(Body)
        Identifier 'p_t'
  SemiColon
",
        );

        // TODO: Test protected types with content
    }
}
