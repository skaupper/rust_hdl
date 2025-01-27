//! Parsing of entity declarations
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2024, Lukas Scheller lukasscheller@icloud.com
/// Parsing of subprogram related rules (LRM §4.2-§4.4)
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::token_kind::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn subprogram_declaration(&mut self) {
        self.start_node(SubprogramDeclaration);
        self.subprogram_specification();
        self.expect_token(SemiColon);
        self.end_node();
    }

    pub fn subprogram_instantiation_declaration(&mut self) {
        self.start_node(SubprogramInstantiationDeclaration);
        self.expect_one_of_tokens([Keyword(Kw::Procedure), Keyword(Kw::Function)]);
        self.designator();
        self.expect_kw(Kw::Is);
        self.expect_kw(Kw::New);
        self.name();
        if self.next_is(LeftSquare) {
            self.signature();
        }
        self.opt_generic_map_aspect();
        self.expect_token(SemiColon);
        self.end_node();
    }

    fn subprogram_specification(&mut self) {
        self.start_node(SubprogramSpecification);
        if self.next_is(Keyword(Kw::Procedure)) {
            self.procedure_specification();
        } else {
            self.function_specification();
        }
        self.end_node();
    }

    fn procedure_specification(&mut self) {
        self.start_node(ProcedureSpecification);
        self.expect_kw(Kw::Procedure);
        self.designator();
        self.subprogram_header();

        let params_kw = self.opt_token(Keyword(Kw::Parameter));
        let params_list = self.next_is(LeftPar);
        if params_kw && !params_list {
            // TODO: That's a syntax error. Currently there is no way to handle that
            return;
        }

        if params_list {
            self.expect_token(LeftPar);
            self.interface_list();
            self.expect_token(RightPar);
        }
        self.end_node();
    }

    fn function_specification(&mut self) {
        self.start_node(FunctionSpecification);
        self.opt_tokens([Keyword(Kw::Pure), Keyword(Kw::Impure)]);
        self.expect_kw(Kw::Function);
        self.designator();
        self.subprogram_header();

        let params_kw = self.opt_token(Keyword(Kw::Parameter));
        let params_list = self.next_is(LeftPar);
        if params_kw && !params_list {
            // TODO: That's a syntax error. Currently there is no way to handle that
            return;
        }

        if params_list {
            self.expect_token(LeftPar);
            self.interface_list();
            self.expect_token(RightPar);
        }

        self.expect_kw(Kw::Return);
        self.name();
        self.end_node();
    }

    fn subprogram_header(&mut self) {
        self.start_node(SubprogramHeader);
        if self.opt_token(Keyword(Kw::Generic)) {
            self.expect_token(LeftPar);
            self.interface_list();
            self.expect_token(RightPar);

            self.opt_generic_map_aspect();
        }
        self.end_node();
    }

    pub(crate) fn subprogram_body(&mut self) {
        self.start_node(SubprogramBody);
        self.subprogram_specification();
        self.expect_kw(Kw::Is);

        self.subprogram_declarative_part();

        self.expect_kw(Kw::Begin);

        self.subprogram_statement_part();

        self.expect_kw(Kw::End);
        self.opt_tokens([Keyword(Kw::Procedure), Keyword(Kw::Function)]);
        if !self.next_is(SemiColon) {
            self.designator();
        }
        self.expect_token(SemiColon);
        self.end_node();
    }

    fn subprogram_statement_part(&mut self) {
        self.start_node(SubprogramStatementPart);
        while self.opt_sequential_statement() {}
        self.end_node();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::check;
    use crate::parser::Parser;

    #[test]
    #[ignore]
    fn dummy() {
        check(Parser::subprogram_declaration, "", "");
    }
}
