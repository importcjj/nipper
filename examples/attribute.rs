use rsquery::Document;

fn main() {
    let html = r#"<div name="foo" value="bar"></div>"#;
    let document = Document::from_str(html);
    println!("{}", document.html());

    let mut input = document.select(r#"div[name="foo"]"#);
    println!("{}", input.html());
    input.set_attr("id", "input");
    input.remove_attr("name");
    println!("{}", input.attr("value").unwrap());

    println!("{}", input.html());

    input.replace_with_html(r#"<a href="https://wisburg.com">wisburg</a><h2>xxx</h2>"#);
    println!("{}", input.html());
    println!("{}", document.html());
}
