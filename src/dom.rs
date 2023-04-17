use std::borrow::Cow;

use generational_arena::{Arena, Index};
use html5ever::{tendril::StrTendril, tree_builder::TreeSink, QualName};

#[derive(Debug, Clone, Copy)]
pub struct NodeHandle(pub Index);

/// The different kinds of nodes in the DOM.
#[derive(Debug)]
pub enum Node {
    /// The `Document` itself - the root node of a HTML document.
    Document,

    /// A `DOCTYPE` with name, public id, and system id. See
    /// [document type declaration on wikipedia][dtd wiki].
    ///
    /// [dtd wiki]: https://en.wikipedia.org/wiki/Document_type_declaration
    Doctype {
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    },

    /// A text node.
    Text { contents: StrTendril },

    /// A comment.
    Comment { contents: StrTendril },

    /// An element with attributes.
    Element {
        elem: html_tags::ElementOwned,
        qualified_name: QualName,
        parent: NodeHandle,
        children: Vec<NodeHandle>,
    },

    /// A Processing instruction.
    ProcessingInstruction {
        target: StrTendril,
        contents: StrTendril,
    },
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default)]
pub struct ArenaDom {
    pub arena: Arena<Node>,
    pub document: Option<NodeHandle>,
}

impl TreeSink for ArenaDom {
    type Handle = NodeHandle;
    type Output = Self;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        eprintln!("Parse error: {msg}");
    }
    fn get_document(&mut self) -> Self::Handle {
        if let Some(doc) = self.document {
            doc
        } else {
            let doc = NodeHandle(self.arena.insert(Node::Document));
            self.document = Some(doc);
            doc
        }
    }
    fn elem_name(&self, target: &Self::Handle) -> html5ever::ExpandedName {
        match &self.arena[target.0] {
            Node::Element { qualified_name, .. } => qualified_name.expanded(),
            _ => panic!("Not an element"),
        }
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<html5ever::Attribute>,
        _flags: html5ever::tree_builder::ElementFlags,
    ) -> Self::Handle {
        // TODO: attrs
        let parent = self.get_document();
        NodeHandle(self.arena.insert(Node::Element {
            elem: html_tags::ElementOwned::from_tag(&name.local),
            qualified_name: name,
            parent,
            children: Vec::new(),
        }))
    }

    fn create_comment(&mut self, contents: StrTendril) -> Self::Handle {
        NodeHandle(self.arena.insert(Node::Comment { contents }))
    }

    fn create_pi(&mut self, target: StrTendril, contents: StrTendril) -> Self::Handle {
        NodeHandle(
            self.arena
                .insert(Node::ProcessingInstruction { target, contents }),
        )
    }

    fn append(
        &mut self,
        parent: &Self::Handle,
        child: html5ever::tree_builder::NodeOrText<Self::Handle>,
    ) {
        match child {
            html5ever::tree_builder::NodeOrText::AppendNode(node) => {
                let node = &mut self.arena[node.0];
                match node {
                    Node::Element { children, .. } => {
                        children.push(*parent);
                    }
                    _ => panic!("Not an element"),
                }
            }
            html5ever::tree_builder::NodeOrText::AppendText(contents) => {
                match &mut self.arena[parent.0] {
                    Node::Element { children, .. } => {
                        let text = NodeHandle(self.arena.insert(Node::Text { contents }));
                        children.push(text);
                    }
                    _ => panic!("Not an element"),
                }
            }
        }
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        _prev_element: &Self::Handle,
        child: html5ever::tree_builder::NodeOrText<Self::Handle>,
    ) {
        self.append(element, child);
    }

    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        let doc = self.get_document();
        let doctype = NodeHandle(self.arena.insert(Node::Doctype {
            name,
            public_id,
            system_id,
        }));
        match &mut self.arena[doc.0] {
            Node::Element { children, .. } => {
                children.push(doctype);
            }
            _ => panic!("Not an element"),
        }
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        todo!()
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x.0 == y.0
    }

    fn set_quirks_mode(&mut self, mode: html5ever::tree_builder::QuirksMode) {
        todo!()
    }

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: html5ever::tree_builder::NodeOrText<Self::Handle>,
    ) {
        todo!()
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        todo!()
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        todo!()
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        todo!()
    }
}
