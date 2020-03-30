use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use markup5ever::serialize::TraversalScope;
use markup5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use markup5ever::serialize::{Serialize, Serializer};
use markup5ever::Attribute;
use markup5ever::QualName;
use std::fmt::{self, Debug};
use std::io;
use tendril::StrTendril;

#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub struct NodeId {
    value: usize,
}

impl NodeId {
    fn new(value: usize) -> Self {
        NodeId { value }
    }
}

impl NodeId {}

pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T: Debug> Debug for Tree<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Tree")
            .field("nodes", &self.nodes)
            .finish()
    }
}

impl<T> Tree<T> {
    pub fn root_id(&self) -> NodeId {
        NodeId { value: 0 }
    }

    pub fn new(root: T) -> Self {
        Self {
            nodes: vec![Node::new(root)],
        }
    }

    pub fn nodes(&self) -> &[Node<T>] {
        &self.nodes
    }

    pub fn get(&self, id: &NodeId) -> Option<NodeRef<T>> {
        self.nodes.get(id.value).map(|node| NodeRef {
            id: *id,
            node,
            tree: self,
        })
    }

    pub fn get_unchecked(&self, id: &NodeId) -> NodeRef<T> {
        NodeRef {
            id: *id,
            node: self.node(id),
            tree: self,
        }
    }

    pub fn get_mut(&mut self, id: &NodeId) -> Option<NodeRefMut<T>> {
        let option = self.nodes.get_mut(id.value).map(|_| ());
        option.map(move |_| NodeRefMut {
            id: *id,
            tree: self,
        })
    }

    pub fn get_unchecked_mut(&mut self, id: &NodeId) -> NodeRefMut<T> {
        NodeRefMut {
            id: *id,
            tree: self,
        }
    }

    pub fn new_node(&mut self, data: T) -> NodeId {
        let node_id = NodeId::new(self.nodes.len());
        self.nodes.push(Node::new(data));
        node_id
    }

    pub fn node_mut(&mut self, id: &NodeId) -> &mut Node<T> {
        unsafe { self.nodes.get_unchecked_mut(id.value) }
    }

    pub fn node(&self, id: &NodeId) -> &Node<T> {
        unsafe { self.nodes.get_unchecked(id.value) }
    }

    pub fn root_mut(&mut self) -> NodeRefMut<T> {
        self.get_unchecked_mut(&NodeId::new(0))
    }

    pub fn root(&self) -> NodeRef<T> {
        self.get_unchecked(&NodeId::new(0))
    }
}

pub struct Node<T> {
    pub parent: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub data: T,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Node {
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            data,
        }
    }
}

impl Node<NodeData> {
    pub fn is_document(&self) -> bool {
        match self.data {
            NodeData::Document => true,
            _ => false,
        }
    }

    pub fn is_element(&self) -> bool {
        match self.data {
            NodeData::Element(_) => true,
            _ => false,
        }
    }

    pub fn is_text(&self) -> bool {
        match self.data {
            NodeData::Text { .. } => true,
            _ => false,
        }
    }
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Node")
            .field("parnet", &self.parent)
            .field("prev_sibling", &self.prev_sibling)
            .field("next_sibling", &self.next_sibling)
            .field("first_child", &self.first_child)
            .field("last_child", &self.last_child)
            .field("data", &self.data)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct NodeRef<'a, T> {
    pub id: NodeId,
    pub node: &'a Node<T>,
    pub tree: &'a Tree<T>,
}

impl<'a, T> NodeRef<'a, T> {
    pub fn new(id: usize, node: &'a Node<T>, tree: &'a Tree<T>) -> Self {
        Self {
            id: NodeId::new(id),
            node,
            tree,
        }
    }

    pub fn children(&self) -> Vec<NodeRef<'a, T>> {
        let first_child_id = self.node.first_child;
        let mut next_child_id = first_child_id;
        let mut children = vec![];
        while let Some(id) = next_child_id {
            let node = self.tree.get_unchecked(&id);
            next_child_id = node.node.next_sibling;
            children.push(node);
        }

        children
    }

    pub fn first_child(&self) -> Option<NodeRef<T>> {
        self.node.first_child.map(|id| self.tree.get_unchecked(&id))
    }

    pub fn parent(&self) -> Option<Self> {
        self.node.parent.map(|id| self.tree.get_unchecked(&id))
    }
}

