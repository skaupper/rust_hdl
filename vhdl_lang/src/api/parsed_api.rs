// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2023, Olof Kraigher olof.kraigher@gmail.com

use std::path::PathBuf;

use fnv::{FnvHashMap, FnvHashSet};

use crate::{
    ast::{
        AnyDesignUnit, AnyPrimaryUnit, Declaration, EntityDeclaration, PackageDeclaration,
        PackageInstantiation, SubprogramDeclaration,
    },
    data::{FilePath, Symbol},
    SourceFile,
};

pub struct ParsedApiRoot<'a> {
    pub(super) parsed_files: &'a FnvHashMap<FilePath, SourceFile>,
}

pub struct ParsedFile<'a> {
    file_path: &'a FilePath,
    source_file: &'a SourceFile,
}

pub struct ParsedLibrary<'a> {
    library_name: &'a Symbol,
    files: FnvHashMap<&'a FilePath, &'a SourceFile>,
}

pub struct ParsedPackage<'a> {
    package_decl: &'a PackageDeclaration,
}

impl<'a> ParsedApiRoot<'a> {
    pub fn iter_files(&self) -> impl Iterator<Item = ParsedFile<'a>> {
        self.parsed_files
            .iter()
            .map(|(file_path, source_file)| ParsedFile {
                file_path,
                source_file,
            })
    }

    pub fn iter_packages(&self) -> impl Iterator<Item = ParsedPackage<'a>> + 'a {
        self.iter_files().flat_map(|f| f.iter_packages())
    }

    pub fn library(&self, library_name: &str) -> Option<ParsedLibrary<'a>> {
        let requested_library = self
            .parsed_files
            .iter()
            .filter_map(|(_, source_file)| {
                source_file
                    .library_names
                    .iter()
                    .find(|lib_name| lib_name.name_utf8() == library_name)
            })
            .nth(0)?;

        let files = self
            .parsed_files
            .iter()
            .filter_map(|(file_path, source_file)| {
                let lib_found = source_file
                    .library_names
                    .iter()
                    .any(|lib_name| lib_name == requested_library);

                if !lib_found {
                    return None;
                }

                Some((file_path, source_file))
            })
            .collect();

        Some(ParsedLibrary {
            library_name: requested_library,
            files,
        })
    }
}

impl<'a> ParsedFile<'a> {
    pub fn file_path(&self) -> PathBuf {
        self.file_path.to_path_buf()
    }

    pub fn library_names(&self) -> FnvHashSet<String> {
        self.source_file
            .library_names
            .iter()
            .map(|sym| sym.name_utf8())
            .collect()
    }

    pub fn iter_design_units(&self) -> impl Iterator<Item = &'a AnyDesignUnit> {
        self.source_file
            .design_file
            .design_units
            .iter()
            .map(|(_, design_unit)| design_unit)
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = &'a EntityDeclaration> {
        self.iter_design_units().filter_map(|du| match du {
            AnyDesignUnit::Primary(AnyPrimaryUnit::Entity(ent)) => Some(ent),
            _ => None,
        })
    }

    pub fn iter_packages(&self) -> impl Iterator<Item = ParsedPackage<'a>> {
        self.iter_design_units().filter_map(|du| match du {
            AnyDesignUnit::Primary(AnyPrimaryUnit::Package(ent)) => {
                Some(ParsedPackage { package_decl: ent })
            }
            _ => None,
        })
    }

    pub fn iter_package_bodies(&self) -> impl Iterator<Item = &'a PackageDeclaration> {
        self.iter_design_units().filter_map(|du| match du {
            AnyDesignUnit::Primary(AnyPrimaryUnit::Package(ent)) => Some(ent),
            _ => None,
        })
    }
}

impl<'a> ParsedLibrary<'a> {
    pub fn name(&self) -> String {
        self.library_name.name_utf8()
    }

    pub fn into_iter_files(self) -> impl Iterator<Item = ParsedFile<'a>> {
        self.files
            .into_iter()
            .map(|(file_path, source_file)| ParsedFile {
                file_path,
                source_file,
            })
    }

    pub fn iter_files(&'a self) -> impl Iterator<Item = ParsedFile> + 'a {
        self.files
            .iter()
            .map(|(file_path, source_file)| ParsedFile {
                file_path,
                source_file,
            })
    }
}

impl<'a> ParsedPackage<'a> {
    pub fn name(&self) -> String {
        self.package_decl.to_string()
    }

    pub fn iter_subprograms(&'a self) -> impl Iterator<Item = &'a SubprogramDeclaration> + 'a {
        self.package_decl.decl.iter().filter_map(|decl| match decl {
            Declaration::SubprogramDeclaration(sub_decl) => Some(sub_decl),
            _ => None,
        })
    }
}
