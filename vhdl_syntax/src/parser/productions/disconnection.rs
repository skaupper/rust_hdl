// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com
/// Parsing of disconnection specifications (LRM §7.4)
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::token_kind::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub(crate) fn disconnection_specification(&mut self) {
        self.start_node(DisconnectionSpecification);
        self.expect_kw(Kw::Disconnect);
        self.guarded_signal_specification();
        self.expect_kw(Kw::After);
        self.expression();
        self.expect_token(SemiColon);
        self.end_node();
    }

    fn guarded_signal_specification(&mut self) {
        self.start_node(GuardedSignalSpecification);
        self.signal_list();
        self.expect_token(Colon);
        self.name();
        self.end_node();
    }

    fn signal_list(&mut self) {
        self.start_node(SignalList);
        match self.peek_token() {
            Some(Keyword(Kw::Others | Kw::All)) => self.skip(),
            Some(_) => self.separated_list(Parser::name, Comma),
            _ => {
                // TODO: Proper error handling
                self.eof_err();
            }
        }
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::*;
    use crate::parser::Parser;

    #[test]
    fn disconnection_specification() {
        check(
            Parser::disconnection_specification,
            "disconnect s1: bit after TIME_CONST;",
            "\
DisconnectionSpecification
  Keyword(Disconnect)
  GuardedSignalSpecification
    SignalList
      Name
        Identifier 's1'
    Colon
    Name
      Identifier 'bit'
  Keyword(After)
  Expression
    SimpleExpression
      Identifier 'TIME_CONST'
  SemiColon
",
        );

        check(
            Parser::disconnection_specification,
            "disconnect s1,s2,s3,s4: work.std_logic_1164.std_ulogic after 10;",
            "\
DisconnectionSpecification
  Keyword(Disconnect)
  GuardedSignalSpecification
    SignalList
      Name
        Identifier 's1'
      Comma
      Name
        Identifier 's2'
      Comma
      Name
        Identifier 's3'
      Comma
      Name
        Identifier 's4'
    Colon
    Name
      Identifier 'work'
      SelectedName
        Dot
        Identifier 'std_logic_1164'
      SelectedName
        Dot
        Identifier 'std_ulogic'
  Keyword(After)
  Expression
    SimpleExpression
      AbstractLiteral
  SemiColon
",
        );
    }
}
