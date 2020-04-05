use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use html5ever::LocalName;
use markup5ever::serialize::TraversalScope;
use markup5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use markup5ever::serialize::{Serialize, Serializer};
use markup5ever::Attribute;
use markup5ever::QualName;
use markup5ever::{namespace_url, ns};
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::io;
use tendril::StrTendril;
pub type Node<'a> = NodeRef<'a, NodeData>;

macro_rules! get_node_unchecked {
    ($nodes: expr, $id: expr) => {
        unsafe { $nodes.get_unchecked($id.value) }
    };
}

macro_rules! get_node_unchecked_mut {
    ($nodes: expr, $id: expr) => {
        unsafe { $nodes.get_unchecked_mut($id.value) }
    };
}

macro_rules! with_cell {
    ($cell: expr, $bind_value: ident, $some_work: block) => {{
        let $bind_value = $cell.take();
        let r = $some_work;
        $cell.set($bind_value);
        r
    }};
}

macro_rules! with_cell_mut {
    ($cell: expr, $bind_value: ident, $some_work: block) => {{
        let mut $bind_value = $cell.take();
        let r = $some_work;
        $cell.set($bind_value);
        r
    }};
}

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

pub struct Tree<T> {
    nodes: Cell<Vec<InnerNode<T>>>,
    names: HashMap<NodeId, QualName>,
}

impl<T: Debug> Debug for Tree<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Tree").finish()
    }
}

impl<T: Clone> Clone for Tree<T> {
    fn clone(&self) -> Self {
        with_cell!(self.nodes, nodes, {
            Self {
                nodes: Cell::new(nodes.clone()),
                names: self.names.clone(),
            }
        })
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

impl<T: Debug> Tree<T> {
    pub fn root_id(&self) -> NodeId {
        NodeId { value: 0 }
    }

    pub fn new(root: T) -> Self {
        let root_id = NodeId::new(0);
        Self {
            nodes: Cell::new(vec![InnerNode::new(root_id, root)]),
            names: HashMap::new(),
        }
    }

    pub fn create_node(&self, data: T) -> NodeId {
        let mut nodes = self.nodes.take();
        let new_child_id = NodeId::new(nodes.len());

        nodes.push(InnerNode::new(new_child_id, data));
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
        let node = nodes.get(id.value).map(|_| NodeRef {
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
        with_cell!(self.nodes, nodes, {
            children_of!(&nodes, id)
                .into_iter()
                .map(|id| NodeRef::new(id, self))
                .collect()
        })
    }

    pub fn first_child_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        with_cell!(self.nodes, nodes, {
            let node = get_node_unchecked!(nodes, id);
            node.first_child.map(|id| NodeRef { id, tree: self })
        })
    }

    pub fn last_child_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        with_cell!(self.nodes, nodes, {
            let node = get_node_unchecked!(nodes, id);
            node.last_child.map(|id| NodeRef { id, tree: self })
        })
    }

    pub fn parent_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        with_cell!(self.nodes, nodes, {
            let node = get_node_unchecked!(nodes, id);
            node.parent.map(|id| NodeRef { id, tree: self })
        })
    }

    pub fn prev_sibling_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        with_cell!(self.nodes, nodes, {
            let node = get_node_unchecked!(nodes, id);
            node.prev_sibling.map(|id| NodeRef { id, tree: self })
        })
    }

    pub fn next_sibling_of(&self, id: &NodeId) -> Option<NodeRef<T>> {
        with_cell!(self.nodes, nodes, {
            let node = get_node_unchecked!(nodes, id);
            node.next_sibling.map(|id| NodeRef { id, tree: self })
        })
    }

    pub fn append_child_data_of(&self, id: &NodeId, data: T) {
        with_cell_mut!(self.nodes, nodes, {
            let mut last_child_id = None;

            {
                let parent = get_node_unchecked!(nodes, id);
                last_child_id = parent.last_child;
            }

            let new_child_id = NodeId::new(nodes.len());
            let mut child = InnerNode::new(new_child_id, data);
            child.prev_sibling = last_child_id;
            child.parent = Some(*id);
            nodes.push(child);

            if let Some(id) = last_child_id {
                let last_child = get_node_unchecked_mut!(nodes, id);
                last_child.next_sibling = Some(new_child_id);
            }

            let parent = get_node_unchecked_mut!(nodes, id);
            if parent.first_child.is_none() {
                parent.first_child = Some(new_child_id);
            }

            parent.last_child = Some(new_child_id);
        })
    }

