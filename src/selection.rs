use crate::document::NodeData;
use crate::document::NodeRef;
use crate::matcher::Matcher;
use crate::matcher::MatchAll;
use crate::Document;
use std::cell::RefCell;

impl Document {
    pub fn find(&self, sel: &str) -> Selection {
        let matcher = Matcher::new(sel).unwrap();
        self.find_with_matcher(&matcher)
    }

    pub fn find_with_matcher<'a, 'b>(&'a self, matcher: &'b Matcher) -> Selection<'a> {
        let root = self.tree.root();

        let nodes = matcher.match_all(root).collect();
        Selection {
            nodes,
            // tree: RefCell:
        }
    }
}
#[derive(Debug)]
pub struct Selection<'a> {
    matches: 
    // tree: RefCell<Tree<NodeData>>,
}

impl<'a> Selection<'a> {
    pub fn find(&self, sel: &str) -> Selection<'a> {
        let matcher = Matcher::new(sel).unwrap();
        let nodes = matcher.match_alls(self.nodes.clone()).collect();
        Selection {
            nodes,
            // tree: RefCell:
        }
    }

    fn new_with_single_node(node: NodeRef<'a, NodeData>) -> Selection<'a> {
        Self { nodes: vec![node] }
    }
}

impl<'a> Iterator for Selection<'a> {
    type Item = Selection<'a>;

    fn next(&mut self) -> Self::Item {
        
    }
}
