#![allow(clippy::future_not_send)]

use html5ever::{
    local_name, namespace_url, ns, tendril::StrTendril, tree_builder::TreeSink, QualName,
};
use lightningcss::rules::CssRuleList;
use ouroboros::self_referencing;
use slotmap::{new_key_type, HopSlotMap};
use std::borrow::Cow;

new_key_type! {
    pub struct NodeHandle;
}
/// A `DOCTYPE` with name, public id, and system id. See
/// [document type declaration on wikipedia][dtd wiki].
///
/// [dtd wiki]: https://en.wikipedia.org/wiki/Document_type_declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Doctype {
    name: StrTendril,
    public_id: StrTendril,
    system_id: StrTendril,
}

#[self_referencing]
#[derive(Debug)]
pub struct StyleSheet {
    pub contents: StrTendril,
    #[borrows(contents)]
    #[not_covariant]
    pub rules: Result<
        CssRuleList<'this>,
        cssparser::ParseError<'this, lightningcss::error::ParserError<'this>>,
    >,
}

/// The different kinds of nodes in the DOM.
#[derive(Debug)]
pub enum Node {
    /// The `Document` itself - the root node of a HTML document.
    Document(Option<Doctype>),

    /// A text node.
    Text(StrTendril),

    /// A comment.
    Comment(StrTendril),

    /// An element with attributes.
    Element {
        elem: html_tags::ElementOwned,
        qualified_name: QualName,
        parent: NodeHandle,
        children: Vec<NodeHandle>,
    },

    /// A stylesheet.
    StyleSheet(StyleSheet),

    /// A Processing instruction.
    ProcessingInstruction {
        target: StrTendril,
        contents: StrTendril,
    },
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default)]
pub struct Dom {
    map: HopSlotMap<NodeHandle, Node>,
    pub document: Option<NodeHandle>,
}
impl Dom {
    pub const fn map(&self) -> &HopSlotMap<NodeHandle, Node> {
        &self.map
    }
}

#[allow(dead_code, unused_variables)]
impl TreeSink for Dom {
    type Handle = NodeHandle;
    type Output = Self;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        tracing::error!("parse error: {msg}");
    }
    fn get_document(&mut self) -> Self::Handle {
        if let Some(doc) = self.document {
            doc
        } else {
            let doc = self.map.insert(Node::Document(None));
            self.document = Some(doc);
            doc
        }
    }
    fn elem_name(&self, &target: &Self::Handle) -> html5ever::ExpandedName {
        match &self.map[target] {
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
        let mut elem = html_tags::ElementOwned::from_tag(&name.local);
        let mut style = None;
        for attr in attrs {
            if attr.name == QualName::new(None, ns!(), local_name!("style")) {
                style = Some(attr.value.clone());
            }
            elem.set_attr(&attr.name.local, attr.value);
        }

        let parent = self.get_document();

        macro_rules! style {
            ($content:expr) => {{
                let stylesheet = StyleSheet::new(StrTendril::from(""), |contents| {
                    let mut input = cssparser::ParserInput::new(contents);
                    let mut parser = cssparser::Parser::new(&mut input);
                    CssRuleList::parse(
                        &mut parser,
                        &lightningcss::stylesheet::ParserOptions::default(),
                    )
                });

                let style = self.map.insert(Node::StyleSheet(stylesheet));

                vec![style]
            }};
        }

        let children = if matches!(elem, html_tags::ElementOwned::Style(_)) {
            style!(StrTendril::new())
        } else if let Some(style) = style {
            style!(style)
        } else {
            vec![]
        };

        self.map.insert(Node::Element {
            elem,
            qualified_name: name,
            parent,
            children,
        })
    }

    fn create_comment(&mut self, contents: StrTendril) -> Self::Handle {
        self.map.insert(Node::Comment(contents))
    }

    fn create_pi(&mut self, target: StrTendril, contents: StrTendril) -> Self::Handle {
        self.map
            .insert(Node::ProcessingInstruction { target, contents })
    }

    fn append(
        &mut self,
        &parent: &Self::Handle,
        child: html5ever::tree_builder::NodeOrText<Self::Handle>,
    ) {
        match child {
            html5ever::tree_builder::NodeOrText::AppendNode(node) => {
                let node = &mut self.map[node];
                match node {
                    Node::Element { children, .. } => {
                        children.push(parent);
                    }
                    _ => panic!("Not an element"),
                }
            }
            html5ever::tree_builder::NodeOrText::AppendText(text) => {
                let text = self.map.insert(Node::Text(text));
                match &mut self.map[parent] {
                    Node::Element { children, .. } => {
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
        tracing::warn!("partially implemented - append based on parent node");
        self.append(element, child);
    }

    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        let doc = self.get_document();
        self.map[doc] = Node::Document(Some(Doctype {
            name,
            public_id,
            system_id,
        }));
    }

    fn get_template_contents(&mut self, &target: &Self::Handle) -> Self::Handle {
        tracing::error!("not implemented - get template contents");
        todo!();
    }

    fn same_node(&self, &x: &Self::Handle, &y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&mut self, mode: html5ever::tree_builder::QuirksMode) {
        tracing::warn!("not implemented - quirks mode: {mode:?}");
    }

    fn append_before_sibling(
        &mut self,
        &sibling: &Self::Handle,
        new_node: html5ever::tree_builder::NodeOrText<Self::Handle>,
    ) {
        tracing::warn!("not implemented - append before sibling");
    }

    fn add_attrs_if_missing(&mut self, &target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        match &mut self.map[target] {
            Node::Element { elem, .. } => {
                for attr in attrs {
                    elem.set_attr(&attr.name.local, attr.value);
                }
            }
            _ => panic!("Not an element"),
        }
    }

    fn remove_from_parent(&mut self, &target: &Self::Handle) {
        match self.map[target] {
            Node::Element { parent, .. } => {
                let parent = &mut self.map[parent];
                match parent {
                    Node::Element { children, .. } => {
                        children.retain(|&x| x != target);
                    }
                    _ => panic!("Not an element"),
                }
            }
            _ => panic!("Not an element"),
        }
    }

    fn reparent_children(&mut self, &node: &Self::Handle, &new_parent: &Self::Handle) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use html5ever::{parse_document, tendril::TendrilSink, ParseOpts};

    #[test]
    fn basic() {
        let dom = parse_document(Dom::default(), ParseOpts::default());
        let dom = dom.one(include_str!("../test.html"));

        println!("{dom:#?}");
    }
}
