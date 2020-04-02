use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use markup5ever::serialize::TraversalScope;
use markup5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use markup5ever::serialize::{Serialize, Serializer};
use markup5ever::Attribute;
use markup5ever::QualName;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::io;
use tendril::StrTendril;

pub type Node<'a> = NodeRef<'a, NodeData>;

pub(crate) fn append_to_existing_text(prev: &mut InnerNode<NodeData>, text: &str) -> bool {
    match prev.data {
        NodeData::Text { ref mut contents } => {
            contents.push_slice(text);
            true
        }
        _ => false,
    }
}

#[derive(Copy, Debug, Clone, Eq, PartialEq, Hash)]
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
    nodes: Cell<Vec<InnerNode<T>>>,
    names: HashMap<NodeId, QualName>,
}

impl<T: Debug> Debug for Tree<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Tree").finish()
    }
}

macro_rules! children_of {
    ($nodes: expr, $id: expr) => {{
        let node = unsafe { $nodes.get_unchecked($id.value) };
        let first_child_id = node.first_child;
        let mut next_child_id = first_child_id;

        let mut children = vec![];

        while let Some(id) = next_child_id {
            let node = unsafe { $nodes.get_unchecked(id.value) };
            next_child_id = node.next_sibling;
            children.push(id);
        }

        children
    }};
}

impl<T> Tree<T> {
    pub fn root_id(&self) -> NodeId {
        NodeId { value: 0 }
    }

    pub fn new(root: T) -> Self {
        Self {
            nodes: Cell::new(vec![InnerNode::new(root)]),
            names: HashMap::new(),
        }
    }

    pub fn create_node(&self, data: T) -> NodeId {
        let mut nodes = self.nodes.take();
        let new_child_id = NodeId::new(nodes.len());

        nodes.push(InnerNode::new(data));
        self.nodes.set(nodes);
        new_child_id
    }

    pub fn set_name(&mut self, id: NodeId, name: QualName) {
        self.names.insert(id, name);
    }

    pub fn get_name(&self, id: &NodeId) -> &QualName {
        self.names.get(id).unwrap()
    }

    pub fn get(&self, id: &NodeId) -> Option<NodeRef<T>> {
        let nodes = self.nodes.take();
        let node = nodes.get(id.value).map(|node| NodeRef {
            id: *id,
            tree: self,
        });

        self.nodes.set(nodes);
        node
    }

    pub fn get_unchecked(&self, id: &NodeId) -> NodeRef<T> {
        NodeRef {
            id: *id,
            tree: self,
        }
    }

    pub fn root(&self) -> NodeRef<T> {
        self.get_unchecked(&NodeId::new(0))
    }

    pub fn children_of(&self, id: &NodeId) -> Vec<NodeRef<T>> {
        let nodes = self.nodes.take();
        let children_ids = children_of!(&nodes, id);
        self.nodes.set(nodes);

        children_ids
            .into_iter()
            .map(|id| NodeRef::new(id, self))
            .collect()
    }

    pub fn first_child_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        let nodes = self.nodes.take();
        let node = unsafe { nodes.get_unchecked(id.value) };
        let target = node.first_child.map(|id| NodeRef { id, tree: self });

