use crate::matcher::InnerSelector;

use crate::document::{Node, NodeData};
use selectors::attr::AttrSelectorOperation;
use selectors::attr::CaseSensitivity;
use selectors::attr::NamespaceConstraint;
use selectors::context::MatchingContext;
use selectors::matching::ElementSelectorFlags;
use selectors::parser::SelectorImpl;
use selectors::OpaqueElement;
use std::ops::Deref;

impl<'a> selectors::Element for Node<'a> {
    type Impl = InnerSelector;

    // Converts self into an opaque representation.
    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(&self.id)
    }

    fn parent_element(&self) -> Option<Self> {
        self.parent()
    }

    // Whether the parent node of this element is a shadow root.
    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    // The host of the containing shadow root, if any.
    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    // Whether we're matching on a pseudo-element.
    fn is_pseudo_element(&self) -> bool {
        false
    }

    // Skips non-element nodes.
    fn prev_sibling_element(&self) -> Option<Self> {
        None
    }

    // Skips non-element nodes.
    fn next_sibling_element(&self) -> Option<Self> {
        None
    }

    fn is_html_element_in_html_document(&self) -> bool {
        false
    }

    fn has_local_name(&self, local_name: &<Self::Impl as SelectorImpl>::BorrowedLocalName) -> bool {
        self.query(|node| {
            if let NodeData::Element(ref e) = node.data {
                return &e.name.local == local_name;
            }

            false
        })
    }

    // Empty string for no namespace.
    fn has_namespace(&self, _ns: &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl) -> bool {
        false
    }

    // Whether this element and the `other` element have the same local name and namespace.
    fn is_same_type(&self, other: &Self) -> bool {
        self.tree.compare_node(&self.id, &other.id, |a, b| {
            if let NodeData::Element(ref e1) = a.data {
                return match b.data {
                    NodeData::Element(ref e2) => e1.name == e2.name,
                    _ => false,
                };
            }

            false
        })
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool {
        self.query(|node| {
            if let NodeData::Element(ref e) = node.data {
                return e.attrs.iter().any(|attr| match *ns {
                    NamespaceConstraint::Specific(url) if *url != attr.name.ns => false,
                    _ => *local_name == attr.name.local && operation.eval_str(&attr.value),
                });
            }

            false
        })
    }

    fn match_non_ts_pseudo_class<F>(
        &self,
        _pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        _context: &mut MatchingContext<Self::Impl>,
        _flags_setter: &mut F,
    ) -> bool
    where
        F: FnMut(&Self, ElementSelectorFlags),
    {
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    // Whether this element is a `link`.
    fn is_link(&self) -> bool {
        false
    }

    // Whether the element is an HTML element.
    fn is_html_slot_element(&self) -> bool {
        true
    }

    fn has_id(
        &self,
        name: &<Self::Impl as SelectorImpl>::Identifier,
        case_sensitivity: CaseSensitivity,
    ) -> bool {
        self.query(|node| {
            if let NodeData::Element(ref e) = node.data {
                return e.attrs.iter().any(|attr| {
                    attr.name.local.deref() == "id"
                        && case_sensitivity.eq(name.as_bytes(), attr.value.as_bytes())
                });
            }

            false
        })
    }

    fn has_class(
        &self,
        name: &<Self::Impl as SelectorImpl>::ClassName,
        case_sensitivity: CaseSensitivity,
    ) -> bool {
        self.query(|node| {
            if let NodeData::Element(ref e) = node.data {
                return e
                    .attrs
                    .iter()
                    .find(|a| a.name.local.deref() == "class")
                    .map_or(vec![], |a| a.value.deref().split_whitespace().collect())
                    .iter()
                    .any(|c| case_sensitivity.eq(name.as_bytes(), c.as_bytes()));
            }

            false
        })
    }

    // Returns the mapping from the `exportparts` attribute in the regular direction, that is, inner-tree->outer-tree.
    fn exported_part(
        &self,
        _name: &<Self::Impl as SelectorImpl>::PartName,
    ) -> Option<<Self::Impl as SelectorImpl>::PartName> {
        None
    }

    // Returns the mapping from the `exportparts` attribute in the regular direction, that is, outer-tree->inner-tree.
    fn imported_part(
        &self,
        _name: &<Self::Impl as SelectorImpl>::PartName,
    ) -> Option<<Self::Impl as SelectorImpl>::PartName> {
        None
    }

    fn is_part(&self, _name: &<Self::Impl as SelectorImpl>::PartName) -> bool {
        false
    }

    // Whether this element matches `:empty`.
    fn is_empty(&self) -> bool {
        !self
            .children()
            .iter()
            .any(|child| child.is_element() || child.is_text())
    }

    // Whether this element matches `:root`, i.e. whether it is the root element of a document.
    fn is_root(&self) -> bool {
        self.is_document()
    }
}
