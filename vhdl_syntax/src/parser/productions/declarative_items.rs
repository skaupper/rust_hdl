// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com
/// Parsing of the different declarative items
use crate::parser::Parser;
use crate::syntax::node_kind::NodeKind::*;
use crate::tokens::token_kind::Keyword as Kw;
use crate::tokens::TokenKind::*;
use crate::tokens::TokenStream;

struct DeclarativeItemOptions {
    subprogram_body: bool,
    subprogram_decl: bool,
    package_body: bool,
    package_decl: bool,
    type_decl: bool,
    constant_decl: bool,
    variable_decl: bool,
    signal_decl: bool,
    file_decl: bool,
    alias_decl: bool,
    component_decl: bool,
    attribute_decl: bool,
    attribute_spec: bool,
    disconnect_spec: bool,
    use_clause: bool,
    group_template_decl: bool,
    group_decl: bool,
}

impl<T: TokenStream> Parser<T> {
    // TODO: Add support for PSL constructs in all relevant declarative items

    /// An internal function which is capable of parsing all declarative items.
    /// To be flexible, these items must be enabled/disabled separately using the `DeclarativeItemOptions` struct.
    fn opt_declarative_item(&mut self, options: DeclarativeItemOptions) -> bool {
        match self.peek_token() {
            Some(tok) => match tok {
                Keyword(Kw::Procedure) | Keyword(Kw::Function) => {
                    // Skip the first token which can either be `function` or `procedure` and was already checked
                    let is_subpgm_inst_decl = options.subprogram_decl
                        && self
                            .next_is_seq_skip(1, [Identifier, Keyword(Kw::Is), Keyword(Kw::New)]);
                    let is_subpgm_body = options.subprogram_body
                        && !is_subpgm_inst_decl
                        && self.next_is_seq_skip(1, [Identifier, Keyword(Kw::Is)]);
                    let is_subpgm_decl = options.subprogram_decl
                        && (!is_subpgm_inst_decl && !is_subpgm_body)
                        && self.next_is_seq_skip(1, [Identifier]);

                    if is_subpgm_inst_decl {
                        self.subprogram_instantiation_declaration();
                    } else if is_subpgm_body {
                        self.subprogram_body();
                    } else if is_subpgm_decl {
                        self.subprogram_declaration();
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Package) => {
                    let is_pkg_inst_decl = options.package_decl
                        && self.next_is_seq([
                            Keyword(Kw::Package),
                            Identifier,
                            Keyword(Kw::Is),
                            Keyword(Kw::New),
                        ]);
                    let is_pkg_body = options.package_body
                        && self.next_is_seq([Keyword(Kw::Package), Keyword(Kw::Body)]);
                    let is_pkg_decl = options.package_decl
                        && !is_pkg_inst_decl
                        && self.next_is_seq([Keyword(Kw::Package), Identifier, Keyword(Kw::Is)]);

                    if is_pkg_inst_decl {
                        self.package_instantiation_declaration();
                    } else if is_pkg_body {
                        self.package_body();
                    } else if is_pkg_decl {
                        self.package_declaration();
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Type) => {
                    if options.type_decl {
                        self.type_declaration()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Subtype) => {
                    if options.type_decl {
                        self.subtype_declaration()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Constant) => {
                    if options.constant_decl {
                        self.constant_declaration()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Variable) => {
                    if options.variable_decl {
                        self.variable_declaration()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Signal) => {
                    if options.signal_decl {
                        self.signal_declaration()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::File) => {
                    if options.file_decl {
                        self.file_declaration()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Alias) => {
                    if options.alias_decl {
                        self.alias_declaration()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Component) => {
                    if options.component_decl {
                        self.component_declaration()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Attribute) => {
                    let is_attr_decl = options.attribute_decl
                        && self.next_is_seq([Keyword(Kw::Attribute), Identifier, Colon]);
                    let is_attr_spec = options.attribute_spec
                        && self.next_is_seq([Keyword(Kw::Attribute), Identifier, Keyword(Kw::Of)]);

                    if is_attr_decl {
                        self.attribute_declaration();
                    } else if is_attr_spec {
                        self.attribute_specification();
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Disconnect) => {
                    if options.disconnect_spec {
                        self.disconnection_specification()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Use) => {
                    if options.use_clause {
                        self.use_clause()
                    } else {
                        return false;
                    }
                }
                Keyword(Kw::Group) => {
                    let is_grp_tmpl_decl = options.group_template_decl
                        && self.next_is_seq([
                            Keyword(Kw::Group),
                            Identifier,
                            Keyword(Kw::Is),
                            LeftPar,
                        ]);
                    let is_grp_decl = options.group_decl
                        && self.next_is_seq([Keyword(Kw::Group), Identifier, Colon]);

                    if is_grp_tmpl_decl {
                        self.group_template_declaration();
                    } else if is_grp_decl {
                        self.group_declaration();
                    } else {
                        return false;
                    }
                }
                _ => return false,
            },
            None => {
                // TODO: Do we need EOF handling when no result is a valid result?
                return false;
            }
        };
        true
    }

    fn opt_protected_type_declarative_item(&mut self) -> bool {
        self.opt_declarative_item(DeclarativeItemOptions {
            subprogram_body: false,
            subprogram_decl: true,
            package_body: false,
            package_decl: false,
            type_decl: false,
            constant_decl: false,
            variable_decl: false,
            signal_decl: false,
            file_decl: false,
            alias_decl: false,
            component_decl: false,
            attribute_decl: false,
            attribute_spec: true,
            disconnect_spec: false,
            use_clause: true,
            group_template_decl: false,
            group_decl: false,
        })
    }

    fn opt_protected_type_body_declarative_item(&mut self) -> bool {
        self.opt_declarative_item(DeclarativeItemOptions {
            subprogram_body: true,
            subprogram_decl: true,
            package_body: true,
            package_decl: true,
            type_decl: true,
            constant_decl: true,
            variable_decl: true,
            signal_decl: false,
            file_decl: true,
            alias_decl: true,
            component_decl: false,
            attribute_decl: true,
            attribute_spec: true,
            disconnect_spec: false,
            use_clause: true,
            group_template_decl: true,
            group_decl: true,
        })
    }

    fn opt_package_declarative_item(&mut self) -> bool {
        self.opt_declarative_item(DeclarativeItemOptions {
            subprogram_body: false,
            subprogram_decl: true,
            package_body: false,
            package_decl: true,
            type_decl: true,
            constant_decl: true,
            variable_decl: true,
            signal_decl: true,
            file_decl: true,
            alias_decl: true,
            component_decl: true,
            attribute_decl: true,
            attribute_spec: true,
            disconnect_spec: true,
            use_clause: true,
            group_template_decl: true,
            group_decl: true,
        })
    }

    fn opt_package_body_declarative_item(&mut self) -> bool {
        self.opt_declarative_item(DeclarativeItemOptions {
            subprogram_body: true,
            subprogram_decl: true,
            package_body: true,
            package_decl: true,
            type_decl: true,
            constant_decl: true,
            variable_decl: true,
            signal_decl: false,
            file_decl: true,
            alias_decl: true,
            component_decl: false,
            attribute_decl: true,
            attribute_spec: true,
            disconnect_spec: false,
            use_clause: true,
            group_template_decl: true,
            group_decl: true,
        })
    }

    fn opt_subprogram_declarative_item(&mut self) -> bool {
        self.opt_declarative_item(DeclarativeItemOptions {
            subprogram_body: true,
            subprogram_decl: true,
            package_body: true,
            package_decl: true,
            type_decl: true,
            constant_decl: true,
            variable_decl: true,
            signal_decl: false,
            file_decl: true,
            alias_decl: true,
            component_decl: false,
            attribute_decl: true,
            attribute_spec: true,
            disconnect_spec: false,
            use_clause: true,
            group_template_decl: true,
            group_decl: true,
        })
    }

    fn opt_configuration_declarative_item(&mut self) -> bool {
        self.opt_declarative_item(DeclarativeItemOptions {
            subprogram_body: false,
            subprogram_decl: false,
            package_body: false,
            package_decl: false,
            type_decl: false,
            constant_decl: false,
            variable_decl: false,
            signal_decl: false,
            file_decl: false,
            alias_decl: false,
            component_decl: false,
            attribute_decl: false,
            attribute_spec: true,
            disconnect_spec: false,
            use_clause: true,
            group_template_decl: false,
            group_decl: true,
        })
    }

    fn opt_entity_declarative_item(&mut self) -> bool {
        self.opt_declarative_item(DeclarativeItemOptions {
            subprogram_body: true,
            subprogram_decl: true,
            package_body: true,
            package_decl: true,
            type_decl: true,
            constant_decl: true,
            variable_decl: true,
            signal_decl: true,
            file_decl: true,
            alias_decl: true,
            component_decl: false,
            attribute_decl: true,
            attribute_spec: true,
            disconnect_spec: true,
            use_clause: true,
            group_template_decl: true,
            group_decl: true,
        })
    }

    fn opt_block_declarative_item(&mut self) -> bool {
        self.opt_declarative_item(DeclarativeItemOptions {
            subprogram_body: true,
            subprogram_decl: true,
            package_body: true,
            package_decl: true,
            type_decl: true,
            constant_decl: true,
            variable_decl: true,
            signal_decl: true,
            file_decl: true,
            alias_decl: true,
            component_decl: true,
            attribute_decl: true,
            attribute_spec: true,
            disconnect_spec: true,
            use_clause: true,
            group_template_decl: true,
            group_decl: true,
        })
    }

    fn opt_process_declarative_item(&mut self) -> bool {
        self.opt_declarative_item(DeclarativeItemOptions {
            subprogram_body: true,
            subprogram_decl: true,
            package_body: true,
            package_decl: true,
            type_decl: true,
            constant_decl: true,
            variable_decl: true,
            signal_decl: false,
            file_decl: true,
            alias_decl: true,
            component_decl: false,
            attribute_decl: true,
            attribute_spec: true,
            disconnect_spec: false,
            use_clause: true,
            group_template_decl: true,
            group_decl: true,
        })
    }

    pub(crate) fn entity_declarative_part(&mut self) {
        self.start_node(EntityDeclarativePart);
        while self.opt_entity_declarative_item() {}
        self.end_node();
    }

    pub(crate) fn architecture_declarative_part(&mut self) {
        self.start_node(ArchitectureDeclarativePart);
        while self.opt_block_declarative_item() {}
        self.end_node();
    }

    pub(crate) fn configuration_declarative_part(&mut self) {
        self.start_node(ConfigurationDeclarativePart);
        while self.opt_configuration_declarative_item() {}
        self.end_node();
    }

    pub(crate) fn subprogram_declarative_part(&mut self) {
        self.start_node(SubprogramDeclarativePart);
        while self.opt_subprogram_declarative_item() {}
        self.end_node();
    }

    pub(crate) fn package_declarative_part(&mut self) {
        self.start_node(PackageDeclarativePart);
        while self.opt_package_declarative_item() {}
        self.end_node();
    }

    pub(crate) fn package_body_declarative_part(&mut self) {
        self.start_node(PackageBodyDeclarativePart);
        while self.opt_package_body_declarative_item() {}
        self.end_node();
    }
    pub(crate) fn protected_type_declarative_part(&mut self) {
        self.start_node(ProtectedTypeDeclarativePart);
        while self.opt_protected_type_declarative_item() {}
        self.end_node();
    }
    pub(crate) fn protected_type_body_declarative_part(&mut self) {
        self.start_node(ProtectedTypeBodyDeclarativePart);
        while self.opt_protected_type_body_declarative_item() {}
        self.end_node();
    }
    pub(crate) fn block_declarative_part(&mut self) {
        self.start_node(BlockDeclarativePart);
        while self.opt_block_declarative_item() {}
        self.end_node();
    }
    pub(crate) fn process_declarative_part(&mut self) {
        self.start_node(ProcessDeclarativePart);
        while self.opt_process_declarative_item() {}
        self.end_node();
    }
}
