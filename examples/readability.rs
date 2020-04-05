use lazy_static::lazy_static;
use nipper::Document;
use nipper::Selection;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::time::Instant;

lazy_static! {
    static ref RE_REPLACE_BRS: Regex = Regex::new(r#"(?is)(<br[^>]*>[ \n\r\t]*){2,}"#).unwrap();
    static ref RE_TITLE_SEPARATOR: Regex = Regex::new(r#"(?is) [\|\-\\/>»] "#).unwrap();
    static ref RE_TITLE_HIERARCHY_SEP: Regex = Regex::new(r#"(?is)[\\/>»]"#).unwrap();
    static ref RE_BY_LINE: Regex = Regex::new(r#"(?is)byline|author|dateline|writtenby|p-author"#).unwrap();
    static ref RE_UNLIKELY_CANDIDATES: Regex = Regex::new(r#"(?is)banner|breadcrumbs|combx|comment|community|cover-wrap|disqus|extra|foot|header|legends|menu|related|remark|replies|rss|shoutbox|sidebar|skyscraper|social|sponsor|supplemental|ad-break|agegate|pagination|pager|popup|yom-remote|subscribe"#).unwrap();
    static ref RE_OK_MAYBE_CANDIDATE: Regex = Regex::new(r#"`(?is)and|article|body|column|main|shadow"#).unwrap();
    static ref RE_UNLIKELY_ELEMENTS: Regex = Regex::new(r#"(?is)(input|time|button|svg)"#).unwrap();
    static ref RE_LIKELY_ELEMENTS: Regex = Regex::new(r#"(?is)(no-svg)"#).unwrap();
    static ref RE_POSITIVE: Regex = Regex::new(r#"(?is)article|body|content|entry|hentry|h-entry|main|page|pagination|post|text|blog|story|paragraph"#).unwrap();
    static ref RE_NEGATIVE: Regex = Regex::new(r#"(?is)hidden|^hid$| hid$| hid |^hid |banner|combx|comment|com-|contact|foot|footer|footnote|masthead|media|meta|outbrain|promo|related|scroll|share|shoutbox|sidebar|skyscraper|sponsor|shopping|tags|tool|widget"#).unwrap();
    static ref RE_DIV_TO_P_ELEMENTS: Regex = Regex::new(r#"(?is)<(a|blockquote|dl|div|img|ol|p|pre|table|ul|select)"#).unwrap();

}

#[derive(Debug)]
struct MetaData {
    title: Option<String>,
    cover: Option<String>,
    description: Option<String>,
    author: Option<String>,
    min_read_time: Option<usize>,
    max_read_time: Option<usize>,
}

impl Default for MetaData {
    fn default() -> MetaData {
        MetaData {
            title: None,
            cover: None,
            description: None,
            author: None,
            min_read_time: None,
            max_read_time: None,
        }
    }
}

fn remove_script(doc: &Document) {
    doc.select("script").remove();
    doc.select("noscript").remove();
}

fn remove_style(doc: &Document) {
    doc.select("style").remove();
}

fn replace_brs(doc: &Document) {
    let mut body = doc.select("body");

    let mut html: &str = &body.html();
    let r = RE_REPLACE_BRS.replace_all(&html, "</p><p>");
    html = &r;
    body.set_html(html);

    body.select("p").iter().for_each(|p| {
        let html: &str = &p.html();
        if html.trim() == "" {
            p.remove();
        }
    });
}

fn prep_document(doc: &Document) {
    replace_brs(&doc);

    doc.select("font").iter().for_each(|mut font| {
        let html: &str = &font.html();
        let mut new_html = "<span>".to_string();
        new_html.push_str(html);
        new_html.push_str("</span>");
        font.replace_with_html(new_html.as_str());
    })
}

fn get_article_metadata(doc: &Document) -> MetaData {
    let mut metadata = MetaData::default();

    doc.select("meta").iter().for_each(|meta| {
        let name = meta.attr_or("name", "");
        let property = meta.attr_or("property", "");
        let content = meta.attr_or("content", "");

        if content.deref() == "" {
            return;
        }

        if name.contains("author") || property.contains("author") {
            metadata.author = Some(content.to_string());
        }

        if property.deref() == "og:image" || name.deref() == "twitter:image" {
            metadata.cover = Some(content.to_string());
        }

        if name.deref() == "description"
            || property.deref() == "og:description"
            || name.deref() == "twitter:description"
        {
            metadata.description = Some(content.to_string());
        }

        if property.deref() == "og:title" || name.deref() == "twitter:title" {
            metadata.title = Some(content.to_string());
        }
    });

    metadata
}

fn get_article_title(doc: &Document) -> Option<String> {
    let original_title = doc
        .select("title")
        .iter()
        .next()
        .map(|t| t.text())
        .unwrap_or_else(|| tendril::StrTendril::new());

    None
}

macro_rules! is_valid_by_line {
    ($text: expr) => {
        $text.len() > 0 && $text.len() < 100
    };
}

macro_rules! is_element_without_content {
    ($sel: expr) => {{
        let html = $sel.html();
        html.trim() == ""
    }};
}

macro_rules! has_single_p_inside_element {
    ($sel: expr) => {{
        let children = $sel.children();
        children.length() == 1 && children.is("p")
    }};
}

macro_rules! has_child_block_element {
    ($sel: expr) => {{
        let html = sel.html();
        RE_DIV_TO_P_ELEMENTS.is_match(&html)
    }};
}

macro_rules! get_class_or_id_weight {
    ($sel: expr) => {{
        let mut weight = 0.0;
        let score = 45.0;

        if let Some(class) = $sel.attr("class") {
            let class = &class.to_lowercase();
            if RE_NEGATIVE.is_match(class) {
                weight -= score;
            }

            if RE_POSITIVE.is_match(class) {
                weight += score;
            }
        }

        if let Some(id) = $sel.attr("id") {
            let id = &id.to_lowercase();
            if RE_NEGATIVE.is_match(id) {
                weight -= score;
            }

            if RE_POSITIVE.is_match(id) {
                weight += score;
            }
        }

        weight
    }};
}

fn grab_article(doc: &Document) {
    let mut author = None;
    // let mut elements_to_score = vec![];
    for node in doc.select("*").nodes() {
        let tag_name = node
            .node_name()
            .unwrap_or_else(|| tendril::StrTendril::new());

        let sel = Selection::from(node.clone());
        let class: &str = &sel.attr_or("class", "");
        let id: &str = &sel.attr_or("id", "");
        let match_str = [class.to_lowercase(), id.to_lowercase()].join(" ");

        if let Some(rel) = sel.attr("rel") {
            if rel.deref() == "author" || RE_BY_LINE.is_match(&match_str) {
                let text = sel.text();
                if is_valid_by_line!(&text) {
                    author = Some(text.to_string());
                    sel.remove();
                    continue;
                }
            }
        }

        if RE_UNLIKELY_CANDIDATES.is_match(&match_str)
            && !RE_OK_MAYBE_CANDIDATE.is_match(&match_str)
            && !sel.is("html")
            && !sel.is("article")
            && !sel.is("body")
            && !sel.is("a")
            && get_class_or_id_weight!(&sel) <= 0.0
        {
            sel.remove();
            continue;
        }

        if RE_UNLIKELY_CANDIDATES.is_match(&tag_name) {
            sel.remove();
            continue;
        }

        if RE_UNLIKELY_ELEMENTS.is_match(&match_str) && !RE_LIKELY_ELEMENTS.is_match(&match_str) {
            sel.remove();
            continue;
        }
        if RE_LIKELY_ELEMENTS.is_match(&tag_name) {
            sel.remove();
            continue;
        }

        if sel.is("div,section,header,h1,h2,h3,h4,h5,h6") && is_element_without_content!(&sel) {
            sel.remove();
            continue;
        }
    }

    for sel in doc.select("*").iter().into_iter() {
        if sel.is("div") {
            if has_single_p_inside_element!(&sel) {
                let node = sel.children();
                // sel.replace_with_html()
            }
        }
    }
}

fn main() {
    let start = Instant::now();
    let html_file_path = env::args().skip(1).next().unwrap();
    let mut html = String::new();
    let mut html_file = File::open(&html_file_path).expect("correct HTML file path");
    html_file
        .read_to_string(&mut html)
        .expect("read HTML page file");

    let document = Document::from_str(&html);

    remove_script(&document);
    remove_style(&document);
    prep_document(&document);

    let metadata = get_article_metadata(&document);

    // println!("{}", document.html());
    println!("{:?}", metadata);
    println!("cost {:?}", start.elapsed());
}