        self.nodes.set(nodes);
        target
    }

    pub fn last_child_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        let nodes = self.nodes.take();
        let node = unsafe { nodes.get_unchecked(id.value) };
        let target = node.last_child.map(|id| NodeRef { id, tree: self });

        self.nodes.set(nodes);
        target
    }

    pub fn parent_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        let nodes = self.nodes.take();
        let node = unsafe { nodes.get_unchecked(id.value) };
        let target = node.parent.map(|id| NodeRef { id, tree: self });

        self.nodes.set(nodes);
        target
    }

    pub fn prev_sibling_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        let nodes = self.nodes.take();
        let node = unsafe { nodes.get_unchecked(id.value) };
        let target = node.prev_sibling.map(|id| NodeRef { id, tree: self });

        self.nodes.set(nodes);
        target
    }

    pub fn next_sibling_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        let nodes = self.nodes.take();
        let node = unsafe { nodes.get_unchecked(id.value) };
        let target = node.next_sibling.map(|id| NodeRef { id, tree: self });

        self.nodes.set(nodes);
        target
    }

    pub fn append_child_data_of(&self, id: &NodeId, data: T) {
        let mut nodes = self.nodes.take();
        let mut last_child_id = None;

        {
            let parent = unsafe { nodes.get_unchecked(id.value) };
            last_child_id = parent.last_child;
        }

        let new_child_id = NodeId::new(nodes.len());
        let mut child = InnerNode::new(data);
        child.prev_sibling = last_child_id;
        child.parent = Some(*id);
        nodes.push(child);

        if let Some(id) = last_child_id {
            let last_child = unsafe { nodes.get_unchecked_mut(id.value) };
            last_child.next_sibling = Some(new_child_id);
        }

        let parent = unsafe { nodes.get_unchecked_mut(id.value) };
        if parent.first_child.is_none() {
            parent.first_child = Some(new_child_id);
        }

        parent.last_child = Some(new_child_id);

        self.nodes.set(nodes);
    }

    pub fn append_child_of(&self, id: &NodeId, new_child_id: &NodeId) {
        let mut nodes = self.nodes.take();
        let mut last_child_id = None;
        {
            let parent = unsafe { nodes.get_unchecked_mut(id.value) };
            last_child_id = parent.last_child;
        }

        if let Some(id) = last_child_id {
            let last_child = unsafe { nodes.get_unchecked_mut(id.value) };
            last_child.next_sibling = Some(*new_child_id);
        }

        let parent = unsafe { nodes.get_unchecked_mut(id.value) };
        if last_child_id.is_none() {
            parent.first_child = Some(*new_child_id);
        }

        parent.last_child = Some(*new_child_id);

        let child = unsafe { nodes.get_unchecked_mut(new_child_id.value) };
        child.prev_sibling = last_child_id;
        child.parent = Some(*id);

        self.nodes.set(nodes);
    }

    pub fn remove_from_parent(&self, id: &NodeId) {
        let mut nodes = self.nodes.take();
        let node = unsafe { nodes.get_unchecked_mut(id.value) };
        let parent_id = node.parent;
        let prev_sibling_id = node.prev_sibling;
        let next_sibling_id = node.next_sibling;

        node.parent = None;
        node.next_sibling = None;
        node.prev_sibling = None;

        if let Some(parent_id) = parent_id {
            let parent = unsafe { nodes.get_unchecked_mut(parent_id.value) };
            if parent.first_child == Some(*id) {
                parent.first_child = next_sibling_id;
            }

            if parent.last_child == Some(*id) {
                parent.last_child = prev_sibling_id;
            }
        }

        if let Some(prev_sibling_id) = prev_sibling_id {
            let prev_sibling = unsafe { nodes.get_unchecked_mut(prev_sibling_id.value) };
            prev_sibling.next_sibling = next_sibling_id;
        }

        if let Some(next_sibling_id) = next_sibling_id {
            let next_sibling = unsafe { nodes.get_unchecked_mut(next_sibling_id.value) };
            next_sibling.prev_sibling = prev_sibling_id;
        }

        self.nodes.set(nodes);
    }

    pub fn set_prev_sibling_of(&self, id: &NodeId, new_sibling_id: &NodeId) {
        self.remove_from_parent(new_sibling_id);

        let mut nodes = self.nodes.take();
        let node = unsafe { nodes.get_unchecked_mut(id.value) };

        let parent_id = node.parent;
        let prev_sibling_id = node.prev_sibling;

        node.prev_sibling = Some(*new_sibling_id);

        let new_sibling = unsafe { nodes.get_unchecked_mut(new_sibling_id.value) };
        new_sibling.parent = parent_id;
        new_sibling.prev_sibling = prev_sibling_id;
        new_sibling.next_sibling = Some(*id);

        if let Some(parent_id) = parent_id {
            let parent = unsafe { nodes.get_unchecked_mut(parent_id.value) };
            if parent.first_child == Some(*id) {
                parent.first_child = Some(*new_sibling_id);
            }
        }

        if let Some(prev_sibling_id) = prev_sibling_id {
            let prev_sibling = unsafe { nodes.get_unchecked_mut(prev_sibling_id.value) };
            prev_sibling.next_sibling = Some(*new_sibling_id);
        }

        self.nodes.set(nodes);
    }

    pub fn reparent_children_of(&self, id: &NodeId, new_parent_id: &NodeId) {
        let mut nodes = self.nodes.take();
        let node = unsafe { nodes.get_unchecked_mut(id.value) };

        let first_child_id = node.first_child;
        let last_child_id = node.last_child;
        node.first_child = None;
        node.last_child = None;

        let mut new_parent = unsafe { nodes.get_unchecked_mut(new_parent_id.value) };
        new_parent.first_child = first_child_id;
        new_parent.last_child = last_child_id;

        let mut next_child_id = first_child_id;
        while let Some(child_id) = next_child_id {
            let child = unsafe { nodes.get_unchecked_mut(child_id.value) };
            child.parent = Some(*new_parent_id);
            next_child_id = child.next_sibling;
        }

        self.nodes.set(nodes);
    }

    pub fn query_node<F, B>(&self, id: &NodeId, f: F) -> B
    where
        F: FnOnce(&InnerNode<T>) -> B,
    {
        let nodes = self.nodes.take();
        let r = f(unsafe { nodes.get_unchecked(id.value) });
        self.nodes.set(nodes);
        r
    }

    pub fn update_node<F, B>(&self, id: &NodeId, f: F) -> B
    where
        F: FnOnce(&mut InnerNode<T>) -> B,
    {
        let mut nodes = self.nodes.take();
        let r = f(unsafe { nodes.get_unchecked_mut(id.value) });
        self.nodes.set(nodes);
        r
    }

    pub fn compare_node<F, B>(&self, a: &NodeId, b: &NodeId, f: F) -> B
    where
        F: FnOnce(&InnerNode<T>, &InnerNode<T>) -> B,
    {
        let nodes = self.nodes.take();
        let node_a = unsafe { nodes.get_unchecked(a.value) };
        let node_b = unsafe { nodes.get_unchecked(b.value) };

        let r = f(node_a, node_b);
        self.nodes.set(nodes);
        r
    }
}

