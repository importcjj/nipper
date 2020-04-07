#![allow(dead_code)]
use nipper::Document;

pub fn doc() -> Document {
    include_str!("../test-pages/page.html").into()
}

pub fn docwiki() -> Document {
    include_str!("../test-pages/rustwiki.html").into()
}

pub fn doc2() -> Document {
    include_str!("../test-pages/page2.html").into()
}
