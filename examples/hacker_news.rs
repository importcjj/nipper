use nipper::Document;

fn main() {
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
}
