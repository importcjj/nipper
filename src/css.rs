use std::convert::AsRef;
use std::fmt;
use std::ops::Deref;

use cssparser::{self, ToCss, serialize_string};
use html5ever::LocalName;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CssString(String);

impl Deref for CssString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for CssString {
    fn as_ref(&self) -> &str {
        return &self.0;
    }
}

impl From<&str> for CssString {
    fn from(value: &str) -> Self {
        return CssString(value.to_owned());
    }
}

impl ToCss for CssString {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        //dest.write_str(&self.0)
        cssparser::serialize_string(&self.0, dest)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct CssLocalName(LocalName);

impl ToCss for CssLocalName {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl From<&str> for CssLocalName {
    fn from(value: &str) -> Self {
        return CssLocalName(value.into());
    }
}

impl Deref for CssLocalName {
    type Target = LocalName;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

