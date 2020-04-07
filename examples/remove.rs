use nipper::Document;

fn main() {
    let html = r#"
    <ul>
    <li>Foo</li>
    <li>Bar</li>
    <li>Baz</li>
</ul>
"#;

    let document = Document::from(html);

    let items = document.select("ul").select("li");
    let ul = items.parent();

    println!("{}", ul.html());

    for mut item in items.next().iter() {
        println!("----");
        item.remove()
    }

    println!("{}", document.select("ul").html());
}
