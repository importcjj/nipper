use crate::document::Node;
use crate::matcher::{Matcher, Matches};
use crate::Document;
use std::vec::IntoIter;
use tendril::StrTendril;

impl Document {
    pub fn select(&self, sel: &str) -> Selection {
        match Matcher::new(sel) {
            Ok(matcher) => self.find_with_matcher(&matcher),
            Err(_) => Default::default(),
        }
    }

    pub fn find_with_matcher<'a, 'b>(&'a self, matcher: &'b Matcher) -> Selection<'a> {
        let root = self.tree.root();
        let nodes = Matches::from_one(root, matcher.clone()).collect();

        Selection { nodes }
    }

    pub fn html(&self) -> StrTendril {
        match self.tree.root().first_child() {
            Some(child) => child.html(),
            _ => StrTendril::new(),
        }
    }
}
#[derive(Debug)]
pub struct Selection<'a> {
    pub(crate) nodes: Vec<Node<'a>>,
}

impl<'a> Default for Selection<'a> {
    fn default() -> Self {
        Self { nodes: vec![] }
    }
}

impl<'a> Selection<'a> {
    pub fn select(&self, sel: &str) -> Selection<'a> {
        match Matcher::new(sel) {
            Ok(matcher) => Selection {
                nodes: Matches::from_list(self.nodes.clone().into_iter(), matcher).collect(),
            },
            Err(_) => Default::default(),
        }
    }

    pub fn iter(&self) -> Selections<Node<'a>> {
        Selections::new(self.nodes.clone().into_iter())
    }

    pub fn nodes(&self) -> &[Node<'a>] {
        &self.nodes
    }

    pub fn remove(&self) {
        for node in &self.nodes {
            node.remove_from_parent()
        }
    }

    pub fn get(&self, index: usize) -> Option<&Node<'a>> {
        self.nodes.get(index)
    }
}

pub struct Selections<I> {
    iter: IntoIter<I>,
}

impl<I> Selections<I> {
    fn new(iter: IntoIter<I>) -> Self {
        Self { iter }
    }
}

impl<'a> Iterator for Selections<Node<'a>> {
    type Item = Selection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|node| Selection { nodes: vec![node] })
    }
}
