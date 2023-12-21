use crate::dom_tree::append_to_existing_text;
use crate::dom_tree::Element;
use crate::dom_tree::NodeData;
use crate::dom_tree::NodeId;
use crate::dom_tree::NodeRef;
use crate::dom_tree::Tree;
use html5ever::parse_document;
use markup5ever::interface::tree_builder;
use markup5ever::interface::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use markup5ever::Attribute;
use markup5ever::ExpandedName;
use markup5ever::QualName;
use std::borrow::Cow;
use std::collections::HashSet;
use tendril::StrTendril;
use tendril::TendrilSink;

/// Document represents an HTML document to be manipulated.
pub struct Document {
    /// The document's dom tree.
    pub(crate) tree: Tree<NodeData>,

    /// Errors that occurred during parsing.
    pub errors: Vec<Cow<'static, str>>,

    /// The document's quirks mode.
    pub quirks_mode: QuirksMode,
}

impl Default for Document {
    fn default() -> Document {
        Self {
            tree: Tree::new(NodeData::Document),
            errors: vec![],
            quirks_mode: tree_builder::NoQuirks,
        }
    }
}

impl From<&str> for Document {
    fn from(html: &str) -> Document {
        parse_document(Document::default(), Default::default()).one(html)
    }
}

impl From<StrTendril> for Document {
    fn from(html: StrTendril) -> Document {
        parse_document(Document::default(), Default::default()).one(html)
    }
}

impl From<&String> for Document {
    fn from(html: &String) -> Document {
        Document::from(html.as_str())
    }
}

impl Document {
    /// Return the underlying root document node.
    pub fn root(&self) -> NodeRef<NodeData> {
        self.tree.root()
    }
}

impl TreeSink for Document {
    // The overall result of parsing.
    type Output = Self;

    // Consume this sink and return the overall result of parsing.
    fn finish(self) -> Self {
        self
    }

    // Handle is a reference to a DOM node. The tree builder requires that a `Handle` implements `Clone` to get
    // another reference to the same node.
    type Handle = NodeId;

    // Signal a parse error.
    fn parse_error(&mut self, msg: Cow<'static, str>) {
        self.errors.push(msg);
    }

    // Get a handle to the `Document` node.
    fn get_document(&mut self) -> NodeId {
        self.tree.root_id()
    }

    // Get a handle to a template's template contents. The tree builder promises this will never be called with
    // something else than a template element.
    fn get_template_contents(&mut self, target: &NodeId) -> NodeId {
        self.tree.query_node(target, |node| match node.data {
            NodeData::Element(Element {
                template_contents: Some(ref contents),
                ..
            }) => *contents,
            _ => panic!("not a template element!"),
        })
    }

