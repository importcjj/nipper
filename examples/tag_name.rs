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
    let selection = document.find("li");

    println!("{:?}", selection);

    let selection = document.find("ul");
    println!("{:?}", selection);
}
