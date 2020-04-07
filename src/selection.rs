use crate::dom_tree::Node;

/// Selection represents a collection of nodes matching some criteria. The
/// initial Selection object can be created by using `Doucment.select`, and then
/// manipulated using methods itself.
#[derive(Debug, Clone)]
pub struct Selection<'a> {
    pub(crate) nodes: Vec<Node<'a>>,
}

impl<'a> Default for Selection<'a> {
    fn default() -> Self {
        Self { nodes: vec![] }
    }
}

impl<'a> From<Node<'a>> for Selection<'a> {
    fn from(node: Node<'a>) -> Selection {
        Self { nodes: vec![node] }
    }
}