impl<'a> NodeRef<'a, NodeData> {
    pub fn to_html(&self) -> StrTendril {
        let inner: SerializableNodeRef = self.clone().into();

        let mut result = vec![];
        serialize(
            &mut result,
            &inner,
            SerializeOpts {
                scripting_enabled: true,
                traversal_scope: TraversalScope::IncludeNode,
                create_missing_parent: false,
            },
        )
        .unwrap();
        StrTendril::try_from_byte_slice(&result).unwrap()
    }
}

pub struct NodeRefMut<'a, T> {
    pub id: NodeId,
    pub tree: &'a mut Tree<T>,
}

impl<'a, T> NodeRefMut<'a, T> {
    pub fn node(&mut self) -> &mut Node<T> {
        self.tree.node_mut(&self.id)
    }

    pub fn last_child(&mut self) -> Option<NodeRefMut<T>> {
        self.node()
            .last_child
            .map(move |id| self.tree.get_unchecked_mut(&id))
    }

    pub fn first_child(&mut self) -> Option<NodeRefMut<T>> {
        self.node()
            .first_child
            .map(move |id| self.tree.get_unchecked_mut(&id))
    }

    pub fn prev_sibling(&mut self) -> Option<NodeRefMut<T>> {
        self.node()
            .prev_sibling
            .map(move |id| self.tree.get_unchecked_mut(&id))
    }

    pub fn next_sibling(&mut self) -> Option<NodeRefMut<T>> {
        self.node()
            .next_sibling
            .map(move |id| self.tree.get_unchecked_mut(&id))
    }

    pub fn parent(&mut self) -> Option<NodeRefMut<T>> {
        self.node()
            .parent
            .map(move |id| self.tree.get_unchecked_mut(&id))
    }

    pub fn append_with_data(&mut self, data: T) {
        let new_child_id = self.tree.new_node(data);
        self.append(&new_child_id)
    }

    pub fn append(&mut self, new_child_id: &NodeId) {
        let id = self.id;
        let last_child_id = self.node().last_child;

        {
            let mut new_child = self.tree.node_mut(new_child_id);
            new_child.parent = Some(id);
            new_child.prev_sibling = last_child_id;
        }

        if let Some(id) = last_child_id {
            let mut last_child = self.tree.node_mut(&id);
            last_child.next_sibling = Some(*new_child_id)
        } else {
            self.node().first_child = Some(*new_child_id);
        }

        self.node().last_child = Some(*new_child_id);
    }

    pub fn remove_from_parent(&mut self) {
        let id = self.id;
        let node = self.node();
        let parent_id = node.parent;
        let prev_sibling_id = node.prev_sibling;
        let next_sibling_id = node.next_sibling;

        node.parent = None;
        node.next_sibling = None;
        node.prev_sibling = None;

        if let Some(parent_id) = parent_id {
            let parent_node = self.tree.node_mut(&parent_id);
            if parent_node.first_child == Some(id) {
                parent_node.first_child = next_sibling_id;
            }

            if parent_node.last_child == Some(id) {
                parent_node.last_child = prev_sibling_id;
            }
        }

        if let Some(prev_sibling_id) = prev_sibling_id {
            let prev_sibling_node = self.tree.node_mut(&prev_sibling_id);
            prev_sibling_node.next_sibling = next_sibling_id;
        }

        if let Some(next_sibling_id) = next_sibling_id {
            let next_sibling_node = self.tree.node_mut(&next_sibling_id);
            next_sibling_node.prev_sibling = prev_sibling_id;
        }
    }

    pub fn insert_prev_slibing(&mut self, sibling_id: &NodeId) {
        let id = self.id;
        let node = self.node();
        let parent_id = node.parent;
        let prev_sibling_id = node.prev_sibling;
        node.prev_sibling = Some(*sibling_id);

        let mut new_sibling = self.tree.get_unchecked_mut(sibling_id);
        new_sibling.remove_from_parent();
        let new_sibling_node = new_sibling.node();
        new_sibling_node.prev_sibling = prev_sibling_id;
        new_sibling_node.next_sibling = Some(id);

        if let Some(parent_id) = parent_id {
            let parent_node = self.tree.node_mut(&parent_id);
            if parent_node.first_child == Some(id) {
                parent_node.first_child = Some(*sibling_id);
            }
        }

        if let Some(prev_sibling_id) = prev_sibling_id {
            let prev_sibling_node = self.tree.node_mut(&prev_sibling_id);
            prev_sibling_node.next_sibling = Some(*sibling_id);
        }
    }

