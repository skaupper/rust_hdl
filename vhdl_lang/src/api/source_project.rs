use std::path::{Path, PathBuf};

use fnv::{FnvHashMap, FnvHashSet};

use super::parsed_project::ParsedProject;
use crate::data::{FilePath, Symbol};
use crate::{Config, Message, MessageHandler, Source, SourceFile, VHDLParser};

struct UnparsedSourceFile {
    library_names: Vec<String>,
    source: Source,
}

/// A completely unprocessed project, which is responsible for collecting configuration files and source files.
#[derive(Default)]
pub struct SourceProject {
    configs: Vec<Config>,
    files: FnvHashMap<FilePath, UnparsedSourceFile>,
    preloaded_sources: FnvHashMap<FilePath, Source>,
}

impl SourceProject {
    //
    // Constructors
    //
    pub fn from_config_file<T: AsRef<Path>>(path: T) -> std::io::Result<Self> {
        let mut inst = Self::default();
        inst.add_config_from_file(path)?;
        Ok(inst)
    }

    pub fn from_config(config: Config) -> Self {
        let mut inst = Self::default();
        inst.add_config(config);
        inst
    }

    //
    // Source file and config management
    //
    pub fn add_source_file<T: AsRef<Path>, U: Into<String>>(
        &mut self,
        path: T,
        library_name: U,
    ) -> std::io::Result<()> {
        let file_path = FilePath::new(path.as_ref());
        let source = Source::from_latin1_file(path.as_ref())?;
        self.files.insert(
            file_path,
            UnparsedSourceFile {
                library_names: vec![library_name.into()],
                source,
            },
        );
        Ok(())
    }

    pub fn add_config_from_file<T: AsRef<Path>>(&mut self, path: T) -> std::io::Result<()> {
        let config = Config::read_file_path(path.as_ref())?;
        self.add_config(config);
        Ok(())
    }

    pub fn add_config(&mut self, config: Config) {
        self.configs.push(config);
    }

    pub fn add_preloaded_source_files(&mut self, source_files: impl Iterator<Item = SourceFile>) {
        for source_file in source_files {
            self.preloaded_sources
                .insert(source_file.source.file_path().clone(), source_file.source);
        }
    }

    /// Process configs and source files and produce a `ParsedProject`
    pub fn parse(self, diagnostics: &mut dyn MessageHandler) -> ParsedProject {
        use rayon::prelude::*;

        // Local intermediate type
        struct FileEntry {
            library_symbols: FnvHashSet<Symbol>,
            source: Source,
        }

        // Merge all given configs (if any)
        let config = self
            .configs
            .into_iter()
            .reduce(|mut lhs, rhs| {
                lhs.append(&rhs, diagnostics);
                lhs
            })
            .unwrap_or(Config::default());

        let parser = VHDLParser::default();
        let mut preloaded_sources = self.preloaded_sources;
        let mut all_files = FnvHashMap::<PathBuf, FileEntry>::default();
        let mut empty_libraries = FnvHashSet::default();

        // Gather all source files referenced by configuration files
        for lib in config.iter_libraries() {
            let lib_symbol = parser.symbols.symtab().insert_utf8(lib.name());

            let files = lib.file_names(diagnostics);
            if files.is_empty() {
                empty_libraries.insert(lib_symbol);
                continue;
            }
            // Iterate over files in library specification
            for file_path in files {
                if let Some(entry) = all_files.get_mut(&file_path) {
                    // File is already in list
                    entry.library_symbols.insert(lib_symbol.clone());
                    continue;
                }

                let source =
                    if let Some(source) = preloaded_sources.remove(&FilePath::new(&file_path)) {
                        // File is already loaded
                        source
                    } else {
                        // File is not loaded
                        let source_result = Source::from_latin1_file(&file_path);
                        match source_result {
                            Ok(result) => result,
                            Err(err) => {
                                diagnostics.push(Message::file_error(err.to_string(), &file_path));
                                continue;
                            }
                        }
                    };

                all_files.insert(
                    file_path,
                    FileEntry {
                        library_symbols: FnvHashSet::from_iter([lib_symbol.clone()]),
                        source,
                    },
                );
            }
        }

        // Gather explicitely added source files
        for (file_path, unparsed_source_file) in self.files.into_iter() {
            if let Some(entry) = all_files.get_mut(&*file_path) {
                // File already exists
                for lib_name in unparsed_source_file.library_names {
                    entry
                        .library_symbols
                        .insert(parser.symbols.symtab().insert_utf8(&lib_name));
                }
            } else {
                // File needs an entry
                let library_symbols = unparsed_source_file
                    .library_names
                    .into_iter()
                    .map(|lib_name| parser.symbols.symtab().insert_utf8(&lib_name))
                    .collect();
                all_files.insert(
                    file_path.to_path_buf(),
                    FileEntry {
                        library_symbols,
                        source: unparsed_source_file.source,
                    },
                );
            }
        }

        // Parse all found files
        let mut parsed_project =
            ParsedProject::new(config, parser, FnvHashMap::default(), empty_libraries);
        let source_files = all_files
            .into_par_iter()
            .map_init(
                || &parsed_project.parser,
                |parser, (_, file_entry)| {
                    let mut diagnostics = Vec::new();
                    let design_file =
                        parser.parse_design_source(&file_entry.source, &mut diagnostics);
                    SourceFile {
                        library_names: file_entry.library_symbols,
                        source: file_entry.source,
                        design_file,
                        parser_diagnostics: diagnostics,
                    }
                },
            )
            .collect::<Vec<_>>();

        for f in source_files {
            parsed_project.add_source_file(f);
        }
        parsed_project
    }
}
