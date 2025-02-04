//! Parsing of statements
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::parser::Parser;
use crate::tokens::TokenStream;

impl<T: TokenStream> Parser<T> {
    pub fn labeled_concurrent_statements(&self) {
        // unimplemented!()
    }

    pub(crate) fn opt_sequential_statement(&mut self) -> bool {
        todo!();
    }
}
