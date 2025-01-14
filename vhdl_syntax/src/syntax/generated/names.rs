// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2025, Lukas Scheller lukasscheller@icloud.com
use crate::syntax::generated::*;
use crate::syntax::node::{SyntaxNode, SyntaxToken};
use crate::syntax::node_kind::NodeKind;
use crate::syntax::AstNode;
use crate::tokens::Keyword as Kw;
use crate::tokens::TokenKind::*;
pub enum DesignatorSyntax {
    Identifier(SyntaxToken),
    StringLiteral(SyntaxToken),
    CharacterLiteral(SyntaxToken),
}
impl DesignatorSyntax {
    pub fn cast(token: SyntaxToken) -> Option<Self> {
        match token.kind() {
            Identifier => Some(DesignatorSyntax::Identifier(token)),
            StringLiteral => Some(DesignatorSyntax::StringLiteral(token)),
            CharacterLiteral => Some(DesignatorSyntax::CharacterLiteral(token)),
            _ => None,
        }
    }
    pub fn raw(&self) -> SyntaxToken {
        match self {
            DesignatorSyntax::Identifier(inner) => inner.clone(),
            DesignatorSyntax::StringLiteral(inner) => inner.clone(),
            DesignatorSyntax::CharacterLiteral(inner) => inner.clone(),
        }
    }
}
pub enum NameSyntax {}
impl AstNode for NameSyntax {
    fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            _ => None,
        }
    }
    fn raw(&self) -> SyntaxNode {
        match self {
            _ => unreachable!(),
        }
    }
}
