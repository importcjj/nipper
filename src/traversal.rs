use crate::Selection;
use std::collections::HashSet;

impl<'a> Selection<'a> {
    /// parents gets the parent of each element in the selection. It returns a
    /// mew selection containing these elements.
    pub fn parent(&self) -> Self {
        let mut result = Vec::with_capacity(self.length());
        let mut set = HashSet::with_capacity(self.length());

        for node in self.nodes() {
            if let Some(parent) = node.parent() {
                if !set.contains(&parent.id) {
                    set.insert(parent.id);
                    result.push(parent);
                }
            }
        }

        Self { nodes: result }
    }

    /// children gets the child elements of each element in the selection.
    /// It returns a new selection containing these elements.
    pub fn children(&self) -> Self {
        let mut result = Vec::with_capacity(self.length());
        let mut set = HashSet::with_capacity(self.length());

        for node in self.nodes() {
            for child in node.children() {
                if !set.contains(&child.id) && child.is_element() {
                    set.insert(child.id);
                    result.push(child);
                }
            }
        }

        Self { nodes: result }
    }

    /// next gets the immediately following sibling of each element in the
    /// selection. It returns a new selection containing these elements.
    pub fn next(&self) -> Self {
        let mut result = Vec::with_capacity(self.length());
        let mut set = HashSet::with_capacity(self.length());

        for node in self.nodes() {
            for sibling in node.next_sibling() {
                if !set.contains(&sibling.id) {
                    set.insert(sibling.id);
                    result.push(sibling);
                }
            }
        }

        Self { nodes: result }
    }
}