    pub fn reparent_children(&mut self, new_parent_id: &NodeId) {
        let node = self.node();
        let first_child_id = node.first_child;
        let last_child_id = node.last_child;

        node.first_child = None;
        node.last_child = None;

        {
            let new_parent = self.tree.node_mut(new_parent_id);
            new_parent.first_child = first_child_id;
            new_parent.last_child = last_child_id;
        }

        let mut next_child_id = first_child_id;
        while let Some(id) = next_child_id {
            let node = self.tree.node_mut(&id);
            node.parent = Some(*new_parent_id);
            next_child_id = node.next_sibling;
        }
    }
}

/// The different kinds of nodes in the DOM.
#[derive(Debug, Clone)]
pub enum NodeData {
    /// The `Tree` itself - the root node of a HTML tree.
    Document,

    /// A `DOCTYPE` with name, public id, and system id. See
    /// [tree type declaration on wikipedia][dtd wiki].
    ///
    /// [dtd wiki]: https://en.wikipedia.org/wiki/Tree_type_declaration
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
    Element(Element),

    /// A Processing instruction.
    ProcessingInstruction {
        target: StrTendril,
        contents: StrTendril,
    },
}

/// An element with attributes.
#[derive(Debug, Clone)]
pub struct Element {
    pub name: QualName,
    pub attrs: Vec<Attribute>,

    /// For HTML \<template\> elements, the [template contents].
    ///
    /// [template contents]: https://html.spec.whatwg.org/multipage/#template-contents
    pub template_contents: Option<NodeId>,

    /// Whether the node is a [HTML integration point].
    ///
    /// [HTML integration point]: https://html.spec.whatwg.org/multipage/#html-integration-point
    mathml_annotation_xml_integration_point: bool,
}

impl Element {
    pub fn new(
        name: QualName,
        attrs: Vec<Attribute>,
        template_contents: Option<NodeId>,
        mathml_annotation_xml_integration_point: bool,
    ) -> Self {
        Element {
            name,
            attrs,
            template_contents,
            mathml_annotation_xml_integration_point,
        }
    }
}

enum SerializeOp<'a> {
    Open(NodeRef<'a, NodeData>),
    Close(QualName),
}

pub struct SerializableNodeRef<'a>(NodeRef<'a, NodeData>);

impl<'a> From<NodeRef<'a, NodeData>> for SerializableNodeRef<'a> {
    fn from(h: NodeRef<'a, NodeData>) -> SerializableNodeRef {
        SerializableNodeRef(h)
    }
}

impl<'a> Serialize for SerializableNodeRef<'a> {
    fn serialize<S>(&self, serializer: &mut S, traversal_scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        let mut ops = match traversal_scope {
            IncludeNode => vec![SerializeOp::Open(self.0.clone())],
            ChildrenOnly(_) => self
                .0
                .children()
                .into_iter()
                .map(|h| SerializeOp::Open(h))
                .collect(),
        };

        while !ops.is_empty() {
            match ops.remove(0) {
                SerializeOp::Open(node_ref) => match &node_ref.node.data {
                    &NodeData::Element(ref e) => {
                        serializer.start_elem(
                            e.name.clone(),
                            e.attrs.iter().map(|at| (&at.name, &at.value[..])),
                        )?;

                        ops.insert(0, SerializeOp::Close(e.name.clone()));

                        for child in node_ref.children().iter().rev() {
                            ops.insert(0, SerializeOp::Open(child.clone()));
                        }
                    }

                    &NodeData::Doctype { ref name, .. } => serializer.write_doctype(&name)?,

                    &NodeData::Text { ref contents } => serializer.write_text(&contents)?,

                    &NodeData::Comment { ref contents } => serializer.write_comment(&contents)?,

                    &NodeData::ProcessingInstruction {
                        ref target,
                        ref contents,
                    } => serializer.write_processing_instruction(target, contents)?,

                    &NodeData::Document => continue,
                },

                SerializeOp::Close(name) => {
                    serializer.end_elem(name)?;
                }
            }
        }

        Ok(())
    }
}
