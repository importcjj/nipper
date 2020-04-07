#![allow(dead_code)]
use nipper::Document;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

pub fn load_doc<P: AsRef<Path>>(path: P) -> io::Result<Document> {
    File::open(path).and_then(|mut f| {
        let mut html = String::new();
        f.read_to_string(&mut html)?;
        Ok(Document::from(&html))
    })
}

pub fn doc() -> Document {
    load_doc("test-pages/page.html").unwrap()
}

pub fn docwiki() -> Document {
    load_doc("test-pages/rustwiki.html").unwrap()
}

pub fn doc2() -> Document {
    load_doc("test-pages/page2.html").unwrap()
}