    pub fn append_child_of(&self, id: &NodeId, new_child_id: &NodeId) {
        with_cell_mut!(self.nodes, nodes, {
            let mut last_child_id = None;
            {
                let parent = get_node_unchecked_mut!(nodes, id);
                last_child_id = parent.last_child;
            }

            if let Some(id) = last_child_id {
                let last_child = get_node_unchecked_mut!(nodes, id);
                last_child.next_sibling = Some(*new_child_id);
            }

            let parent = get_node_unchecked_mut!(nodes, id);
            if last_child_id.is_none() {
                parent.first_child = Some(*new_child_id);
            }

            parent.last_child = Some(*new_child_id);

            let child = get_node_unchecked_mut!(nodes, new_child_id);
            child.prev_sibling = last_child_id;
            child.parent = Some(*id);
        })
    }

    pub fn append_children_from_another_tree(&self, id: &NodeId, tree: Tree<T>) {
        with_cell_mut!(self.nodes, nodes, {
            let mut new_nodes = tree.nodes.take();
            assert!(
                new_nodes.len() > 0,
                "The tree should have at leaset one root node"
            );
            assert!(
                nodes.len() > 0,
                "The tree should have at leaset one root node"
            );

            let offset = nodes.len();

            // `parse_fragment` returns a document that looks like:
            // <:root>                     id -> 0
            //  <body>                     id -> 1
            //      <html>                 id -> 2
            //          things we need.
            //      </html>
            //  </body>
            // <:root>
            const TRUE_ROOT_ID: usize = 2;
            let root = get_node_unchecked!(new_nodes, NodeId::new(TRUE_ROOT_ID));

            macro_rules! fix_id {
                ($id: expr) => {
                    $id.map(|old| NodeId::new(old.value + offset))
                };
            }

            let first_child_id = fix_id!(root.first_child);
            let last_child_id = fix_id!(root.last_child);

            // Update new parent's first and last child id.
            let mut parent = get_node_unchecked_mut!(nodes, id);
            if parent.first_child.is_none() {
                parent.first_child = first_child_id;
            }

            let parent_last_child_id = parent.last_child;
            parent.last_child = last_child_id;

            // Update next_sibling_id
            if let Some(last_child_id) = parent_last_child_id {
                let mut last_child = get_node_unchecked_mut!(nodes, last_child_id);
                last_child.next_sibling = first_child_id;
            }

            let mut first_valid_child = false;

            // Fix nodes's ref id.
            for node in &mut new_nodes {
                node.parent = node.parent.and_then(|parent_id| match parent_id.value {
                    i if i < TRUE_ROOT_ID => None,
                    i if i == TRUE_ROOT_ID => Some(*id),
                    i @ _ => fix_id!(Some(NodeId::new(i))),
                });

                // Update prev_sibling_id
                if !first_valid_child && node.parent == Some(*id) {
                    first_valid_child = true;

                    node.prev_sibling = parent_last_child_id;
                }

                node.id = fix_id!(node.id);
                node.prev_sibling = fix_id!(node.prev_sibling);
                node.next_sibling = fix_id!(node.next_sibling);
                node.first_child = fix_id!(node.first_child);
                node.last_child = fix_id!(node.last_child);
            }

            // Put all the new nodes except the root node into the nodes.
            nodes.extend(new_nodes);
        })
    }

    pub fn append_prev_siblings_from_another_tree(&self, id: &NodeId, tree: Tree<T>) {
        self.debug_nodes();
        with_cell_mut!(self.nodes, nodes, {
            let mut new_nodes = tree.nodes.take();
            assert!(
                new_nodes.len() > 0,
                "The tree should have at leaset one root node"
            );
            assert!(
                nodes.len() > 0,
                "The tree should have at leaset one root node"
            );

            let offset = nodes.len();
            // `parse_fragment` returns a document that looks like:
            // <:root>                     id -> 0
            //  <body>                     id -> 1
            //      <html>                 id -> 2
            //          things we need.
            //      </html>
            //  </body>
            // <:root>
            const TRUE_ROOT_ID: usize = 2;
            let root = get_node_unchecked!(new_nodes, NodeId::new(TRUE_ROOT_ID));
            macro_rules! fix_id {
                ($id: expr) => {
                    $id.map(|old| NodeId::new(old.value + offset))
                };
            }

            let first_child_id = fix_id!(root.first_child);
            let last_child_id = fix_id!(root.last_child);

            let mut node = get_node_unchecked_mut!(nodes, id);
            let prev_sibling_id = node.prev_sibling;
            let parent_id = node.parent;

            // Update node's previous sibling.
            node.prev_sibling = last_child_id;

            // Update prev sibling's next sibling
            if let Some(prev_sibling_id) = prev_sibling_id {
                let mut prev_sibling = get_node_unchecked_mut!(nodes, prev_sibling_id);
                prev_sibling.next_sibling = first_child_id;
            // Update parent's first child.
            } else if let Some(parent_id) = parent_id {
                let mut parent = get_node_unchecked_mut!(nodes, parent_id);
                parent.first_child = first_child_id;
            }

            let mut i = 0;
            let mut last_valid_child = 0;
            let mut first_valid_child = true;
            // Fix nodes's ref id.
            for node in &mut new_nodes {
                node.parent = node
                    .parent
                    .and_then(|old_parent_id| match old_parent_id.value {
                        i if i < TRUE_ROOT_ID => None,
                        i if i == TRUE_ROOT_ID => parent_id,
                        i @ _ => fix_id!(Some(NodeId::new(i))),
                    });

                // Update first child's prev_sibling
                if !first_valid_child && node.parent == Some(*id) {
                    first_valid_child = true;
                    node.prev_sibling = prev_sibling_id;
                }

                if node.parent == parent_id {
                    last_valid_child = i;
                }

                node.id = fix_id!(node.id);
                node.first_child = fix_id!(node.first_child);
                node.last_child = fix_id!(node.last_child);
                node.prev_sibling = fix_id!(node.prev_sibling);
                node.next_sibling = fix_id!(node.next_sibling);
                i += 1;
            }

            // Update last child's next_sibling.
            new_nodes[last_valid_child as usize].next_sibling = Some(*id);

            // Put all the new nodes except the root node into the nodes.
            nodes.extend(new_nodes);
        })
    }

