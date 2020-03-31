use rsquery::Document;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let html = reqwest::blocking::get("https://wisburg.com")?.text()?;

    let document = Document::from_str(&html);

    for article in document.find(".article-list .item").iter() {
        let title = article.find(".title");
        let summary = article.find(".summary");
        println!("title => {}", title.text());
        println!("summary => {}", summary.text());
    }

    Ok(())
}
