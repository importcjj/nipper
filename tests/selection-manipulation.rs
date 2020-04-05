mod data;

use data::doc2;

#[test]
fn test_replace_with_html() {
    let doc = doc2();

    println!("{}", doc.html());

    let mut sel = doc.select("#main,#foot");
    println!("{}", sel.length());
    sel.replace_with_html(r#"<div id="replace"></div>"#);

    println!("{}", doc.html());
    println!("======");

    assert_eq!(doc.select("#replace").length(), 2);
}

#[test]
fn test_set_html() {
    let doc = doc2();
    let mut q = doc.select("#main, #foot");
    q.set_html(r#"<div id="replace">test</div>"#);

    assert_eq!(doc.select("#replace").length(), 2);
    assert_eq!(doc.select("#main, #foot").length(), 2);

    let html: &str = &q.text();
    assert_eq!(html, "testtest");
}

#[test]
fn test_set_html_no_match() {
    let doc = doc2();
    let mut q = doc.select("#notthere");
    q.set_html(r#"<div id="replace">test</div>"#);
    assert_eq!(doc.select("#replace").length(), 0);
}

#[test]
fn test_set_html_empty() {
    let doc = doc2();
    let mut q = doc.select("#main");
    q.set_html("");
    assert_eq!(doc.select("#main").length(), 1);
    assert_eq!(doc.select("#main").children().length(), 0);
}

#[test]
fn test_replace_with_selection() {
    let doc = doc2();

    let s1 = doc.select("#nf5");
    let mut sel = doc.select("#nf6");

    sel.replace_with_selection(&s1);

    assert!(sel.is("#nf6"));
    assert_eq!(doc.select("#nf6").length(), 0);
    assert_eq!(doc.select("#nf5").length(), 1);
}