    pub fn remove_from_parent(&self, id: &NodeId) {
        with_cell_mut!(self.nodes, nodes, {
            let node = get_node_unchecked_mut!(nodes, id);
            let parent_id = node.parent;
            let prev_sibling_id = node.prev_sibling;
            let next_sibling_id = node.next_sibling;

            node.parent = None;
            node.next_sibling = None;
            node.prev_sibling = None;

            if let Some(parent_id) = parent_id {
                let parent = get_node_unchecked_mut!(nodes, parent_id);
                if parent.first_child == Some(*id) {
                    parent.first_child = next_sibling_id;
                }

                if parent.last_child == Some(*id) {
                    parent.last_child = prev_sibling_id;
                }
            }

            if let Some(prev_sibling_id) = prev_sibling_id {
                let prev_sibling = get_node_unchecked_mut!(nodes, prev_sibling_id);
                prev_sibling.next_sibling = next_sibling_id;
            }

            if let Some(next_sibling_id) = next_sibling_id {
                let next_sibling = get_node_unchecked_mut!(nodes, next_sibling_id);
                next_sibling.prev_sibling = prev_sibling_id;
            }
        })
    }

    pub fn append_prev_sibling_of(&self, id: &NodeId, new_sibling_id: &NodeId) {
        self.remove_from_parent(new_sibling_id);

        with_cell_mut!(self.nodes, nodes, {
            let node = get_node_unchecked_mut!(nodes, id);

            let parent_id = node.parent;
            let prev_sibling_id = node.prev_sibling;

            node.prev_sibling = Some(*new_sibling_id);

            let new_sibling = get_node_unchecked_mut!(nodes, new_sibling_id);
            new_sibling.parent = parent_id;
            new_sibling.prev_sibling = prev_sibling_id;
            new_sibling.next_sibling = Some(*id);

            if let Some(parent_id) = parent_id {
                let parent = get_node_unchecked_mut!(nodes, parent_id);
                if parent.first_child == Some(*id) {
                    parent.first_child = Some(*new_sibling_id);
                }
            }

            if let Some(prev_sibling_id) = prev_sibling_id {
                let prev_sibling = get_node_unchecked_mut!(nodes, prev_sibling_id);
                prev_sibling.next_sibling = Some(*new_sibling_id);
            }
        })
    }

    pub fn reparent_children_of(&self, id: &NodeId, new_parent_id: Option<NodeId>) {
        with_cell_mut!(self.nodes, nodes, {
            let node = get_node_unchecked_mut!(nodes, id);

            let first_child_id = node.first_child;
            let last_child_id = node.last_child;
            node.first_child = None;
            node.last_child = None;

            if let Some(new_parent_id) = new_parent_id {
                let mut new_parent = get_node_unchecked_mut!(nodes, new_parent_id);
                new_parent.first_child = first_child_id;
                new_parent.last_child = last_child_id;
            }
            let mut next_child_id = first_child_id;
            while let Some(child_id) = next_child_id {
                let child = get_node_unchecked_mut!(nodes, child_id);
                child.parent = new_parent_id;
                next_child_id = child.next_sibling;
            }
        })
    }

    pub fn debug_nodes(&self) {
        with_cell!(self.nodes, nodes, {
            println!("==============");
            for node in &nodes {
                println!("{:?}", node);
            }

            println!("==============");
        })
    }

