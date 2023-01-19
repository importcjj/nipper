use std::convert::AsRef;
use std::fmt;
use std::ops::Deref;

use cssparser::{self, ToCss};
use html5ever::LocalName;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StringCSS(String);

impl Deref for StringCSS {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for StringCSS {
    fn as_ref(&self) -> &str {
        return self.0.as_str();
    }
}

impl From<&str> for StringCSS {
    fn from(value: &str) -> Self {
        let s = String::from(value);
        return StringCSS(s);
    }
}

impl ToCss for StringCSS {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(self.0.as_str())
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LocalNameCSS(LocalName);

impl ToCss for LocalNameCSS {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(self.0.trim())
    }
}

impl From<&str> for LocalNameCSS {
    fn from(value: &str) -> Self {
        let s = LocalName::from(value);
        return LocalNameCSS(s);
    }
}

impl Deref for LocalNameCSS {
    type Target = LocalName;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for LocalNameCSS {
    fn default() -> Self {
        Self(Default::default())
    }
}
