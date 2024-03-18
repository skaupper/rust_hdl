// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2018, Olof Kraigher olof.kraigher@gmail.com

use crate::ast::DesignFile;
use crate::data::*;
use fnv::FnvHashSet;

pub struct SourceFile {
    pub(crate) library_names: FnvHashSet<Symbol>,
    pub(crate) source: Source,
    pub(crate) design_file: DesignFile,
    pub(crate) parser_diagnostics: Vec<Diagnostic>,
}

impl SourceFile {
    pub(crate) fn take_design_file(&mut self) -> DesignFile {
        std::mem::take(&mut self.design_file)
    }

    pub fn num_lines(&self) -> usize {
        self.source.contents().num_lines()
    }
}
