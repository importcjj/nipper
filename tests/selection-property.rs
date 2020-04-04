mod data;

use data::doc;
use data::doc2;

#[test]
fn test_attr_exists() {
    let doc = doc();
    assert!(doc.select("a").attr("href").is_some());
}

#[test]
fn test_attr_or() {
    let doc = doc();
    let attr1: &str = &doc.select("a").attr_or("fake-attribute", "alternative");
    let attr2: &str = &doc.select("zz").attr_or("fake-attribute", "alternative");
    assert_eq!(attr1, "alternative");
    assert_eq!(attr2, "alternative");
}

#[test]
fn test_attr_not_exist() {
    let doc = doc();
    assert!(doc.select("div.row-fluid").attr("href").is_none());
}

#[test]
fn test_remove_attr() {
    let doc = doc2();
    let mut sel = doc.select("div");

    sel.remove_attr("id");

    assert!(sel.attr("id").is_none());
}

#[test]
fn test_set_attr() {
    let doc = doc2();
    let mut sel = doc.select("#main");
    sel.set_attr("id", "not-main");

    let id: &str = &sel.attr("id").expect("got an attribute");
    assert_eq!(id, "not-main");
}

#[test]
fn test_set_attr2() {
    let doc = doc2();
    let mut sel = doc.select("#main");

    sel.set_attr("foo", "bar");

    let id: &str = &sel.attr("foo").expect("got an attribute");
    assert_eq!(id, "bar");
}

#[test]
fn test_text() {
    let doc = doc();
    let txt: &str = &doc.select("h1").text();

    assert_eq!(txt.trim(), "Provok.in");
}
