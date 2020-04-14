use nipper::Document;
use std::error::Error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let html = reqwest::blocking::get("https://wisburg.com")?.text()?;

    let start = Instant::now();
    let document = Document::from(&html);

    for article in document.nip(".article-list .item").iter() {
        let title = article.select(".title");
        let summary = article.select(".summary");
        println!("title => {}", title.text());
        println!("summary => {}", summary.text());
        println!("href => {}", article.attr("href").unwrap());
    }

    println!("{:?}", start.elapsed());
    Ok(())
}
