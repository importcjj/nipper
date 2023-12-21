use nipper::Document;

#[test]
fn test_pseudo_class_has() {
    let html = r#"
    <div>
        <a href="/1">One</a>
        <a href="/2">Two</a>
        <a href="/3"><span>Three</span></a>
    </div>"#;
    let document = Document::from(html);
    let sel = r#"div:has(a[href="/1"]) a span"#;
    let span = document.select(sel);

    let text: &str = &span.text();
    assert!(text == "Three");
}

#[test]
fn test_pseudo_class_has_any_link() {
    let html = r#"
    <div>
        <a href="/1">One</a>
        <a href="/2">Two</a>
        <a href="/3"><span>Three</span></a>
    </div>"#;
    let document = Document::from(html);
    let sel = r#"div:has(*:any-link) a span"#;
    let span = document.select(sel);

    let text: &str = &span.text();
    assert!(text == "Three");
}
