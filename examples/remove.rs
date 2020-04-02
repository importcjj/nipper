use rsquery::Document;

fn main() {
    let html = r#"
    <ul>
    <li>Foo</li>
    <li>Bar</li>
    <li>Baz</li>
</ul>
"#;

    let document = Document::from_str(html);

    let items = document.find("ul").find("li");

    for item in items.iter() {
        item.remove()
    }

    println!("{}", document.find("ul").html());
}
