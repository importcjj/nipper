use cssparser::ToCss;
use html5ever::{LocalName, Namespace};
use selectors::parser;
use std::fmt;
pub(crate) struct SelectorParser;

impl<'i> parser::Parser<'i> for SelectorParser {
    type Impl = Selector;
    type Error = parser::SelectorParseErrorKind<'i>;
}

#[derive(Debug, Clone)]
pub(crate) struct Selector;

impl parser::SelectorImpl for Selector {
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
pub(crate) struct NonTSPseudoClass;

impl parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = Selector;

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
pub(crate) struct PseudoElement;

impl parser::PseudoElement for PseudoElement {
    type Impl = Selector;
}

impl cssparser::ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str("")
    }
}
