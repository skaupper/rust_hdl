// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn opt_generic_clause(&mut self) -> bool {
        if self.next_is(Keyword(Kw::Generic)) {
            self.generic_clause();
            true
        } else {
            false
        }
    }

    pub fn generic_clause(&mut self) {
        self.start_node(GenericClause);
        self.expect_kw(Kw::Generic);
        self.expect_token(LeftPar);
        self.interface_list();
        self.expect_tokens([RightPar, SemiColon]);
        self.end_node();
    }

    pub fn opt_generic_map_aspect(&mut self) -> bool {
        if !self.next_is(Keyword(Kw::Generic)) || self.next_nth_is(Keyword(Kw::Map), 1) {
            return false;
        }
        self.generic_map_aspect();
        true
    }

    pub fn generic_map_aspect(&mut self) {
        self.start_node(GenericMapAspect);
        self.expect_tokens([Keyword(Kw::Generic), Keyword(Kw::Map), LeftPar]);
        self.association_list();
        self.expect_token(RightPar);
        self.end_node();
    }

    pub fn opt_port_clause(&mut self) {
        if self.next_is(Keyword(Kw::Port)) {
            self.port_clause();
        }
    }

    pub fn port_clause(&mut self) {
        self.start_node(PortClause);
        self.expect_kw(Kw::Port);
        self.expect_token(LeftPar);
        self.interface_list();
        self.expect_tokens([RightPar, SemiColon]);
        self.end_node();
    }

    pub fn interface_list(&mut self) {
        self.start_node(InterfaceList);
        self.separated_list(Parser::interface_declaration, SemiColon);
        self.end_node();
    }

    pub fn interface_declaration(&mut self) {
        match self.peek_token() {
            Some(Keyword(Kw::Signal | Kw::Constant | Kw::Variable) | Identifier) => {
                self.interface_object_declaration();
            }
            Some(Keyword(Kw::File)) => {}
            Some(Keyword(Kw::Type)) => {}
            Some(Keyword(Kw::Function | Kw::Procedure | Kw::Impure | Kw::Pure)) => {}
            Some(Keyword(Kw::Package)) => {}
            _ => {}
        }
    }

    pub fn interface_object_declaration(&mut self) {
        self.start_node(InterfaceObjectDeclaration);
        self.opt_tokens([
            Keyword(Kw::Signal),
            Keyword(Kw::Constant),
            Keyword(Kw::Variable),
        ]);
        self.identifier_list();
        self.expect_token(Colon);
        self.opt_mode();
        self.subtype_indication();
        self.opt_token(Keyword(Kw::Bus));
        if self.opt_token(ColonEq) {
            self.expression();
        }
        self.end_node();
    }

    pub fn opt_mode(&mut self) {
        self.opt_tokens([
            Keyword(Kw::In),
            Keyword(Kw::Out),
            Keyword(Kw::Inout),
            Keyword(Kw::Buffer),
            Keyword(Kw::Linkage),
        ]);
    }

    pub fn association_list(&mut self) {
        self.association_list_bounded(usize::MAX);
    }
    fn association_list_bounded(&mut self, max_index: usize) {
        self.start_node(AssociationList);
        self.separated_list(
            |parser| {
                let end_of_element_idx =
                    match parser.lookahead_max_token_index(max_index, [Comma, RightPar]) {
                        Ok((_, idx)) => idx,
                        Err(idx) => idx,
                    };
                parser.association_element_bounded(end_of_element_idx);
            },
            Comma,
        );
        self.end_node();
    }

    fn association_element_bounded(&mut self, max_index: usize) {
        self.start_node(AssociationElement);

        // TODO: Error handling is done at a bare minimum.
        if let Ok(_) = self.lookahead_max_token_index(max_index, [RightArrow]) {
            self.formal_part();
            self.expect_token(RightArrow);
        }
        self.actual_part_bounded(max_index);

        self.end_node();
    }

    pub fn formal_part(&mut self) {
        self.start_node(FormalPart);
        self.name();
        // Note: `self.name()` will already consume any trailing parenthesized names!
        self.end_node();
    }

    fn actual_part_bounded(&mut self, max_index: usize) {
        self.start_node(ActualPart);
        // Parsing of `actual_part` would boil down to `name | expression | subtype_indication`
        self.start_node(RawTokens);
        self.skip_to(max_index);
        self.end_node();
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::check;
    use crate::parser::Parser;

    #[test]
    fn association_list() {
        check(
            Parser::association_list,
            "arg1, arg2",
            "\
AssociationList
  AssociationElement
    ActualPart
      RawTokens
        Identifier 'arg1'
  Comma
  AssociationElement
    ActualPart
      RawTokens
        Identifier 'arg2'
",
        );

        check(
            Parser::association_list,
            "p1 => 1, std_ulogic(p2)=>     sl_sig",
            "\
AssociationList
  AssociationElement
    FormalPart
      Name
        Identifier 'p1'
    RightArrow
    ActualPart
      RawTokens
        AbstractLiteral
  Comma
  AssociationElement
    FormalPart
      Name
        Identifier 'std_ulogic'
        RawTokens
          LeftPar
          Identifier 'p2'
          RightPar
    RightArrow
    ActualPart
      RawTokens
        Identifier 'sl_sig'
",
        );
    }

    #[test]
    fn empty_generic_clause() {
        check(
            Parser::generic_clause,
            "generic();",
            "\
GenericClause
  Keyword(Generic)
  LeftPar
  InterfaceList
  RightPar
  SemiColon
",
        );
    }

    #[test]
    fn empty_port_clause() {
        check(
            Parser::port_clause,
            "port();",
            "\
PortClause
  Keyword(Port)
  LeftPar
  InterfaceList
  RightPar
  SemiColon
",
        );
    }

    #[test]
    fn object_declaration() {
        check(
            Parser::interface_declaration,
            "a : in std_logic",
            "\
InterfaceObjectDeclaration
  IdentifierList
    Identifier 'a'
  Colon
  Keyword(In)
  Identifier 'std_logic'
",
        );
        check(
            Parser::interface_declaration,
            "a : out std_logic",
            "\
InterfaceObjectDeclaration
  IdentifierList
    Identifier 'a'
  Colon
  Keyword(Out)
  Identifier 'std_logic'
",
        );
        check(
            Parser::interface_declaration,
            "signal a : out std_logic",
            "\
InterfaceObjectDeclaration
  Keyword(Signal)
  IdentifierList
    Identifier 'a'
  Colon
  Keyword(Out)
  Identifier 'std_logic'
",
        );
        check(
            Parser::interface_declaration,
            "constant a : out std_logic",
            "\
InterfaceObjectDeclaration
  Keyword(Constant)
  IdentifierList
    Identifier 'a'
  Colon
  Keyword(Out)
  Identifier 'std_logic'
",
        );
        check(
            Parser::interface_declaration,
            "a : inout std_logic",
            "\
InterfaceObjectDeclaration
  IdentifierList
    Identifier 'a'
  Colon
  Keyword(Inout)
  Identifier 'std_logic'
",
        );
        check(
            Parser::interface_declaration,
            "a : linkage std_logic",
            "\
InterfaceObjectDeclaration
  IdentifierList
    Identifier 'a'
  Colon
  Keyword(Linkage)
  Identifier 'std_logic'
",
        );
        check(
            Parser::interface_declaration,
            "a : buffer std_logic",
            "\
InterfaceObjectDeclaration
  IdentifierList
    Identifier 'a'
  Colon
  Keyword(Buffer)
  Identifier 'std_logic'
",
        );
        check(
            Parser::interface_declaration,
            "a, b, c : in std_logic",
            "\
InterfaceObjectDeclaration
  IdentifierList
    Identifier 'a'
    Comma
    Identifier 'b'
    Comma
    Identifier 'c'
  Colon
  Keyword(In)
  Identifier 'std_logic'
",
        );
    }
}