    // Set the document's quirks mode.
    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.quirks_mode = mode;
    }

    // Do two handles refer to the same node?.
    fn same_node(&self, x: &NodeId, y: &NodeId) -> bool {
        *x == *y
    }

    // What is the name of the element?
    // Should never be called on a non-element node; Feel free to `panic!`.
    fn elem_name(&self, target: &NodeId) -> ExpandedName {
        self.tree.query_node(target, |node| match node.data {
            NodeData::Element(Element { .. }) => self.tree.get_name(target).expanded(),
            _ => panic!("not an element!"),
        })
    }

    // Create an element.
    // When creating a template element (`name.ns.expanded() == expanded_name!(html"template")`), an
    // associated document fragment called the "template contents" should also be created. Later calls to
    // self.get_template_contents() with that given element return it. See `the template element in the whatwg spec`,
    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> NodeId {
        let template_contents = if flags.template {
            Some(self.tree.create_node(NodeData::Document))
        } else {
            None
        };

        let id = self.tree.create_node(NodeData::Element(Element::new(
            name.clone(),
            attrs,
            template_contents,
            flags.mathml_annotation_xml_integration_point,
        )));

        self.tree.set_name(id, name);
        id
    }

    // Create a comment node.
    fn create_comment(&mut self, text: StrTendril) -> NodeId {
        self.tree.create_node(NodeData::Comment { contents: text })
    }

    // Create a Processing Instruction node.
    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> NodeId {
        self.tree.create_node(NodeData::ProcessingInstruction {
            target,
            contents: data,
        })
    }

    // Append a node as the last child of the given node. If this would produce adjacent slbling text nodes, it
    // should concatenate the text instead.
    // The child node will not already have a parent.
    fn append(&mut self, parent: &NodeId, child: NodeOrText<NodeId>) {
        // Append to an existing Text node if we have one.

        match child {
            NodeOrText::AppendNode(node_id) => self.tree.append_child_of(parent, &node_id),
            NodeOrText::AppendText(text) => {
                let last_child = self.tree.last_child_of(parent);
                let concated = last_child
                    .map(|child| {
                        self.tree
                            .update_node(&child.id, |node| append_to_existing_text(node, &text))
                    })
                    .unwrap_or(false);

                if concated {
                    return;
                }

                self.tree
                    .append_child_data_of(parent, NodeData::Text { contents: text })
            }
        }
    }

    // Append a node as the sibling immediately before the given node.
    // The tree builder promises that `sibling` is not a text node. However its old previous sibling, which would
    // become the new node's previs sibling, could be a text node. If the new node is also a text node, the two
    // should be merged, as in the behavior of `append`.
    fn append_before_sibling(&mut self, sibling: &NodeId, child: NodeOrText<NodeId>) {
        match child {
            NodeOrText::AppendText(text) => {
                let prev_sibling = self.tree.prev_sibling_of(sibling);
                let concated = prev_sibling
                    .map(|sibling| {
                        self.tree
                            .update_node(&sibling.id, |node| append_to_existing_text(node, &text))
                    })
                    .unwrap_or(false);

                if concated {
                    return;
                }

                let id = self.tree.create_node(NodeData::Text { contents: text });
                self.tree.append_prev_sibling_of(sibling, &id);
            }

            // The tree builder promises we won't have a text node after
            // the insertion point.

            // Any other kind of node.
            NodeOrText::AppendNode(id) => self.tree.append_prev_sibling_of(sibling, &id),
        };
    }

    // When the insertion point is decided by the existence of a parent node of the element, we consider both
    // possibilities and send the element which will be used if a parent node exists, along with the element to be
    // used if there isn't one.
    fn append_based_on_parent_node(
        &mut self,
        element: &NodeId,
        prev_element: &NodeId,
        child: NodeOrText<NodeId>,
    ) {
        let has_parent = self.tree.parent_of(element).is_some();

        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    // Append a `DOCTYPE` element to the `Document` node.
    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        let root = self.tree.root_id();
        self.tree.append_child_data_of(
            &root,
            NodeData::Doctype {
                name,
                public_id,
                system_id,
            },
        );
    }

    // Add each attribute to the given element, if no attribute with that name already exists. The tree builder
    // promises this will never be called with something else than an element.
    fn add_attrs_if_missing(&mut self, target: &NodeId, attrs: Vec<Attribute>) {
        self.tree.update_node(target, |node| {
            let existing = if let NodeData::Element(Element { ref mut attrs, .. }) = node.data {
                attrs
            } else {
                panic!("not an element")
            };
            let existing_names = existing
                .iter()
                .map(|e| e.name.clone())
                .collect::<HashSet<_>>();
            existing.extend(
                attrs
                    .into_iter()
                    .filter(|attr| !existing_names.contains(&attr.name)),
            );
        })
    }

    // Detach the given node from its parent.
    fn remove_from_parent(&mut self, target: &NodeId) {
        self.tree.remove_from_parent(target);
    }

    // Remove all the children from node and append them to new_parent.
    fn reparent_children(&mut self, node: &NodeId, new_parent: &NodeId) {
        self.tree.reparent_children_of(node, Some(*new_parent));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use html5ever::driver::parse_document;
    use tendril::TendrilSink;
    #[test]
    fn test_parse_html_dom() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;

        let dom: Document = Default::default();
        let parser = parse_document(dom, Default::default());
        let _document = parser.one(html);
    }
}
