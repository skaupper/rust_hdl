// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::util::LookaheadError;
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

fn is_start_of_attribute_name<T: TokenStream>(parser: &mut Parser<T>) -> bool {
    // Checking for `LeftSquare || Tick` will result in ambiguities with other grammar rules where a signature is possible right after a name.
    // Those rules can be `alias_declaration` (LRM §6.6.1) and `subprogram_instantiation_declaration` (LRM §4.4).
    // By checking whether the closing square bracket is followed by a `Tick` this ambiguity is resolved
    match parser.peek_token() {
        Some(Tick) => true,
        Some(LeftSquare) => {
            let mut idx = 1;
            let mut bracket_count = 1;

            while bracket_count > 0 {
                match parser.peek_nth_token(idx) {
                    Some(LeftSquare) => bracket_count += 1,
                    Some(RightSquare) => bracket_count -= 1,
                    Some(_) => {}
                    None => {
                        return false;
                    }
                }

                idx += 1;
            }

            parser.next_nth_is(Tick, idx)
        }
        Some(_) | None => false,
    }
}

impl<T: TokenStream> Parser<T> {
    pub fn name(&mut self) {
        self.name_bounded(usize::MAX);
    }

    pub(crate) fn name_bounded(&mut self, max_index: usize) {
        // (Based on) LRM §8.1
        // The LRM grammar rules for names were transformed to avoid left recursion.

        // In contrast to the LRM, this parsing routine is greedy. Meaning, it will consume trailing parenthesized
        // expressions even if the belong to an outer grammar rule!
        self.start_node(Name);

        if self.next_is(LtLt) {
            self.external_name();
        } else {
            self.expect_one_of_tokens([Identifier, StringLiteral, CharacterLiteral]);
        }

        self.opt_name_tail_bounded(max_index);
        self.end_node();
    }

    pub fn type_mark(&mut self) {
        self.name()
    }

    pub(crate) fn designator(&mut self) {
        // TODO: That designator is not fully LRM compliant
        self.expect_one_of_tokens([Identifier, StringLiteral, CharacterLiteral]);
    }

    pub(crate) fn opt_label(&mut self) {
        if self.next_is(Identifier) && self.next_nth_is(Colon, 1) {
            self.start_node(Label);
            self.skip_n(2);
            self.end_node();
        }
    }
    pub(crate) fn name_list(&mut self) {
        self.start_node(NameList);
        self.separated_list(Parser::name, Comma);
        self.end_node();
    }

    fn suffix(&mut self) {
        // LRM §8.3
        // suffix ::= identifier | string_literal | character_literal | `all` ;
        self.expect_one_of_tokens([
            Identifier,
            StringLiteral,
            CharacterLiteral,
            Keyword(Kw::All),
        ]);
    }

