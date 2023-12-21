use crate::dom_tree::Node;

/// Selection represents a collection of nodes matching some criteria. The
/// initial Selection object can be created by using [`Document::select`], and then
/// manipulated using methods itself.
#[derive(Debug, Clone, Default)]
pub struct Selection<'a> {
    pub(crate) nodes: Vec<Node<'a>>,
}

impl<'a> From<Node<'a>> for Selection<'a> {
    fn from(node: Node<'a>) -> Selection {
        Self { nodes: vec![node] }
    }
}
