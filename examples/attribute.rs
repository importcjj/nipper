
use rsquery::Document;


fn main() {
    let html = r#"<input name="foo" value="bar">"#;
    let document = Document::from_str(html);

    let input = document.find(r#"input[name="foox"]"#);
    println!("{:?}",input);
}