// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com
/// Parsing of scalar types (LRM §5.2)
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn scalar_type_definition(&mut self, max_length: usize) {
        self.start_node(ScalarTypeDefinition);
        if self.next_is(LeftPar) {
            self.enumeration_type_definition();
        } else {
            self.range_constraint(max_length);

            if self.opt_token(Keyword(Kw::Units)) {
                self.primary_unit_declaration();
                while !self.next_is(Keyword(Kw::End)) {
                    self.secondary_unit_declaration();
                }
                self.expect_kw(Kw::End);
                self.expect_kw(Kw::Units);
                self.opt_identifier();
            }
        }
        self.end_node();
    }

    pub fn enumeration_type_definition(&mut self) {
        self.start_node(EnumerationTypeDefinition);
        self.expect_token(LeftPar);
        self.separated_list(
            |parser| parser.expect_one_of_tokens([Identifier, CharacterLiteral]),
            Comma,
        );
        self.expect_token(RightPar);
        self.end_node();
    }

    pub fn range_constraint(&mut self, max_index: usize) {
        self.start_node(RangeConstraint);
        self.expect_kw(Kw::Range);
        self.range_bounded(max_index);
        self.end_node();
    }

    pub fn range(&mut self) {
        self.range_bounded(usize::MAX);
    }
    fn range_bounded(&mut self, max_index: usize) {
        // LRM §5.2.1

        // `max_index` should point to the end of the range to parse (exclusive).
        // This way the parser can use a bounded lookahead to distinguish between range expressions (using `to` or `downto`) and attribute names.
        self.start_node(Range);

        let is_range_expression = self
            .lookahead_max_token_index(max_index, [Keyword(Kw::To), Keyword(Kw::Downto)])
            .is_ok();

        if is_range_expression {
            self.simple_expression();
            self.expect_one_of_tokens([Keyword(Kw::To), Keyword(Kw::Downto)]);
            self.simple_expression();
        } else {
            self.name();
        }

        self.end_node();
    }

    pub fn primary_unit_declaration(&mut self) {
        self.start_node(PrimaryUnitDeclaration);
        self.identifier();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn secondary_unit_declaration(&mut self) {
        self.start_node(SecondaryUnitDeclaration);
        self.identifier();
        self.expect_token(EQ);
        self.opt_token(AbstractLiteral);
        self.name();
        self.expect_token(SemiColon);
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::check;
    use crate::parser::Parser;

    #[test]
    fn integer_type_declaration() {
        check(
            Parser::type_declaration,
            "type positive_t is range 0 to C_MAX;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'positive_t'
  Keyword(Is)
  TypeDefinition
    ScalarTypeDefinition
      RangeConstraint
        Keyword(Range)
        Range
          SimpleExpression
            AbstractLiteral
          Keyword(To)
          SimpleExpression
            Identifier 'C_MAX'
  SemiColon
",
        );
    }

    #[test]
    fn floating_type_declaration() {
        check(
            Parser::type_declaration,
            "type some_float_t is range C_MAX downto 3.141592654;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'some_float_t'
  Keyword(Is)
  TypeDefinition
    ScalarTypeDefinition
      RangeConstraint
        Keyword(Range)
        Range
          SimpleExpression
            Identifier 'C_MAX'
          Keyword(Downto)
          SimpleExpression
            AbstractLiteral
  SemiColon
",
        );
    }

    #[test]
    fn physical_type_declaration() {
        check(
            Parser::type_declaration,
            "\
type dec_t is range 0 to 1e10 units
    prim;
    sec        = 2 prim;
    ter        = 3 prim;
    alias_prim =   prim;
end units;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'dec_t'
  Keyword(Is)
  TypeDefinition
    ScalarTypeDefinition
      RangeConstraint
        Keyword(Range)
        Range
          SimpleExpression
            AbstractLiteral
          Keyword(To)
          SimpleExpression
            AbstractLiteral
      Keyword(Units)
      PrimaryUnitDeclaration
        Identifier 'prim'
        SemiColon
      SecondaryUnitDeclaration
        Identifier 'sec'
        EQ
        AbstractLiteral
        Name
          Identifier 'prim'
        SemiColon
      SecondaryUnitDeclaration
        Identifier 'ter'
        EQ
        AbstractLiteral
        Name
          Identifier 'prim'
        SemiColon
      SecondaryUnitDeclaration
        Identifier 'alias_prim'
        EQ
        Name
          Identifier 'prim'
        SemiColon
      Keyword(End)
      Keyword(Units)
  SemiColon
",
        );

        check(
            Parser::type_declaration,
            "\
type distance_t is range 0 to 10 units
    m;
end units;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'distance_t'
  Keyword(Is)
  TypeDefinition
    ScalarTypeDefinition
      RangeConstraint
        Keyword(Range)
        Range
          SimpleExpression
            AbstractLiteral
          Keyword(To)
          SimpleExpression
            AbstractLiteral
      Keyword(Units)
      PrimaryUnitDeclaration
        Identifier 'm'
        SemiColon
      Keyword(End)
      Keyword(Units)
  SemiColon
",
        );
    }

    #[test]
    fn enumeration_type_declaration() {
        check(
            Parser::type_declaration,
            "type enum_t is (A);",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'enum_t'
  Keyword(Is)
  TypeDefinition
    ScalarTypeDefinition
      EnumerationTypeDefinition
        LeftPar
        Identifier 'A'
        RightPar
  SemiColon
",
        );

        check(
            Parser::type_declaration,
            "type enum_2_t is (S1, S2, S3);",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'enum_2_t'
  Keyword(Is)
  TypeDefinition
    ScalarTypeDefinition
      EnumerationTypeDefinition
        LeftPar
        Identifier 'S1'
        Comma
        Identifier 'S2'
        Comma
        Identifier 'S3'
        RightPar
  SemiColon
",
        );

        check(
            Parser::type_declaration,
            "type chars_t is ('A', 'B');",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'chars_t'
  Keyword(Is)
  TypeDefinition
    ScalarTypeDefinition
      EnumerationTypeDefinition
        LeftPar
        CharacterLiteral ''A''
        Comma
        CharacterLiteral ''B''
        RightPar
  SemiColon
",
        );
    }

    #[test]
    fn range_declaration() {
        check(
            Parser::type_declaration,
            "type positive_t is range 0 to C_MAX;",
            "\
TypeDeclaration
  Keyword(Type)
  Identifier 'positive_t'
  Keyword(Is)
  TypeDefinition
    ScalarTypeDefinition
      RangeConstraint
        Keyword(Range)
        Range
          SimpleExpression
            AbstractLiteral
          Keyword(To)
          SimpleExpression
            Identifier 'C_MAX'
  SemiColon
",
        );
    }

    #[test]
    fn range() {
        check(
            Parser::range,
            "100 downto 10",
            "\
Range
  SimpleExpression
    AbstractLiteral
  Keyword(Downto)
  SimpleExpression
    AbstractLiteral
",
        );

        check(
            Parser::range,
            "0 to 0",
            "\
Range
  SimpleExpression
    AbstractLiteral
  Keyword(To)
  SimpleExpression
    AbstractLiteral
",
        );

        check(
            Parser::range,
            "slv32_t'range",
            "\
Range
  Name
    Identifier 'slv32_t'
    AttributeName
      Tick
      Keyword(Range)
",
        );
    }
}