pub struct InnerNode<T> {
    pub parent: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub data: T,
}

impl<T> InnerNode<T> {
    fn new(data: T) -> Self {
        InnerNode {
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            data,
        }
    }
}

impl<T: Debug> Debug for InnerNode<T> {
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
    pub tree: &'a Tree<T>,
}

impl<'a, T> NodeRef<'a, T> {
    pub fn new(id: NodeId, tree: &'a Tree<T>) -> Self {
        Self { id, tree }
    }

    pub fn inner_query<F, B>(&self, f: F) -> B
    where
        F: FnOnce(&InnerNode<T>) -> B,
    {
        self.tree.query_node(&self.id, f)
    }

    pub fn parent(&self) -> Option<Self> {
        self.tree.parent_of(&self.id)
    }

    pub fn children(&self) -> Vec<Self> {
        self.tree.children_of(&self.id)
    }

    pub fn first_child(&self) -> Option<Self> {
        self.tree.first_child_of(&self.id)
    }

    pub fn remove_from_parent(&self) {
        self.tree.remove_from_parent(&self.id)
    }
}

impl<'a> NodeRef<'a, NodeData> {
    pub fn is_document(&self) -> bool {
        self.inner_query(|node| match node.data {
            NodeData::Document => true,
            _ => false,
        })
    }

    pub fn is_element(&self) -> bool {
        self.inner_query(|node| match node.data {
            NodeData::Element(_) => true,
            _ => false,
        })
    }

    pub fn is_text(&self) -> bool {
        self.inner_query(|node| match node.data {
            NodeData::Text { .. } => true,
            _ => false,
        })
    }
}

impl<'a> NodeRef<'a, NodeData> {
    pub fn html(&self) -> StrTendril {
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

    pub fn text(&self) -> StrTendril {
        let mut ops = vec![self.id];
        let mut text = StrTendril::new();
        let nodes = self.tree.nodes.take();
        while !ops.is_empty() {
            let id = ops.remove(0);
            let node = unsafe { nodes.get_unchecked(id.value) };
            match node.data {
                NodeData::Element(_) => {
                    for child in children_of!(nodes, id).into_iter().rev() {
                        ops.insert(0, child);
                    }
                }

                NodeData::Text { ref contents } => text.push_tendril(&contents),

                _ => continue,
            }
        }

        self.tree.nodes.set(nodes);

        text
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

enum SerializeOp {
    Open(NodeId),
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
        let nodes = self.0.tree.nodes.take();
        let id = self.0.id;
        let mut ops = match traversal_scope {
            IncludeNode => vec![SerializeOp::Open(id)],
            ChildrenOnly(_) => children_of!(nodes, id)
                .into_iter()
                .map(|h| SerializeOp::Open(h))
                .collect(),
        };

        while !ops.is_empty() {
            if let Err(e) = match ops.remove(0) {
                SerializeOp::Open(id) => match unsafe { &nodes.get_unchecked(id.value).data } {
                    NodeData::Element(ref e) => {
                        serializer.start_elem(
                            e.name.clone(),
                            e.attrs.iter().map(|at| (&at.name, &at.value[..])),
                        )?;

                        ops.insert(0, SerializeOp::Close(e.name.clone()));

                        for child_id in children_of!(nodes, id).into_iter().rev() {
                            ops.insert(0, SerializeOp::Open(child_id));
                        }

                        Ok(())
                    }
                    NodeData::Doctype { ref name, .. } => serializer.write_doctype(&name),
                    NodeData::Text { ref contents } => serializer.write_text(&contents),
                    NodeData::Comment { ref contents } => serializer.write_comment(&contents),
                    NodeData::ProcessingInstruction {
                        ref target,
                        ref contents,
                    } => serializer.write_processing_instruction(target, contents),
                    NodeData::Document => continue,
                },
                SerializeOp::Close(name) => serializer.end_elem(name),
            } {
                self.0.tree.nodes.set(nodes);
                return Err(e);
            }
        }

        self.0.tree.nodes.set(nodes);
        Ok(())
    }
}
