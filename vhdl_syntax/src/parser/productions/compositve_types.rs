// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com
/// Parsing of composite types (LRM §5.3)
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn composite_type_definition(&mut self) {
        match_next_token!(self,
            Keyword(Kw::Record) => self.record_type_definition(),
            Keyword(Kw::Array) => self.array_type_definition()
        );
    }

    pub fn array_type_definition(&mut self) {
        self.start_node(ArrayTypeDefinition);
        self.expect_kw(Kw::Array);
        let box_found = self.lookahead_skip_n(1, [BOX]).is_ok();

        if box_found {
            self.expect_token(LeftPar);
            self.separated_list(Parser::index_subtype_definition, Comma);
            self.expect_token(RightPar);
        } else {
            self.index_constraint();
        }
        self.expect_kw(Kw::Of);
        self.subtype_indication();
        self.end_node();
    }

    pub fn record_type_definition(&mut self) {
        self.start_node(RecordTypeDefinition);
        self.expect_kw(Kw::Record);

        while !self.next_is(Keyword(Kw::End)) {
            self.element_declaration();
        }

        self.expect_kw(Kw::End);
        self.expect_kw(Kw::Record);
        self.opt_identifier();
        self.end_node();
    }

    pub fn element_declaration(&mut self) {
        self.start_node(ElementDeclaration);
        self.identifier_list();
        self.expect_token(Colon);
        self.subtype_indication();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn index_subtype_definition(&mut self) {
        self.start_node(IndexSubtypeDefinition);
        self.name();
        self.expect_kw(Kw::Range);
        self.expect_token(BOX);
        self.end_node();
    }

    pub fn index_constraint(&mut self) {
        self.start_node(IndexConstraint);
        self.expect_token(LeftPar);
        self.separated_list(Parser::discrete_range, Comma);
        self.expect_token(RightPar);
        self.end_node();
    }

    pub fn discrete_range(&mut self) {
        let possible_follow_tokens = [
            Comma,
            RightPar,
            Keyword(Kw::Generate),
            RightArrow,
            Keyword(Kw::Loop),
            SemiColon,
            Bar,
        ];
        self.start_node(DiscreteRange);
        let max_index = match self.lookahead(possible_follow_tokens) {
            Ok((_, idx)) => idx,
            Err(_) => {
                self.eof_err();
                return;
            }
        };

        // TODO: That's not exactly true. A `range` may also consist of a single `attribute_name`
        let is_range = self
            .lookahead_max_token_index(max_index, [Keyword(Kw::To), Keyword(Kw::Downto)])
            .is_ok();

        if is_range {
            self.range_bounded(max_index);
        } else {
            self.start_node(RawTokens);
            self.skip_to(max_index);
            self.end_node();
        }
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::check;
    use crate::parser::Parser;

    #[test]
    fn array_type_declaration() {
        check(
            Parser::type_declaration,
            "type int_arr_t is array (natural range <>) of integer;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'int_arr_t'
  Keyword(Is)
  TypeDefinition
    ArrayTypeDefinition
      Keyword(Array)
      LeftPar
      IndexSubtypeDefinition
        Name
          Identifier 'natural'
        Keyword(Range)
        BOX
      RightPar
      Keyword(Of)
      Identifier 'integer'
  SemiColon
",
        );

        check(
            Parser::type_declaration,
            "type int_arr_2d_t is array (natural range <>, integer range <>) of positive;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'int_arr_2d_t'
  Keyword(Is)
  TypeDefinition
    ArrayTypeDefinition
      Keyword(Array)
      LeftPar
      IndexSubtypeDefinition
        Name
          Identifier 'natural'
        Keyword(Range)
        BOX
      Comma
      IndexSubtypeDefinition
        Name
          Identifier 'integer'
        Keyword(Range)
        BOX
      RightPar
      Keyword(Of)
      Identifier 'positive'
  SemiColon
",
        );
    }

    #[test]
    fn array_type_declaration_with_discrete_range() {
        check(
            Parser::type_declaration,
            "type constrained_int_arr is array (0 to 1) of positive;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'constrained_int_arr'
  Keyword(Is)
  TypeDefinition
    ArrayTypeDefinition
      Keyword(Array)
      IndexConstraint
        LeftPar
        DiscreteRange
          Range
            SimpleExpression
              AbstractLiteral
            Keyword(To)
            SimpleExpression
              AbstractLiteral
        RightPar
      Keyword(Of)
      Identifier 'positive'
  SemiColon
",
        );

        check(
            Parser::type_declaration,
            "type constrained_int_arr_2d is array (10 downto 5, 'A' to 'B', enum_t'range) of bit;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'constrained_int_arr_2d'
  Keyword(Is)
  TypeDefinition
    ArrayTypeDefinition
      Keyword(Array)
      IndexConstraint
        LeftPar
        DiscreteRange
          Range
            SimpleExpression
              AbstractLiteral
            Keyword(Downto)
            SimpleExpression
              AbstractLiteral
        Comma
        DiscreteRange
          Range
            SimpleExpression
              CharacterLiteral ''A''
            Keyword(To)
            SimpleExpression
              CharacterLiteral ''B''
        Comma
        DiscreteRange
          RawTokens
            Identifier 'enum_t'
            Tick
            Keyword(Range)
        RightPar
      Keyword(Of)
      Identifier 'bit'
  SemiColon
",
        );
    }

    #[test]
    fn record_type_declaration() {
        check(
            Parser::type_declaration,
            "type rec_t is record state: enum_t; end record;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'rec_t'
  Keyword(Is)
  TypeDefinition
    RecordTypeDefinition
      Keyword(Record)
      ElementDeclaration
        IdentifierList
          Identifier 'state'
        Colon
        Identifier 'enum_t'
        SemiColon
      Keyword(End)
      Keyword(Record)
  SemiColon
",
        );

        check(
            Parser::type_declaration,
            "type rec_t is record s1: bit; s2, s3: std_ulogic; end record;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'rec_t'
  Keyword(Is)
  TypeDefinition
    RecordTypeDefinition
      Keyword(Record)
      ElementDeclaration
        IdentifierList
          Identifier 's1'
        Colon
        Identifier 'bit'
        SemiColon
      ElementDeclaration
        IdentifierList
          Identifier 's2'
          Comma
          Identifier 's3'
        Colon
        Identifier 'std_ulogic'
        SemiColon
      Keyword(End)
      Keyword(Record)
  SemiColon
",
        );
    }
}