    pub fn remove_children_of(&self, id: &NodeId) {
        self.reparent_children_of(id, None)
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
    pub id: Option<NodeId>,
    pub parent: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub data: T,
}

impl<T> InnerNode<T> {
    fn new(id: NodeId, data: T) -> Self {
        InnerNode {
            id: Some(id),
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
            .field("id", &self.id)
            .field("parnet", &self.parent)
            .field("prev_sibling", &self.prev_sibling)
            .field("next_sibling", &self.next_sibling)
            .field("first_child", &self.first_child)
            .field("last_child", &self.last_child)
            .field("data", &self.data)
            .finish()
    }
}

impl InnerNode<NodeData> {
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

impl<T: Clone> Clone for InnerNode<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            parent: self.parent.clone(),
            prev_sibling: self.prev_sibling.clone(),
            next_sibling: self.next_sibling.clone(),
            first_child: self.first_child.clone(),
            last_child: self.last_child.clone(),
            data: self.data.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeRef<'a, T> {
    pub id: NodeId,
    pub tree: &'a Tree<T>,
}

impl<'a, T: Debug> NodeRef<'a, T> {
    pub fn new(id: NodeId, tree: &'a Tree<T>) -> Self {
        Self { id, tree }
    }

    pub fn query<F, B>(&self, f: F) -> B
    where
        F: FnOnce(&InnerNode<T>) -> B,
    {
        self.tree.query_node(&self.id, f)
    }

    pub fn update<F, B>(&self, f: F) -> B
    where
        F: FnOnce(&mut InnerNode<T>) -> B,
    {
        self.tree.update_node(&self.id, f)
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

    pub fn next_sibling(&self) -> Option<Self> {
        self.tree.next_sibling_of(&self.id)
    }

    pub fn remove_from_parent(&self) {
        self.tree.remove_from_parent(&self.id)
    }

    pub fn remove_children(&self) {
        self.tree.remove_children_of(&self.id)
    }

    pub fn append_prev_sibling(&self, id: &NodeId) {
        self.tree.append_prev_sibling_of(&self.id, id)
    }

    pub fn append_children_from_another_tree(&self, tree: Tree<T>) {
        self.tree.append_children_from_another_tree(&self.id, tree)
    }

    pub fn append_prev_siblings_from_another_tree(&self, tree: Tree<T>) {
        self.tree
            .append_prev_siblings_from_another_tree(&self.id, tree)
    }
}

impl<'a> Node<'a> {
    pub fn next_element_sibling(&self) -> Option<Self> {
        with_cell!(self.tree.nodes, nodes, {
            let node = get_node_unchecked!(nodes, self.id);
            let mut next_sibling = node.next_sibling;
            while let Some(id) = next_sibling {
                let node = get_node_unchecked!(nodes, id);
                if node.is_element() {
                    return Some(NodeRef::new(id, self.tree));
                }
                next_sibling = node.next_sibling;
            }
            None
        })
    }
}

impl<'a> Node<'a> {
    pub fn node_name(&self) -> Option<StrTendril> {
        self.query(|node| match node.data {
            NodeData::Element(ref e) => {
                let name: &str = &e.name.local;
                Some(StrTendril::from(name))
            }
            _ => None,
        })
    }

    pub fn attr(&self, name: &str) -> Option<StrTendril> {
        self.query(|node| match node.data {
            NodeData::Element(ref e) => e
                .attrs
                .iter()
                .find(|attr| &attr.name.local == name)
                .map(|attr| attr.value.clone()),
            _ => None,
        })
    }

    pub fn set_attr(&self, name: &str, val: &str) {
        self.update(|node| {
            if let NodeData::Element(ref mut e) = node.data {
                let updated = e.attrs.iter_mut().any(|attr| {
                    if &attr.name.local == name {
                        attr.value = StrTendril::from(val);
                        true
                    } else {
                        false
                    }
                });

                if !updated {
                    let value = StrTendril::from(val);
                    // The namespace on the attribute name is almost always ns!().
                    let name = QualName::new(None, ns!(), LocalName::from(name));

                    e.attrs.push(Attribute { name, value })
                }
            }
        })
    }

    pub fn remove_attr(&self, name: &str) {
        self.update(|node| {
            if let NodeData::Element(ref mut e) = node.data {
                e.attrs.retain(|attr| &attr.name.local != name);
            }
        })
    }
}

impl<'a> Node<'a> {
    pub fn is_document(&self) -> bool {
        self.query(|node| node.is_document())
    }

    pub fn is_element(&self) -> bool {
        self.query(|node| node.is_element())
    }

    pub fn is_text(&self) -> bool {
        self.query(|node| node.is_text())
    }
}

impl<'a> Node<'a> {
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
                    NodeData::Document => {
                        for child_id in children_of!(nodes, id).into_iter().rev() {
                            ops.insert(0, SerializeOp::Open(child_id));
                        }
                        continue;
                    }
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
