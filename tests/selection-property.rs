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

#[test]
fn test_add_class() {
    let doc = doc2();
    let mut sel = doc.select("#main");

    sel.add_class("main main main");
    let class: &str = &sel.attr("class").unwrap();
    assert_eq!(class, "main");
}

#[test]
fn test_add_class_similar() {
    let doc = doc2();
    let mut sel = doc.select("#nf5");

    sel.add_class("odd");
    println!("{}", sel.html());

    assert!(sel.has_class("odd"));
    assert!(sel.has_class("odder"));
}

#[test]
fn test_add_empty_class() {
    let doc = doc2();
    let mut sel = doc.select("#main");

    sel.add_class("");
    assert!(sel.attr("class").is_none());
}

#[test]
fn test_add_classes() {
    let doc = doc2();
    let mut sel = doc.select("#main");

    sel.add_class("a b");
    assert!(sel.has_class("a"));
    assert!(sel.has_class("b"));
}

#[test]
fn test_has_class() {
    let doc = doc();
    let sel = doc.select("div");
    assert!(sel.has_class("span12"));
}

#[test]
fn has_class_none() {
    let doc = doc();
    let sel = doc.select("toto");
    assert!(!sel.has_class("toto"));
}

#[test]
fn has_class_not_first() {
    let doc = doc();
    let sel = doc.select(".alert");
    assert!(sel.has_class("alert-error"));
}

#[test]
fn test_remove_class() {
    let doc = doc2();
    let mut sel = doc.select("#nf1");
    sel.remove_class("one row");

    assert!(sel.has_class("even"));
    assert!(!sel.has_class("one"));
    assert!(!sel.has_class("row"));
}

#[test]
fn test_remove_class_similar() {
    let doc = doc2();
    let mut sel = doc.select("#nf5, #nf6");
    assert_eq!(sel.length(), 2);

    sel.remove_class("odd");
    assert!(sel.has_class("odder"));
}
