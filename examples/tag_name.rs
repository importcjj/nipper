use nipper::Document;

fn main() {
    let html = r#"
    <ul>
    <li>Foo</li>
    <li>Bar</li>
    <li>Baz</li>
</ul>
"#;

    let document = Document::from_str(html);

    let items = document.select("ul").select("li");

    for item in items.iter() {
        println!("{}", item.html());
        println!("{}", item.text());
    }
}
