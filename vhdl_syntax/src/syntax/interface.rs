// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com

use crate::syntax::node::{SyntaxNode, SyntaxToken};
use crate::syntax::node_kind::NodeKind;
use crate::syntax::AstNode;
use crate::tokens::Keyword as Kw;
use crate::tokens::TokenKind::*;

pub struct GenericClause(pub(crate) SyntaxNode);

impl AstNode for GenericClause {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            NodeKind::GenericClause => Some(GenericClause(node)),
            _ => None,
        }
    }

    fn raw(&self) -> SyntaxNode {
        self.0.clone()
    }
}

impl GenericClause {
    pub fn generic_token(&self) -> Option<SyntaxToken> {
        self.0
            .tokens()
            .find(|tok| tok.kind() == Keyword(Kw::Generic))
    }

    pub fn left_par_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| tok.kind() == LeftPar)
    }

    pub fn interface_list(&self) -> Option<InterfaceList> {
        self.0.children().find_map(InterfaceList::cast)
    }

    pub fn right_par_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| tok.kind() == RightPar)
    }

    pub fn semi_colon_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| tok.kind() == SemiColon)
    }
}

pub struct PortClause(pub(crate) SyntaxNode);

impl PortClause {
    pub fn port_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| tok.kind() == Keyword(Kw::Port))
    }

    pub fn left_par_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| tok.kind() == LeftPar)
    }

    pub fn interface_list(&self) -> Option<InterfaceList> {
        self.0.children().find_map(InterfaceList::cast)
    }

    pub fn right_par_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| tok.kind() == RightPar)
    }

    pub fn semi_colon_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| tok.kind() == SemiColon)
    }
}

impl AstNode for PortClause {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            NodeKind::PortClause => Some(PortClause(node)),
            _ => None,
        }
    }

    fn raw(&self) -> SyntaxNode {
        self.0.clone()
    }
}

pub struct InterfaceList(pub(crate) SyntaxNode);

impl AstNode for InterfaceList {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            NodeKind::InterfaceList => Some(InterfaceList(node)),
            _ => None,
        }
    }

    fn raw(&self) -> SyntaxNode {
        self.0.clone()
    }
}

impl InterfaceList {
    pub fn interface_declarations(&self) -> impl Iterator<Item = InterfaceDeclaration> + use<'_> {
        self.0.children().filter_map(InterfaceDeclaration::cast)
    }

    pub fn semi_colon_tokens(&self) -> impl Iterator<Item = SyntaxToken> + use<'_> {
        self.0.tokens().filter(|tok| tok.kind() == SemiColon)
    }
}

pub enum InterfaceDeclaration {
    Object(InterfaceObjectDeclaration),
}

impl AstNode for InterfaceDeclaration {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            NodeKind::InterfaceObjectDeclaration => Some(InterfaceDeclaration::Object(
                InterfaceObjectDeclaration(node),
            )),
            _ => None,
        }
    }

    fn raw(&self) -> SyntaxNode {
        match self {
            InterfaceDeclaration::Object(obj) => obj.raw(),
        }
    }
}

pub struct InterfaceObjectDeclaration(pub(crate) SyntaxNode);

impl AstNode for InterfaceObjectDeclaration {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            NodeKind::InterfaceObjectDeclaration => Some(InterfaceObjectDeclaration(node)),
            _ => None,
        }
    }

    fn raw(&self) -> SyntaxNode {
        self.0.clone()
    }
}

impl InterfaceObjectDeclaration {
    pub fn interface_class_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| {
            matches!(
                tok.kind(),
                Keyword(Kw::Constant | Kw::Signal | Kw::Variable)
            )
        })
    }

    pub fn identifier_list(&self) -> Option<IdentifierList> {
        self.0.children().find_map(IdentifierList::cast)
    }

    pub fn colon_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| tok.kind() == Colon)
    }

    pub fn mode_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| {
            matches!(
                tok.kind(),
                Keyword(Kw::In | Kw::Out | Kw::Inout | Kw::Buffer | Kw::Linkage)
            )
        })
    }

    pub fn bus_token(&self) -> Option<SyntaxToken> {
        self.0.tokens().find(|tok| tok.kind() == Keyword(Kw::Bus))
    }
}

pub struct IdentifierList(pub(crate) SyntaxNode);

impl AstNode for IdentifierList {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            NodeKind::IdentifierList => Some(IdentifierList(node)),
            _ => None,
        }
    }

    fn raw(&self) -> SyntaxNode {
        self.0.clone()
    }
}
