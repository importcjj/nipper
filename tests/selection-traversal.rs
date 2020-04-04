mod data;

use data::doc;
use data::docwiki;

#[test]
fn test_select() {
    let doc = doc();
    let sel = doc.select("div.row-fluid");
    assert_eq!(sel.length(), 9);
}

#[test]
fn test_select_not_self() {
    let doc = doc();
    let sel = doc.select("h1").select("h1");
    assert_eq!(sel.length(), 0);
}

#[test]
fn test_select_invalid() {
    let doc = doc();
    let sel = doc.select(":+ ^");
    assert_eq!(sel.length(), 0);
}

#[test]
fn test_select_big() {
    let doc = docwiki();
    let sel = doc.select("li");
    assert_eq!(sel.length(), 420);
    let sel = doc.select("span");
    assert_eq!(sel.length(), 706);
}

#[test]
fn test_chained_select() {
    let doc = doc();
    let sel = doc.select("div.hero-unit").select(".row-fluid");
    assert_eq!(sel.length(), 4);
}

#[test]
fn test_chained_select_invalid() {
    let doc = doc();
    let sel = doc.select("div.hero-unit").select("");
    assert_eq!(sel.length(), 0);
}

#[test]
fn test_children() {
    let doc = doc();
    let sel = doc.select(".pvk-content").children();
    assert_eq!(sel.length(), 5)
}

#[test]
fn test_parent() {
    let doc = doc();
    let sel = doc.select(".container-fluid").parent();
    assert_eq!(sel.length(), 3)
}

#[test]
fn test_parent_body() {
    let doc = doc();
    let sel = doc.select("body").parent();
    assert_eq!(sel.length(), 1)
}

#[test]
fn test_next() {
    let doc = doc();
    let sel = doc.select("h1").next();
    assert_eq!(sel.length(), 1)
}

#[test]
fn test_next2() {
    let doc = doc();
    let sel = doc.select(".close").next();
    assert_eq!(sel.length(), 1)
}

#[test]
fn test_next_none() {
    let doc = doc();
    let sel = doc.select("small").next();
    assert_eq!(sel.length(), 0)
}
