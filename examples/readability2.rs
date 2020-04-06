use readability::extractor::extract;
use std::time::Instant;

use std::env;
use std::fs::File;

fn main() {
    let start = Instant::now();
    let html_file_path = env::args().skip(1).next().unwrap();
    let mut html_file = File::open(&html_file_path).expect("correct HTML file path");
    let url = "https://www.wisburg.com".parse().unwrap();

    let article =  extract(&mut html_file, &url).unwrap();

    println!("title ====> {}", article.title);
    println!("{}", article.content);
    println!("{:?}", start.elapsed());
}