    fn opt_name_tail_bounded(&mut self, max_index: usize) -> bool {
        // name             ::= prefix [ name_tail ] ;
        // name_tail        ::= selected_name | attribute_name | indexed_name | slice_name | function_name ;
        // selected_name    ::= `.` suffix [ name_tail ] ;
        // attribute_name   ::= [ signature ] `'` identifier [ `(` expression `)` ] [ name_tail ] ;
        // function_name    ::= `(` association_list `)` [ name_tail ] ;
        // indexed_name     ::= `(` expression { `,` expression } `)` [ name_tail ] ;
        // slice_name       ::= `(` discrete_range `)` [ name_tail ] ;

        if self.next_is(Dot) {
            self.start_node(SelectedName);
            self.expect_token(Dot);
            self.suffix();
            self.end_node();
            self.opt_name_tail_bounded(max_index)
        } else if self.next_is(LeftPar) {
            // Instead of trying to differentiate between `subtype_indication`, `association_list`, a list of `expression`s and a `discrete_range`
            // put all tokens inside the parenthesis in a `RawTokens` node.
            let end_index_opt =
                match self.lookahead_max_token_index_skip_n(max_index, 1, [RightPar]) {
                    Ok((_, end_index)) => Some(end_index),
                    Err((LookaheadError::MaxIndexReached, _)) => None,
                    // Skip parsing of the parenthesized group, if EOF is reached
                    Err((LookaheadError::Eof, _)) => None,
                    // This error is only possible, when a `RightPar` is found before any token in `kinds`.
                    // Since `RightPar` is in `kinds` that's not possible!
                    Err((LookaheadError::TokenKindNotFound, _)) => unreachable!(),
                };

            if let Some(end_index) = end_index_opt {
                self.start_node(RawTokens);
                self.expect_token(LeftPar);
                self.skip_to(end_index);
                self.expect_token(RightPar);
                self.end_node();

                self.opt_name_tail_bounded(max_index)
            } else {
                false
            }
        } else if is_start_of_attribute_name(self) {
            self.start_node(AttributeName);
            if self.next_is(LeftSquare) {
                self.signature();
            }
            self.expect_token(Tick);

            // `range` is a keyword, but may appear as an `attribute_name`
            if !self.opt_identifier() {
                self.expect_kw(Kw::Range);
            }

            if self.next_is(LeftPar) {
                self.start_node(ParenthesizedExpression);
                self.expect_token(LeftPar);
                self.expression();
                self.expect_token(RightPar);
                self.end_node();
            }
            self.end_node();
            self.opt_name_tail_bounded(max_index)
        } else {
            false
        }
    }

    pub fn external_name(&mut self) {
        // LRM §8.7
        self.start_node(ExternalName);
        self.expect_token(LtLt);

        self.expect_one_of_tokens([
            Keyword(Kw::Constant),
            Keyword(Kw::Signal),
            Keyword(Kw::Variable),
        ]);
        self.external_pathname();
        self.expect_token(Colon);
        self.subtype_indication();

        self.expect_token(GtGt);
        self.end_node();
    }

    fn external_pathname(&mut self) {
        // LRM §8.7
        self.start_node(ExternalPathName);
        match_next_token!(self,
        CommAt => {
            self.expect_token(CommAt);
            self.identifier();
            self.expect_token(Dot);
            self.identifier();
            self.expect_token(Dot);
            self.identifier();
            while self.opt_token(Dot) {
                self.identifier();
            }
        },
        Dot => {
            self.expect_token(Dot);
            self.partial_pathname();
        },
        Circ, Identifier => {
            while self.opt_token(Circ) {
                self.expect_token(Dot);
            }
            self.partial_pathname();
        });
        self.end_node();
    }

