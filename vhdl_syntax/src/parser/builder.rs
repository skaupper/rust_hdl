// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c)  2024, Lukas Scheller lukasscheller@icloud.com

use crate::syntax::green::{GreenNode, GreenNodeData};
use crate::syntax::node_kind::NodeKind;
use crate::tokens::Token;

/// Internal builder used to create nodes when parsing.
pub(crate) struct NodeBuilder {
    stack: Vec<(usize, GreenNodeData)>,
    rel_offset: usize,
    text_len: usize,
    token_index: usize,
}

impl NodeBuilder {
    pub fn new() -> NodeBuilder {
        NodeBuilder {
            stack: Vec::default(),
            rel_offset: 0,
            text_len: 0,
            token_index: 0,
        }
    }

    fn current(&mut self) -> &mut GreenNodeData {
        &mut self.stack.last_mut().unwrap().1
    }

    pub fn push(&mut self, token: Token) {
        let tok_text_len = token.byte_len();
        let offset = self.rel_offset;
        self.current().push_token(offset, token);
        self.rel_offset += tok_text_len;
        self.token_index += 1;
    }

    pub fn start_node(&mut self, kind: NodeKind) {
        self.stack.push((self.rel_offset, GreenNodeData::new(kind)));
        self.rel_offset = 0;
    }

    pub fn end_node(&mut self) {
        if self.stack.len() == 1 {
            return;
        }
        let (offset, node) = self.stack.pop().expect("Unbalanced start_node / end_node");
        self.current().push_node(offset, node)
    }

    /// Ends the current node and changes the kind.
    /// This is useful when the exact kind cannot be determined a priori (resp. this is hard).
    /// For instance, when parsing a type, the following production
    /// ```vhdl
    /// type foo
    /// ```
    /// could either be an incomplete type:
    /// ```vhdl
    /// type foo;
    /// ```
    /// or a regular type indication:
    /// ```vhdl
    /// type foo is -- ...
    /// ```
    pub fn end_node_with_kind(&mut self, kind: NodeKind) {
        if self.stack.len() == 1 {
            return;
        }
        let (offset, mut node) = self.stack.pop().expect("Unbalanced start_node / end_node");
        node.set_kind(kind);
        self.current().push_node(offset, node)
    }

    pub fn end(mut self) -> GreenNode {
        GreenNode::new(
            self.stack
                .pop()
                .expect("Unbalanced start_node / end_node")
                .1,
        )
    }

    pub fn current_pos(&self) -> usize {
        self.text_len
    }

    pub fn current_token_index(&self) -> usize {
        self.token_index
    }
}
