use crate::document::NodeData;
use crate::document::NodeRef;
use crate::matcher::Matcher;
use crate::Document;
use std::cell::RefCell;

impl Document {
    pub fn find(&self, sel: &str) -> Selection {
        let matcher = Matcher::new(sel).unwrap();

        let nodes: Vec<NodeRef<NodeData>> = self
            .tree
            .nodes()
            .iter()
            .enumerate()
            .map(|(i, node)| NodeRef::new(i, node, &self.tree))
            .filter(|element| matcher.match_element(element))
            .collect();

        Selection {
            nodes,
            // tree: RefCell:
        }
    }
}
#[derive(Debug)]
pub struct Selection<'a> {
    nodes: Vec<NodeRef<'a, NodeData>>,
    // tree: RefCell<Tree<NodeData>>,
}
