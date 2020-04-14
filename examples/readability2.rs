use readability::extractor::extract;
use std::time::Instant;

use std::env;
use std::fs::File;
use std::io::Cursor;

fn main() {
    let start = Instant::now();
    let url = env::args().skip(1).next().unwrap();
    let html = reqwest::blocking::get(&url).unwrap().text().unwrap();
    let url = &url.parse().unwrap();
    let mut c = Cursor::new(html.as_bytes());

    let article = extract(&mut c, &url).unwrap();

    println!("title ====> {}", article.title);
    println!("{}", article.content);
    println!("{:?}", start.elapsed());
}
