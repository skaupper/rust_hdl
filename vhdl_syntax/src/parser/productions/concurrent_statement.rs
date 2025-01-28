// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    fn block_statement_inner(&mut self) {
        self.expect_kw(Kw::Block);
        if self.opt_token(LeftPar) {
            self.condition();
            self.expect_token(RightPar);
        }
        self.opt_token(Keyword(Kw::Is));
        self.block_header();
        self.block_declarative_part();
        self.expect_kw(Kw::Begin);
        self.concurrent_statements();
        self.expect_tokens([Keyword(Kw::End), Keyword(Kw::Block)]);
        self.opt_identifier();
        self.expect_token(SemiColon);
    }

    pub fn block_header(&mut self) {
        self.start_node(BlockHeader);
        self.opt_generic_clause();
        if self.opt_generic_map_aspect() {
            self.expect_token(SemiColon)
        }
        self.opt_port_clause();
        if self.opt_port_map_aspect() {
            self.expect_token(SemiColon)
        }
        self.end_node();
    }

    pub fn block_declarative_part(&mut self) {
        self.declarative_part()
    }

    pub(crate) fn concurrent_statements(&mut self) {
        loop {
            match self.peek_token() {
                Some(Keyword(Kw::End | Kw::Elsif | Kw::Else | Kw::When)) | None => {
                    return;
                }
                _ => self.concurrent_statement(),
            }
        }
    }

    pub(crate) fn concurrent_statement(&mut self) {
        self.start_unknown_node();
        self.opt_label();
        match_next_token!(self,
            Keyword(Kw::Block) => {
                self.block_statement_inner();
                self.end_node_with_kind(BlockStatement);
            },
            Keyword(Kw::Process) => {
                self.process_statement_inner();
                self.end_node_with_kind(ProcessStatement);
            },
            Keyword(Kw::Component) => {
                self.skip();
                self.name();
                self.instantiation_statement_inner();
                self.end_node_with_kind(ComponentInstantiationStatement)
            },
            Keyword(Kw::Configuration) => {
                self.skip();
                self.name();
                self.instantiation_statement_inner();
                self.end_node_with_kind(ConfigurationInstantiationStatement)
            },
            Keyword(Kw::Entity) => {
                self.skip();
                self.name();
                if self.opt_token(LeftPar) {
                    self.identifier();
                    self.expect_token(RightPar);
                }
                self.instantiation_statement_inner();
                self.end_node_with_kind(EntityInstantiationStatement);
            },
            Keyword(Kw::For) => {
                self.for_generate_statement_inner();
                self.end_node_with_kind(ForGenerateStatement);
            },
            Keyword(Kw::If) => {
                self.if_generate_statement_inner();
                self.end_node_with_kind(IfGenerateStatement);
            },
            Keyword(Kw::Case) => {
                self.case_generate_statement_inner();
                self.end_node_with_kind(CaseGenerateStatement);
            },
            Keyword(Kw::Assert) => {
                self.concurrent_assert_statement_inner();
                self.end_node_with_kind(ConcurrentAssertionStatement);
            },
            Keyword(Kw::Postponed) => {
                match self.peek_nth_token(1) {
                    Some(Keyword(Kw::Process)) => {
                        self.process_statement_inner();
                        self.end_node_with_kind(ProcessStatement);
                    },
                    Some(Keyword(Kw::Assert)) => {
                        self.concurrent_assert_statement_inner();
                        self.end_node_with_kind(ConcurrentAssertionStatement);
                    }
                    Some(Keyword(Kw::With)) => {
                        self.concurrent_selected_signal_assignment_inner();
                        self.end_node_with_kind(ConcurrentSelectedSignalAssignmentStatement);
                    }
                    _ => todo!()
                }
            },
            Keyword(Kw::With) => {
                self.concurrent_selected_signal_assignment_inner();
                self.end_node_with_kind(ConcurrentSelectedSignalAssignmentStatement);
            }
            // _ => todo!()
        );
    }

    fn concurrent_selected_signal_assignment_inner(&mut self) {
        self.opt_token(Keyword(Kw::Postponed));
        self.expect_kw(Kw::With);
        self.expression();
        self.expect_kw(Kw::Select);
        self.opt_token(Que);
        self.target();
        self.expect_token(LTE);
        self.opt_token(Keyword(Kw::Guarded));
        self.opt_delay_mechanism();
        self.selected_waveforms();
        self.expect_token(SemiColon);
    }

    pub fn selected_waveforms(&mut self) {
        self.start_node(SelectedWaveforms);
        self.separated_list(Parser::selected_waveform, Comma);
        self.end_node();
    }

    fn selected_waveform(&mut self) {
        self.start_node(SelectedWaveform);
        self.waveform();
        self.expect_kw(Kw::When);
        self.choices();
        self.end_node();
    }

    pub fn waveform(&mut self) {
        self.start_node(Waveform);
        if self.opt_token(Keyword(Kw::Unaffected)) {
            self.end_node();
            return;
        }
        self.separated_list(Parser::waveform_element, Comma);
        self.end_node();
    }

    pub fn waveform_element(&mut self) {
        self.start_node(WaveformElement);
        self.expression();
        if self.opt_token(Keyword(Kw::After)) {
            self.expression();
        }
        self.end_node()
    }

    fn opt_delay_mechanism(&mut self) {
        match self.peek_token() {
            Some(Keyword(Kw::Transport | Kw::Inertial)) => {
                self.start_node(DelayMechanism);
                self.skip();
                self.end_node();
            }
            Some(Keyword(Kw::Reject)) => {
                self.start_node(DelayMechanism);
                self.skip();
                self.expression();
                self.expect_kw(Kw::Inertial);
                self.end_node();
            }
            _ => {}
        }
    }

    pub fn target(&mut self) {
        self.name()
    }

    fn concurrent_assert_statement_inner(&mut self) {
        self.opt_token(Keyword(Kw::Postponed));
        self.assertion();
        self.expect_token(SemiColon);
    }

    pub fn assertion(&mut self) {
        self.start_node(Assertion);
        self.expect_kw(Kw::Assert);
        self.condition();
        if self.opt_token(Keyword(Kw::Report)) {
            self.expression();
        }
        if self.opt_token(Keyword(Kw::Severity)) {
            self.expression();
        }
        self.end_node();
    }

    fn case_generate_statement_inner(&mut self) {
        self.expect_kw(Kw::Case);
        self.condition();
        self.expect_kw(Kw::Generate);
        while self.next_is(Keyword(Kw::When)) {
            self.case_generate_alternative();
        }
        self.expect_tokens([Keyword(Kw::End), Keyword(Kw::Generate)]);
        self.opt_identifier();
        self.expect_token(SemiColon);
    }

    pub fn case_generate_alternative(&mut self) {
        self.start_node(CaseGenerateAlternative);
        self.expect_kw(Kw::When);
        self.opt_label();
        self.choices();
        self.expect_token(RightArrow);
        self.generate_statement_body();
        self.end_node();
    }

    fn for_generate_statement_inner(&mut self) {
        self.expect_kw(Kw::For);
        self.parameter_specification();
        self.expect_kw(Kw::Generate);
        self.generate_statement_body();
        self.expect_tokens([Keyword(Kw::End), Keyword(Kw::Generate)]);
        self.opt_identifier();
        self.expect_token(SemiColon);
    }

    fn if_generate_statement_inner(&mut self) {
        self.expect_kw(Kw::If);
        self.opt_label();
        self.condition();
        self.expect_kw(Kw::Generate);
        self.generate_statement_body();
        while self.opt_token(Keyword(Kw::Elsif)) {
            self.opt_label();
            self.condition();
            self.expect_kw(Kw::Generate);
            self.generate_statement_body();
        }
        if self.opt_token(Keyword(Kw::Else)) {
            self.opt_label();
            self.expect_kw(Kw::Generate);
            self.generate_statement_body();
        }
        self.expect_tokens([Keyword(Kw::End), Keyword(Kw::Generate)]);
        self.opt_identifier();
        self.expect_token(SemiColon);
    }

    pub fn generate_statement_body(&mut self) {
        self.start_node(GenerateStatementBody);
        self.declarative_part();
        self.opt_token(Keyword(Kw::Begin));
        self.concurrent_statements();
        if self.opt_token(Keyword(Kw::End)) {
            self.opt_identifier();
            self.opt_token(SemiColon);
        }
        self.end_node();
    }

    pub fn parameter_specification(&mut self) {
        self.start_node(ParameterSpecification);
        self.identifier();
        self.expect_kw(Kw::In);
        self.discrete_range();
        self.end_node();
    }

    fn instantiation_statement_inner(&mut self) {
        self.opt_generic_map_aspect();
        self.opt_port_map_aspect();
    }

    fn process_statement_inner(&mut self) {
        self.opt_token(Keyword(Kw::Postponed));
        self.expect_token(Keyword(Kw::Process));
        if self.opt_token(LeftPar) {
            self.process_sensitivity_list();
            self.expect_token(RightPar);
        }
        self.opt_token(Keyword(Kw::Is));
        self.declarative_part();
        self.expect_token(Keyword(Kw::Begin));
        self.concurrent_statements();
        self.expect_kw(Kw::End);
        self.opt_token(Keyword(Kw::Postponed));
        self.expect_token(Keyword(Kw::Process));
        self.opt_identifier();
        self.expect_token(SemiColon);
    }

    pub fn process_sensitivity_list(&mut self) {
        if !self.opt_token(Keyword(Kw::All)) {
            self.sensitivity_list()
        }
    }

    pub fn sensitivity_list(&mut self) {
        self.separated_list(Parser::name, Comma);
    }
}
