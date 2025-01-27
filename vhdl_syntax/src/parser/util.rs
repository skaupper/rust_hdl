// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2024, Lukas Scheller lukasscheller@icloud.com
/// (private) utility functions used when parsing
use crate::parser::diagnostics::ParserDiagnostic;
use crate::parser::diagnostics::ParserError::*;
use crate::parser::Parser;
use crate::syntax::green::GreenNode;
use crate::syntax::node_kind::NodeKind;
use crate::tokens::TokenStream;
use crate::tokens::{Keyword, TokenKind};

/// Allows match-style syntax for tokens.
/// This functino does not consume the next token.
/// If the token was not seen, or the parser is at EOF, this function pushes an error.
///
/// Note: It is possible to simulate 'Or' branches, i.e.,
/// ```no-test
/// match x {
///     A | B => { /*...*/ }
/// }
/// ```
/// However, due to the syntax limitations from macro_rules, these
/// are spelled out in the following way:
/// ```no-test
/// match_next_token!(x,
///     A, B => { /*...*/ }
/// )
/// ```
#[macro_export]
macro_rules! match_next_token {
    ($parser:expr, $($body:tt)*) => {
        match_next_token!(@inner $parser, [[ $($body)* ]], [[ $($body)* ]])
    };
    (@inner $parser:expr, [[ $($($pattern:pat_param),+ => $action:expr),+ ]], [[ $($($pattern_expr:expr),+ => $_action_expr:expr),+ ]]) => {
        match $parser.peek_token() {
            $(Some($($pattern)|+) => $action),+,
            None => $parser.eof_err(),
            _ => $parser.expect_tokens_err([$($($pattern_expr),+),+])
        }
    };
}

/// Allows match-style syntax for tokens.
/// This functino consumes the next token, if found.
/// If the token was not seen, or the parser is at EOF, this function pushes an error.
#[macro_export]
macro_rules! match_next_token_consume {
    ($parser:expr, $($body:tt)*) => {
        match_next_token_consume!(@inner $parser, [[ $($body)* ]], [[ $($body)* ]])
    };
    (@inner $parser:expr, [[ $($($pattern:pat_param),+ => $action:expr),+ ]], [[ $($($pattern_expr:expr),+ => $_action_expr:expr),+ ]]) => {
        match $parser.peek_token() {
            $(Some($($pattern)|+) => {
                $parser.skip();
                $action
            }),+
            None => $parser.eof_err(),
            _ => $parser.expect_tokens_err([$($($pattern_expr),+),+])
        }
    };
}

impl<T: TokenStream> Parser<T> {
    pub(crate) fn skip(&mut self) {
        if let Some(token) = self.tokenizer.next() {
            self.builder.push(token);
            self.token_index += 1;
        }
    }

    pub(crate) fn skip_n(&mut self, n: usize) {
        for _ in 0..n {
            self.skip();
            if self.peek_token().is_none() {
                break;
            }
        }
    }

    pub(crate) fn skip_to(&mut self, token_index: usize) {
        assert!(token_index > self.token_index);
        self.skip_n(token_index - self.token_index);
    }

    pub(crate) fn expect_token(&mut self, kind: TokenKind) {
        if let Some(token) = self.tokenizer.next_if(|token| token.kind() == kind) {
            self.builder.push(token);
            self.token_index += 1;
            return;
        }
        // TODO: what are possible recovery strategies?
        // - Leave as is
        // - Insert pseudo-token
        self.skip();
        self.expect_tokens_err([kind]);
    }

    pub(crate) fn expect_tokens<const N: usize>(&mut self, kinds: [TokenKind; N]) {
        for kind in kinds {
            self.expect_token(kind)
        }
    }

    pub(crate) fn expect_one_of_tokens<const N: usize>(&mut self, kinds: [TokenKind; N]) {
        for kind in kinds {
            if self.opt_token(kind) {
                return;
            }
        }
        self.expect_tokens_err(kinds);
    }

    pub(crate) fn peek_token(&self) -> Option<TokenKind> {
        Some(self.tokenizer.peek(0)?.kind())
    }

    pub(crate) fn peek_nth_token(&self, n: usize) -> Option<TokenKind> {
        Some(self.tokenizer.peek(n)?.kind())
    }

    pub(crate) fn next_is(&self, kind: TokenKind) -> bool {
        self.peek_token() == Some(kind)
    }

    pub(crate) fn next_is_one_of<const N: usize>(&self, kinds: [TokenKind; N]) -> bool {
        match self.peek_token() {
            Some(tok) => kinds.contains(&tok),
            None => false,
        }
    }

    pub(crate) fn next_nth_is(&self, kind: TokenKind, n: usize) -> bool {
        self.peek_nth_token(n) == Some(kind)
    }

