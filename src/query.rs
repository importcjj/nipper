use crate::matcher::Matcher;
use crate::Selection;
use std::collections::HashSet;

impl<'a> Selection<'a> {
    /// Checks the current matched set of elements against a selector and
    /// returns true if at least one of these elements matches.
    pub fn is(&self, sel: &str) -> bool {
        if self.length() > 0 {
            let matcher = Matcher::new(sel).unwrap();
            return self.is_matcher(&matcher);
        }

        false
    }

    /// Checks the current matched set of elements against a matcher and
    /// returns true if at least one of these elements matches.
    pub fn is_matcher(&self, matcher: &Matcher) -> bool {
        if self.length() > 0 {
            let elements = self.nodes.clone().into_iter();
            return matcher.clone().filter(elements).count() > 0;
        }

        false
    }

    /// Checks the current matches set of elemets against a selection and
    /// returns true if at least one of these elements matches.
    pub fn is_selection(&self, sel: &Selection) -> bool {
        if self.length() == 0 || sel.length() == 0 {
            return false;
        }

        let mut m = HashSet::with_capacity(sel.length());
        for node in sel.nodes() {
            m.insert(node.id);
        }

        for node in self.nodes() {
            if m.contains(&node.id) {
                return true;
            }
        }

        false
    }
}
