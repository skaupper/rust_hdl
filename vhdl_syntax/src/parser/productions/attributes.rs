// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::match_next_token;
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn attribute_declaration(&mut self) {
        self.start_node(AttributeDeclaration);
        self.expect_kw(Kw::Attribute);
        self.identifier();
        self.expect_token(Colon);
        self.name();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn attribute_specification(&mut self) {
        self.start_node(AttributeSpecification);
        self.expect_kw(Kw::Attribute);
        self.identifier();
        self.expect_token(Keyword(Kw::Of));
        self.entity_specification();
        self.expect_token(Keyword(Kw::Is));
        self.expression();
        self.expect_token(SemiColon);
        self.end_node();
    }

    fn entity_specification(&mut self) {
        self.start_node(EntitySpecification);
        self.entity_name_list();
        self.expect_token(Colon);
        self.entity_class();
        self.end_node();
    }

    pub(crate) fn entity_class(&mut self) {
        self.expect_one_of_tokens([
            Keyword(Kw::Entity),
            Keyword(Kw::Architecture),
            Keyword(Kw::Configuration),
            Keyword(Kw::Procedure),
            Keyword(Kw::Function),
            Keyword(Kw::Package),
            Keyword(Kw::Type),
            Keyword(Kw::Subtype),
            Keyword(Kw::Constant),
            Keyword(Kw::Signal),
            Keyword(Kw::Variable),
            Keyword(Kw::Component),
            Keyword(Kw::Label),
            Keyword(Kw::Literal),
            Keyword(Kw::Units),
            Keyword(Kw::Group),
            Keyword(Kw::File),
            Keyword(Kw::Property),
            Keyword(Kw::Sequence),
        ])
    }

    fn entity_name_list(&mut self) {
        self.start_node(EntityNameList);
        match_next_token!(self,
            Keyword(Kw::All) => self.skip(),
            Keyword(Kw::Others) => self.skip(),
            Identifier, StringLiteral, CharacterLiteral => {
                self.start_node(DesignatorList);
                self.separated_list(Parser::entity_designator, Comma);
                self.end_node();
            }
        );
        self.end_node();
    }

    fn entity_designator(&mut self) {
        self.start_node(EntityDesignator);
        self.entity_tag();
        if self.peek_token() == Some(LeftSquare) {
            self.signature();
        }
        self.end_node();
    }

    fn entity_tag(&mut self) {
        self.expect_one_of_tokens([Identifier, CharacterLiteral, StringLiteral])
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{test_utils::check, Parser};

    #[test]
    fn attribute_declaration() {
        check(
            Parser::attribute_declaration,
            "attribute mark_debug : string;",
            "\
AttributeDeclaration
  Keyword(Attribute)
  Identifier 'mark_debug'
  Colon
  Name
    Identifier 'string'
  SemiColon
",
        );

        check(
            Parser::attribute_declaration,
            "attribute my_attribute : work.my_pkg.my_attribute_type;",
            "\
AttributeDeclaration
  Keyword(Attribute)
  Identifier 'my_attribute'
  Colon
  Name
    Identifier 'work'
    SelectedName
      Dot
      Identifier 'my_pkg'
    SelectedName
      Dot
      Identifier 'my_attribute_type'
  SemiColon
",
        );
    }

    #[test]
    fn attribute_specification() {
        check(
            Parser::attribute_specification,
            "attribute mark_debug of s1, s2, s3: signal is \"TRUE\";",
            "\
AttributeSpecification
  Keyword(Attribute)
  Identifier 'mark_debug'
  Keyword(Of)
  EntitySpecification
    EntityNameList
      DesignatorList
        EntityDesignator
          Identifier 's1'
        Comma
        EntityDesignator
          Identifier 's2'
        Comma
        EntityDesignator
          Identifier 's3'
    Colon
    Keyword(Signal)
  Keyword(Is)
  Expression
    SimpleExpression
      StringLiteral '\"TRUE\"'
  SemiColon
",
        );

        check(
            Parser::attribute_specification,
            "attribute my_attr of all: label is value;",
            "\
AttributeSpecification
  Keyword(Attribute)
  Identifier 'my_attr'
  Keyword(Of)
  EntitySpecification
    EntityNameList
      Keyword(All)
    Colon
    Keyword(Label)
  Keyword(Is)
  Expression
    SimpleExpression
      Identifier 'value'
  SemiColon
",
        );

        check(
            Parser::attribute_specification,
            "attribute keep of others: architecture is value_false;",
            "\
AttributeSpecification
  Keyword(Attribute)
  Identifier 'keep'
  Keyword(Of)
  EntitySpecification
    EntityNameList
      Keyword(Others)
    Colon
    Keyword(Architecture)
  Keyword(Is)
  Expression
    SimpleExpression
      Identifier 'value_false'
  SemiColon
",
        );
    }
}
