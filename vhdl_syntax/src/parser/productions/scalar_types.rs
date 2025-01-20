// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2024, Lukas Scheller lukasscheller@icloud.com
/// Parsing of scalar types
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::{Keyword as Kw, TokenKind::*, TokenStream};

impl<T: TokenStream> Parser<T> {
    pub fn range(&mut self, max_length: usize) {
        // LRM §5.2.1

        // `max_length` should give the distance between the current token position and the end of the range to parse.
        // This way the parser can use a bounded lookahead to distinguish between range expressions (using `to` or `downto`) and attribute names.
        self.start_node(Range);

        let is_range_expression = self
            .lookahead_max_distance(max_length, [Keyword(Kw::To), Keyword(Kw::Downto)])
            .is_some();

        if is_range_expression {
            self.simple_expression();
            self.expect_one_of_tokens([Keyword(Kw::To), Keyword(Kw::Downto)]);
            self.simple_expression();
        } else {
            self.name();
        }

        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{test_utils::check, Parser};

    #[test]
    fn parse_range() {
        check(
            |parser| Parser::range(parser, usize::MAX),
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
            |parser| Parser::range(parser, usize::MAX),
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
            |parser| Parser::range(parser, usize::MAX),
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
