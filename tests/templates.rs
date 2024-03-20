use markup5ever::interface::TreeSink;
use nipper::{Document, ParseOptions};

#[test]
fn test_templates() {
    let mut doc = Document::parse(
        ParseOptions {
            keep_templates: true,
        },
        r#"
<html>
    <body>
        <template>
            <p>Hello world!</p>
        </template>
    </body>
</html>
"#,
    );

    let nodes = doc
        .select("template")
        .nodes()
        .iter()
        .map(|n| n.id)
        .collect::<Vec<_>>();
    for node in nodes {
        let c = doc.get_template_contents(&node);
        println!("Node: {c:?}");
        let data = doc.get_node(&node);
        println!("Data: {data:?}");
    }

    let result = doc.html().to_string();
    assert_eq!(
        result,
        r#"<html><head></head><body>
        <template>
            <p>Hello world!</p>
        </template>
    

</body></html>"#
    );
}
