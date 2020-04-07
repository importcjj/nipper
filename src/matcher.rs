use crate::dom_tree::{NodeData, NodeId, NodeRef};
use cssparser::ParseError;
use html5ever::{LocalName, Namespace};
use selectors::matching;

use selectors::parser::{self, SelectorList, SelectorParseErrorKind};
use selectors::Element;
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Matcher {
    selector_list: SelectorList<InnerSelector>,
}

impl Matcher {
    pub(crate) fn new(sel: &str) -> Result<Self, ParseError<SelectorParseErrorKind>> {
        let mut input = cssparser::ParserInput::new(sel);
        let mut parser = cssparser::Parser::new(&mut input);
        selectors::parser::SelectorList::parse(&InnerSelectorParser, &mut parser)
            .map(|selector_list| Matcher { selector_list })
    }

    pub(crate) fn match_element<E>(&self, element: &E) -> bool
    where
        E: Element<Impl = InnerSelector>,
    {
        let mut ctx = matching::MatchingContext::new(
            matching::MatchingMode::Normal,
            None,
            None,
            matching::QuirksMode::NoQuirks,
        );

        matching::matches_selector_list(&self.selector_list, element, &mut ctx)
    }
}

#[derive(Debug, Clone)]
pub struct Matches<T> {
    roots: Vec<T>,
    nodes: Vec<T>,
    matcher: Matcher,
    set: HashSet<NodeId>,
}

impl<T> Matches<T> {
    pub fn from_one(node: T, matcher: Matcher) -> Self {
        Self {
            roots: vec![node],
            nodes: vec![],
            matcher: matcher,
            set: HashSet::new(),
        }
    }

    pub fn from_list<I: Iterator<Item = T>>(nodes: I, matcher: Matcher) -> Self {
        Self {
            roots: nodes.collect(),
            nodes: vec![],
            matcher: matcher,
            set: HashSet::new(),
        }
    }
}

impl<'a> Iterator for Matches<NodeRef<'a, NodeData>> {
    type Item = NodeRef<'a, NodeData>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.nodes.is_empty() {
                if self.roots.is_empty() {
                    return None;
                }

                let root = self.roots.remove(0);
                for child in root.children().into_iter().rev() {
                    self.nodes.insert(0, child);
                }
            }

            while !self.nodes.is_empty() {
                let node = self.nodes.remove(0);

                for node in node.children().into_iter().rev() {
                    self.nodes.insert(0, node);
                }

                if self.matcher.match_element(&node) {
                    if self.set.contains(&node.id) {
                        continue;
                    }

                    self.set.insert(node.id);
                    return Some(node);
                }
            }
        }
    }
}

pub(crate) struct InnerSelectorParser;

impl<'i> parser::Parser<'i> for InnerSelectorParser {
    type Impl = InnerSelector;
    type Error = parser::SelectorParseErrorKind<'i>;
}

#[derive(Debug, Clone)]
pub struct InnerSelector;

impl parser::SelectorImpl for InnerSelector {
    type ExtraMatchingData = String;
    type AttrValue = String;
    type Identifier = LocalName;
    type ClassName = LocalName;
    type PartName = LocalName;
    type LocalName = LocalName;
    type NamespaceUrl = Namespace;
    type NamespacePrefix = LocalName;
    type BorrowedLocalName = LocalName;
    type BorrowedNamespaceUrl = Namespace;

    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}

#[derive(Clone, Eq, PartialEq)]
pub struct NonTSPseudoClass;

impl parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = InnerSelector;

    fn is_active_or_hover(&self) -> bool {
        false
    }

    fn is_user_action_state(&self) -> bool {
        false
    }

    fn has_zero_specificity(&self) -> bool {
        false
    }
}

impl cssparser::ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str("")
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct PseudoElement;

impl parser::PseudoElement for PseudoElement {
    type Impl = InnerSelector;
}

impl cssparser::ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str("")
    }
}
