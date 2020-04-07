
# Nipper

> Manipulation HTML with CSS Selectors.

<div>
  <!-- Crates version -->
  <a href="https://crates.io/crates/nipper">
    <img src="https://img.shields.io/crates/v/nipper.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/nipper">
    <img src="https://img.shields.io/crates/d/nipper.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/nipper">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>


Nipper based on HTML crate html5ever and the CSS selector crate selectors. You can use the jQuery-like syntax to query and manipulate an HTML document quickly. **Not only can query, but also can modify**.


![nipper-logo](./logo.jpg)

## Example

#### Extract the hacker news.

```rust
use nipper::Document;

let html = include_str!("../test-pages/hacker_news.html");
let document = Document::from(html);

document.select("tr.athing").iter().for_each(|athing| {
    let title = athing.select(".title a");
    let source = title.select(".sitestr");
    // The next sibling.
    let meta = athing.next();
    let score = meta.select("span.score");
    let hnuser = meta.select("a.hnuser");
    let age = meta.select("span.age");
    // The last matched element.
    let comment = meta.select("a").last();

    println!("Title: {}", title.text());
    if source.exists() {
        println!("> from: {}", source.text());
    }
    if score.exists() {
        println!("> {}", score.text());
    }
    if hnuser.exists() {
        println!("> by {}", hnuser.text());
    }
    println!("> {}", age.text());
    println!("> {}", comment.text());
    println!();
});
```

#### Readability. 
[examples/readability.rs](./examples/readability.rs)

## Related projects

* [html5ever](https://crates.io/crates/html5ever)
* [selectors](https://crates.io/crates/selectors)
* [goquery](https://godoc.org/github.com/PuerkitoBio/goquery)
* [scraper](https://crates.io/crates/scraper)
* [select.rs](https://crates.io/crates/select)


## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
