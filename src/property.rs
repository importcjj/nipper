use crate::Selection;
use tendril::StrTendril;

impl<'a> Selection<'a> {
    /// attr gets the specified attribute's value for the first element in the
    /// selection. To get the value for each element individually, use a looping
    /// construct such as map method.
    pub fn attr(&self, name: &str) -> Option<StrTendril> {
        self.nodes().first().and_then(|node| node.attr(name))
    }

    /// sets the given attribute to each element in the set of matched elements.
    pub fn set_attr(&self, name: &str, val: &str) {
        for node in self.nodes() {
            node.set_attr(name, val);
        }
    }

    /// remove_attr removes the named attribute from each element in the set of matched elements.
    pub fn remove_attr(&self, name: &str) {
        for node in self.nodes() {
            node.remove_attr(name);
        }
    }

    /// length returns the number of elements in the selection object.
    pub fn length(&self) -> usize {
        self.nodes().len()
    }

    /// size is an alias for `length`.
    pub fn size(&self) -> usize {
        self.length()
    }

    /// attor_or works like `attr` but returns default value if attribute is not present.
    pub fn attr_or(&self, name: &str, default: &str) -> Option<StrTendril> {
        self.attr(name).or_else(|| Some(StrTendril::from(default)))
    }

    /// html gets the HTML contents of the first element in the set of matched
    /// elements. It includes the text and comment nodes.
    pub fn html(&self) -> StrTendril {
        if self.length() > 0 {
            return self.nodes().first().unwrap().html();
        }

        StrTendril::new()
    }

    /// text get the combined text content of each element in the set of matched
    /// elements, including their descendants.
    pub fn text(&self) -> StrTendril {
        let mut s = StrTendril::new();

        for node in self.nodes() {
            s.push_tendril(&node.text());
        }

        s
    }
}
