use crate::document::append_to_existing_text;
use crate::document::Element;
use crate::document::NodeData;
use crate::document::NodeId;
use crate::document::NodeRef;
use crate::document::Tree;
use markup5ever::interface::tree_builder;
use markup5ever::interface::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};

use markup5ever::Attribute;
use markup5ever::ExpandedName;
use markup5ever::QualName;
use std::borrow::Cow;
use std::collections::HashSet;
use tendril::StrTendril;
use tendril::TendrilSink;

use html5ever::parse_document;

pub struct Document {
    pub tree: Tree<NodeData>,
    /// Errors that occurred during parsing.
    pub errors: Vec<Cow<'static, str>>,

    /// The document's quirks mode.
    pub quirks_mode: QuirksMode,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            tree: Tree::new(NodeData::Document),
            errors: vec![],
            quirks_mode: tree_builder::NoQuirks,
        }
    }
}

impl Document {
    pub fn root(&self) -> NodeRef<NodeData> {
        self.tree.root()
    }
}

impl Document {
    pub fn from_str(html: &str) -> Document {
        parse_document(Document::default(), Default::default()).one(html)
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
        if let NodeData::Element(Element {
            template_contents: Some(ref contents),
            ..
        }) = self.tree.get(target).unwrap().node.data
        {
            contents.clone()
        } else {
            panic!("not a template element!")
        }
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
        match self.tree.node(target).data {
            NodeData::Element(Element { ref name, .. }) => name.expanded(),
            _ => panic!("not an element!"),
        }
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
            Some(self.tree.new_node(NodeData::Document))
        } else {
            None
        };

        self.tree.new_node(NodeData::Element(Element::new(
            name,
            attrs,
            template_contents,
            flags.mathml_annotation_xml_integration_point,
        )))
    }

    // Create a comment node.
    fn create_comment(&mut self, text: StrTendril) -> NodeId {
        self.tree.new_node(NodeData::Comment { contents: text })
    }

    // Create a Processing Instruction node.
    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> NodeId {
        self.tree.new_node(NodeData::ProcessingInstruction {
            target: target,
            contents: data,
        })
    }

    // Append a node as the last child of the given node. If this would produce adjacent slbling text nodes, it
    // should concatenate the text instead.
    // The child node will not already have a parent.
    fn append(&mut self, parent: &NodeId, child: NodeOrText<NodeId>) {
        // Append to an existing Text node if we have one.

        let mut parent = self.tree.get_unchecked_mut(parent);
        match child {
            NodeOrText::AppendNode(node_id) => parent.append(&node_id),
            NodeOrText::AppendText(text) => {
                if let Some(mut last_child) = parent.last_child() {
                    if append_to_existing_text(last_child.node(), &text) {
                        return;
                    }
                }

                parent.append_with_data(NodeData::Text { contents: text });
            }
        }
    }

    // Append a node as the sibling immediately before the given node.
    // The tree builder promises that `sibling` is not a text node. However its old previous sibling, which would
    // become the new node's previs sibling, could be a text node. If the new node is also a text node, the two
    // should be merged, as in the behavior of `append`.
    fn append_before_sibling(&mut self, sibling: &NodeId, child: NodeOrText<NodeId>) {
        let mut sibling = self.tree.get_unchecked_mut(sibling);

        match child {
            NodeOrText::AppendText(text) => {
                if let Some(mut prev_sibling_node) = sibling.prev_sibling() {
                    if append_to_existing_text(prev_sibling_node.node(), &text) {
                        return;
                    }
                }

                // No previous node.
                sibling.append_with_data(NodeData::Text { contents: text });
            }

            // The tree builder promises we won't have a text node after
            // the insertion point.

            // Any other kind of node.
            NodeOrText::AppendNode(node) => sibling.append(&node),
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
        let has_parent = self.tree.get_unchecked_mut(element).parent().is_some();

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
        let id = self.tree.new_node(NodeData::Doctype {
            name: name,
            public_id: public_id,
            system_id: system_id,
        });

        self.tree.root_mut().append(&id);
    }

    // Add each attribute to the given element, if no attribute with that name already exists. The tree builder
    // promises this will never be called with something else than an element.
    fn add_attrs_if_missing(&mut self, target: &NodeId, attrs: Vec<Attribute>) {
        let existing = if let NodeData::Element(Element { ref mut attrs, .. }) =
            self.tree.node_mut(target).data
        {
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
    }

    // Detach the given node from its parent.
    fn remove_from_parent(&mut self, target: &NodeId) {
        self.tree.get_unchecked_mut(target).remove_from_parent();
    }

    // Remove all the children from node and append them to new_parent.
    fn reparent_children(&mut self, node: &NodeId, new_parent: &NodeId) {
        self.tree
            .get_unchecked_mut(node)
            .reparent_children(new_parent);
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