    pub(crate) fn next_is_seq<const N: usize>(&mut self, kinds: [TokenKind; N]) -> bool {
        self.next_is_seq_skip(0, kinds)
    }

    pub(crate) fn next_is_seq_skip<const N: usize>(
        &mut self,
        skip_n: usize,
        kinds: [TokenKind; N],
    ) -> bool {
        (skip_n..)
            .zip(kinds.iter())
            .all(|(idx, tok)| self.peek_nth_token(idx) == Some(*tok))
    }

    pub(crate) fn expect_kw(&mut self, kind: Keyword) {
        self.expect_token(TokenKind::Keyword(kind))
    }

    pub(crate) fn opt_token(&mut self, kind: TokenKind) -> bool {
        if let Some(token) = self.tokenizer.next_if(|token| token.kind() == kind) {
            self.builder.push(token);
            self.token_index += 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn opt_tokens<const N: usize>(
        &mut self,
        kinds: [TokenKind; N],
    ) -> Option<TokenKind> {
        if let Some(token) = self
            .tokenizer
            .next_if(|token| kinds.contains(&token.kind()))
        {
            let kind = token.kind();
            self.builder.push(token);
            self.token_index += 1;
            Some(kind)
        } else {
            None
        }
    }

    pub(crate) fn start_node(&mut self, kind: NodeKind) {
        self.builder.start_node(kind)
    }

    pub(crate) fn end_node(&mut self) {
        self.builder.end_node()
    }

    pub(crate) fn eof_err(&mut self) {
        if !self.unexpected_eof {
            self.unexpected_eof = true;
            self.diagnostics
                .push(ParserDiagnostic::new(self.builder.current_pos(), Eof))
        }
    }

    pub(crate) fn expect_tokens_err(&mut self, tokens: impl Into<Box<[TokenKind]>>) {
        self.diagnostics.push(ParserDiagnostic::new(
            self.builder.current_pos(),
            ExpectingTokens(tokens.into()),
        ));
    }

    pub(crate) fn end(self) -> (GreenNode, Vec<ParserDiagnostic>) {
        (self.builder.end(), self.diagnostics)
    }

    pub(crate) fn lookahead<const N: usize>(
        &mut self,
        kinds: [TokenKind; N],
    ) -> Result<(TokenKind, usize), usize> {
        self.lookahead_max_token_index_skip_n(usize::MAX, 0, kinds)
    }

    pub(crate) fn lookahead_skip_n<const N: usize>(
        &mut self,
        skip_n: usize,
        kinds: [TokenKind; N],
    ) -> Result<(TokenKind, usize), usize> {
        self.lookahead_max_token_index_skip_n(usize::MAX, skip_n, kinds)
    }

    pub(crate) fn lookahead_max_token_index<const N: usize>(
        &mut self,
        maximum_index: usize,
        kinds: [TokenKind; N],
    ) -> Result<(TokenKind, usize), usize> {
        self.lookahead_max_token_index_skip_n(maximum_index, 0, kinds)
    }

    /// Lookahead in the current token stream until one of the given `TokenKind`s are found.
    /// In case of success, the matching `TokenKind` is returned, as well as the token index it was found at.
    /// In case of an error (EOF or a nesting error) the index at which the lookahead ended is returned.
    ///
    /// TODO: For better error handling you probably will need a way to differentiate between EOF and nesting errors!
    pub(crate) fn lookahead_max_token_index_skip_n<const N: usize>(
        &mut self,
        maximum_index: usize,
        skip_n: usize,
        kinds: [TokenKind; N],
    ) -> Result<(TokenKind, usize), usize> {
        let mut length = skip_n;
        let mut paren_count = 0;

        while self.token_index + length <= maximum_index && paren_count >= 0 {
            match self.peek_nth_token(length) {
                Some(TokenKind::LeftPar) => paren_count += 1,
                Some(TokenKind::RightPar) => {
                    // Allow the closing parenthesis to match as well
                    if paren_count == 0 && kinds.contains(&TokenKind::RightPar) {
                        return Ok((TokenKind::RightPar, self.token_index + length));
                    }

                    paren_count -= 1;

                    // A closing parenthesis indicates that some form of
                    // grouping ended that was not started during this lookahead.
                    if paren_count < 0 {
                        return Err(self.token_index + length);
                    }
                }

                Some(tok) => {
                    // To avoid matching tokens in some (potentially recursive) sub expression of some sort,
                    // only check the current token if we at the outer most grouping layer (`paren_count == 0`).
                    if paren_count == 0 && kinds.contains(&tok) {
                        return Ok((tok, self.token_index + length));
                    }
                }
                None => return Err(self.token_index + length),
            }
            length += 1;
        }

        Err(self.token_index + length)
    }
}
