use crate::css::{CssLocalName, CssString};
use crate::dom_tree::{NodeData, NodeId, NodeRef};

use std::convert::Into;

use cssparser::ParseError;
use cssparser::{self, CowRcStr, SourceLocation, ToCss};
use html5ever::Namespace;
use selectors::parser::{self, Selector, SelectorList, SelectorParseErrorKind};
use selectors::visitor;
use selectors::Element;
use selectors::{matching, SelectorImpl};
use std::collections::HashSet;
use std::fmt;

/// CSS selector.
#[derive(Clone, Debug)]
pub struct Matcher {
    selector_list: SelectorList<InnerSelector>,
}

impl Matcher {
    /// Greate a new CSS matcher.
    pub fn new(sel: &str) -> Result<Self, ParseError<SelectorParseErrorKind>> {
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
    match_scope: MatchScope,
}

/// Telling a `matches` if we want to skip the roots.
#[derive(Debug, Clone)]
pub enum MatchScope {
    IncludeNode,
    ChildrenOnly,
}

impl<T> Matches<T> {
    pub fn from_one(node: T, matcher: Matcher, match_scope: MatchScope) -> Self {
        Self {
            roots: vec![node],
            nodes: vec![],
            matcher: matcher,
            set: HashSet::new(),
            match_scope,
        }
    }

    pub fn from_list<I: Iterator<Item = T>>(
        nodes: I,
        matcher: Matcher,
        match_scope: MatchScope,
    ) -> Self {
        Self {
            roots: nodes.collect(),
            nodes: vec![],
            matcher: matcher,
            set: HashSet::new(),
            match_scope,
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

                match self.match_scope {
                    MatchScope::IncludeNode => self.nodes.insert(0, root),
                    MatchScope::ChildrenOnly => {
                        for child in root.children().into_iter().rev() {
                            self.nodes.insert(0, child);
                        }
                    }
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

    fn parse_non_ts_pseudo_class(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<NonTSPseudoClass, ParseError<'i, Self::Error>> {
        use self::NonTSPseudoClass::*;
        if name.eq_ignore_ascii_case("any-link") {
            Ok(AnyLink)
        } else if name.eq_ignore_ascii_case("link") {
            Ok(Link)
        } else if name.eq_ignore_ascii_case("visited") {
            Ok(Visited)
        } else if name.eq_ignore_ascii_case("active") {
            Ok(Active)
        } else if name.eq_ignore_ascii_case("focus") {
            Ok(Focus)
        } else if name.eq_ignore_ascii_case("hover") {
            Ok(Hover)
        } else if name.eq_ignore_ascii_case("enabled") {
            Ok(Enabled)
        } else if name.eq_ignore_ascii_case("disabled") {
            Ok(Disabled)
        } else if name.eq_ignore_ascii_case("checked") {
            Ok(Checked)
        } else if name.eq_ignore_ascii_case("indeterminate") {
            Ok(Indeterminate)
        } else {
            Err(
                location.new_custom_error(SelectorParseErrorKind::UnsupportedPseudoClassOrElement(
                    name,
                )),
            )
        }
    }
    fn parse_non_ts_functional_pseudo_class<'t>(
        &self,
        name: CowRcStr<'i>,
        arguments: &mut cssparser::Parser<'i, 't>,
    ) -> Result<NonTSPseudoClass, ParseError<'i, Self::Error>> {
        if name.starts_with("has") {
            let list: SelectorList<InnerSelector> = SelectorList::parse(self, arguments)?;
            Ok(NonTSPseudoClass::Has(list))
        } else {
            Err(arguments.new_custom_error(
                SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name),
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InnerSelector;

impl parser::SelectorImpl for InnerSelector {
    type ExtraMatchingData = String;
    type AttrValue = CssString;
    type Identifier = CssLocalName;
    type LocalName = CssLocalName;
    type NamespaceUrl = Namespace;
    type NamespacePrefix = CssLocalName;
    type BorrowedLocalName = CssLocalName;
    type BorrowedNamespaceUrl = Namespace;

    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum NonTSPseudoClass {
    AnyLink,
    Link,
    Visited,
    Active,
    Focus,
    Hover,
    Enabled,
    Disabled,
    Checked,
    Indeterminate,
    Has(SelectorList<InnerSelector>),
}

impl ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        println!("{:?}", self);
        match self {
            NonTSPseudoClass::AnyLink => dest.write_str(":any-link"),
            NonTSPseudoClass::Link => dest.write_str(":link"),
            NonTSPseudoClass::Visited => dest.write_str(":visited"),
            NonTSPseudoClass::Active => dest.write_str(":active"),
            NonTSPseudoClass::Focus => dest.write_str(":focus"),
            NonTSPseudoClass::Hover => dest.write_str(":hover"),
            NonTSPseudoClass::Enabled => dest.write_str(":enabled"),
            NonTSPseudoClass::Disabled => dest.write_str(":disabled"),
            NonTSPseudoClass::Checked => dest.write_str(":checked"),
            NonTSPseudoClass::Indeterminate => dest.write_str(":indeterminate"),
            NonTSPseudoClass::Has(list) => {
                println!("{}", list.to_css_string());
                dest.write_str("has:(")?;
                list.to_css(dest)?;
                dest.write_str(")")
            }
        }
    }
}

impl parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = InnerSelector;

    fn is_active_or_hover(&self) -> bool {
        false
    }

    fn is_user_action_state(&self) -> bool {
        false
    }

    fn visit<V>(&self, _visitor: &mut V) -> bool
    where
        V: visitor::SelectorVisitor<Impl = Self::Impl>,
    {
        true
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PseudoElement;

impl ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str("")
    }
}

impl parser::PseudoElement for PseudoElement {
    type Impl = InnerSelector;

    fn accepts_state_pseudo_classes(&self) -> bool {
        false
    }

    fn valid_after_slotted(&self) -> bool {
        false
    }
}