    fn partial_pathname(&mut self) {
        // LRM §8.7
        // partial_pathname ::= { identifier [ `(` expression `)` ] `.` } identifier ;
        self.identifier();
        loop {
            if self.next_is(LeftPar) {
                self.start_node(ParenthesizedExpression);
                self.expect_token(LeftPar);
                self.expression();
                self.expect_token(RightPar);
                self.end_node();
                self.expect_token(Dot);
            } else if !self.opt_token(Dot) {
                break;
            }
            self.identifier();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{test_utils::check, Parser};

    #[test]
    fn parse_name() {
        check(
            Parser::name,
            "lib1.fn('a', 1, sig).vector(100 downto 10).all",
            "\
Name
  Identifier 'lib1'
  SelectedName
    Dot
    Identifier 'fn'
  RawTokens
    LeftPar
    CharacterLiteral ''a''
    Comma
    AbstractLiteral
    Comma
    Identifier 'sig'
    RightPar
  SelectedName
    Dot
    Identifier 'vector'
  RawTokens
    LeftPar
    AbstractLiteral
    Keyword(Downto)
    AbstractLiteral
    RightPar
  SelectedName
    Dot
    Keyword(All)
",
        );
    }

    #[test]
    fn parse_external_name() {
        check(
            Parser::name,
            "<< constant @lib.pkg.obj : std_ulogic >>",
            "\
Name
  ExternalName
    LtLt
    Keyword(Constant)
    ExternalPathName
      CommAt
      Identifier 'lib'
      Dot
      Identifier 'pkg'
      Dot
      Identifier 'obj'
    Colon
    Identifier 'std_ulogic'
    GtGt
",
        );

        check(
            Parser::name,
            "<< variable .tb.sig : bit >>",
            "\
Name
  ExternalName
    LtLt
    Keyword(Variable)
    ExternalPathName
      Dot
      Identifier 'tb'
      Dot
      Identifier 'sig'
    Colon
    Identifier 'bit'
    GtGt
",
        );

        check(
            Parser::name,
            "<< signal uut.sig : natural >>",
            "\
Name
  ExternalName
    LtLt
    Keyword(Signal)
    ExternalPathName
      Identifier 'uut'
      Dot
      Identifier 'sig'
    Colon
    Identifier 'natural'
    GtGt
",
        );

        check(
            Parser::name,
            "<< signal ^.up1_signal : real >>",
            "\
Name
  ExternalName
    LtLt
    Keyword(Signal)
    ExternalPathName
      Circ
      Dot
      Identifier 'up1_signal'
    Colon
    Identifier 'real'
    GtGt
",
        );

        check(
            Parser::name,
            "<<constant^.^.^.^.up4_signal:integer>>",
            "\
Name
  ExternalName
    LtLt
    Keyword(Constant)
    ExternalPathName
      Circ
      Dot
      Circ
      Dot
      Circ
      Dot
      Circ
      Dot
      Identifier 'up4_signal'
    Colon
    Identifier 'integer'
    GtGt
",
        );

        check(
            Parser::name,
            "<< constant .tb.uut.gen(1).sig : bit >>",
            "\
Name
  ExternalName
    LtLt
    Keyword(Constant)
    ExternalPathName
      Dot
      Identifier 'tb'
      Dot
      Identifier 'uut'
      Dot
      Identifier 'gen'
      ParenthesizedExpression
        LeftPar
        Expression
          SimpleExpression
            AbstractLiteral
        RightPar
      Dot
      Identifier 'sig'
    Colon
    Identifier 'bit'
    GtGt
",
        );
    }

    #[test]
    fn parse_selected_name() {
        check(
            Parser::name,
            "lib.pkg_outer.pkg_inner.obj",
            "\
Name
  Identifier 'lib'
  SelectedName
    Dot
    Identifier 'pkg_outer'
  SelectedName
    Dot
    Identifier 'pkg_inner'
  SelectedName
    Dot
    Identifier 'obj'
",
        );

        check(
            Parser::name,
            "pkg.all",
            "\
Name
  Identifier 'pkg'
  SelectedName
    Dot
    Keyword(All)
",
        );
    }

    #[test]
    fn parse_attribute_name() {
        check(
            Parser::name,
            "obj'left",
            "\
Name
  Identifier 'obj'
  AttributeName
    Tick
    Identifier 'left'
",
        );

        check(
            Parser::name,
            "slv'range",
            "\
Name
  Identifier 'slv'
  AttributeName
    Tick
    Keyword(Range)
",
        );

        check(
            Parser::name,
            "slv'reverse_range",
            "\
Name
  Identifier 'slv'
  AttributeName
    Tick
    Identifier 'reverse_range'
",
        );

        check(
            Parser::name,
            "integer'image(obj)",
            "\
Name
  Identifier 'integer'
  AttributeName
    Tick
    Identifier 'image'
    ParenthesizedExpression
      LeftPar
      Expression
        SimpleExpression
          Identifier 'obj'
      RightPar
",
        );

        check(
            Parser::name,
            "ieee.numeric_std.to_unsigned[natural, natural return unsigned]'simple_name",
            "\
Name
  Identifier 'ieee'
  SelectedName
    Dot
    Identifier 'numeric_std'
  SelectedName
    Dot
    Identifier 'to_unsigned'
  AttributeName
    Signature
      LeftSquare
      NameList
        Name
          Identifier 'natural'
        Comma
        Name
          Identifier 'natural'
      Keyword(Return)
      Name
        Identifier 'unsigned'
      RightSquare
    Tick
    Identifier 'simple_name'
",
        );
    }
}
