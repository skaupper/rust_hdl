// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::Parser;
use crate::syntax::node_kind::InternalNodeKind;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn opt_generic_clause(&mut self) {
        if self.next_is(Keyword(Kw::Generic)) {
            self.generic_clause();
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
        self.start_node(AssociationList);
        self.separated_list(Parser::association_element, Comma);
        self.end_node();
    }

    pub fn association_element(&mut self) {
        self.start_node(AssociationElement);

        let right_arrow_idx = match self.distance_to_closing_paren_or_token(Comma) {
            Some(length) => self.lookahead_max_distance(length, [RightArrow]),
            None => {
                self.eof_err();
                return;
            }
        };

        if right_arrow_idx.is_some() {
            self.formal_part();
            self.expect_token(RightArrow);
        }
        self.actual_part();

        self.end_node();
    }

    pub fn formal_part(&mut self) {
        self.start_node(FormalPart);
        self.name();
        // Note: `self.name()` will already consume any trailing parenthesized names!
        self.end_node();
    }

    pub fn actual_part(&mut self) {
        self.start_node(ActualPart);
        let length = match self.distance_to_closing_paren_or_token(Comma) {
            Some(distance) => distance,
            None => {
                self.eof_err();
                return;
            }
        };

        // TODO: Parsing of `actual_part` would boil down to `name | expression | subtype_indication`
        self.start_node(Internal(InternalNodeKind::ActualPartTokens));
        self.skip_n(length);
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
        // Make sure the association list is followed by a closing parenthesis, otherwise parsing will fail
        // In reality that shouldn't be a problem, since association lists are always to be enclosed in parenthesis!
        check(
            Parser::association_list,
            "arg1, arg2)",
            "\
AssociationList
  AssociationElement
    ActualPart
      Internal(ActualPartTokens)
        Identifier 'arg1'
  Comma
  AssociationElement
    ActualPart
      Internal(ActualPartTokens)
        Identifier 'arg2'
",
        );

        check(
            Parser::association_list,
            "p1 => 1, std_ulogic(p2)=>     sl_sig)",
            "\
AssociationList
  AssociationElement
    FormalPart
      Name
        Identifier 'p1'
    RightArrow
    ActualPart
      Internal(ActualPartTokens)
        AbstractLiteral
  Comma
  AssociationElement
    FormalPart
      Name
        Identifier 'std_ulogic'
        Internal(SubtypeIndicationOrExpressionTokens)
          LeftPar
          Identifier 'p2'
          RightPar
    RightArrow
    ActualPart
      Internal(ActualPartTokens)
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
