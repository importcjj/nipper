use crate::document::Node;
use crate::matcher::Matcher;
use crate::matcher::Matches;
use crate::Document;
use std::vec::IntoIter;
use tendril::StrTendril;

impl Document {
    pub fn find(&self, sel: &str) -> Selection {
        let matcher = Matcher::new(sel).unwrap();
        self.find_with_matcher(&matcher)
    }

    pub fn find_with_matcher<'a, 'b>(&'a self, matcher: &'b Matcher) -> Selection<'a> {
        let root = self.tree.root();
        let nodes = matcher.clone().match_one(root).collect();

        Selection { nodes }
    }
}
#[derive(Debug)]
pub struct Selection<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'a> Selection<'a> {
    pub fn find(&self, sel: &str) -> Selection<'a> {
        let matcher = Matcher::new(sel).unwrap();
        let nodes = matcher.match_all(self.nodes.clone().into_iter()).collect();

        Selection { nodes }
    }

    pub fn iter(&self) -> Selections<Node<'a>> {
        Selections::new(self.nodes.clone().into_iter())
    }

    pub fn html(&self) -> StrTendril {
        let mut s = StrTendril::new();

        for node in &self.nodes {
            s.push_tendril(&node.html());
        }

        s
    }

    pub fn text(&self) -> StrTendril {
        let mut s = StrTendril::new();

        for node in &self.nodes {
            s.push_tendril(&node.text());
        }

        s
    }

    pub fn remove(&self) {
        for node in &self.nodes {
            node.remove_from_parent()
        }
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
