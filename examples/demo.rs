use rsquery::Document;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let html = reqwest::blocking::get("https://wisburg.com")?.text()?;

    let document = Document::from_str(&html);

    Ok(())
}
