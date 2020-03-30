use html5ever::driver;
use html5ever::tendril::TendrilSink;
use html5ever::ExpandedName;
use html5ever::QualName;
use markup5ever::interface::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use markup5ever::{local_name, namespace_url, ns, Attribute};
use rsquery::{Document, NodeId};
use std::borrow::Cow;
use tendril::StrTendril;

struct LineCountingDOM {
    pub line_vec: Vec<(QualName, u64)>,
    pub dom: Document,
    pub current_line: u64,
}

impl TreeSink for LineCountingDOM {
    type Output = Self;

    fn finish(self) -> Self {
        self
    }

    type Handle = NodeId;

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        self.dom.parse_error(msg);
    }

    fn get_document(&mut self) -> NodeId {
        self.dom.get_document()
    }

    fn get_template_contents(&mut self, target: &NodeId) -> NodeId {
        self.dom.get_template_contents(target)
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.dom.set_quirks_mode(mode)
    }

    fn same_node(&self, x: &NodeId, y: &NodeId) -> bool {
        self.dom.same_node(x, y)
    }

    fn elem_name<'a>(&'a self, target: &'a NodeId) -> ExpandedName<'a> {
        self.dom.elem_name(target)
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> NodeId {
        self.line_vec.push((name.clone(), self.current_line));
        self.dom.create_element(name, attrs, flags)
    }

    fn create_comment(&mut self, text: StrTendril) -> NodeId {
        self.dom.create_comment(text)
    }

    fn create_pi(&mut self, target: StrTendril, content: StrTendril) -> NodeId {
        self.dom.create_pi(target, content)
    }

    fn append(&mut self, parent: &NodeId, child: NodeOrText<NodeId>) {
        self.dom.append(parent, child)
    }

    fn append_before_sibling(&mut self, sibling: &NodeId, child: NodeOrText<NodeId>) {
        self.dom.append_before_sibling(sibling, child)
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &NodeId,
        prev_element: &NodeId,
        child: NodeOrText<NodeId>,
    ) {
        self.dom
            .append_based_on_parent_node(element, prev_element, child)
    }

    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        self.dom
            .append_doctype_to_document(name, public_id, system_id);
    }

    fn add_attrs_if_missing(&mut self, target: &NodeId, attrs: Vec<Attribute>) {
        self.dom.add_attrs_if_missing(target, attrs);
    }

    fn remove_from_parent(&mut self, target: &NodeId) {
        self.dom.remove_from_parent(target);
    }

    fn reparent_children(&mut self, node: &NodeId, new_parent: &NodeId) {
        self.dom.reparent_children(node, new_parent);
    }

    fn mark_script_already_started(&mut self, target: &NodeId) {
        self.dom.mark_script_already_started(target);
    }

    fn set_current_line(&mut self, line_number: u64) {
        self.current_line = line_number;
    }
}

#[test]
fn check_four_lines() {
    // Input
    let sink = LineCountingDOM {
        line_vec: vec![],
        current_line: 1,
        dom: Document::default(),
    };
    let mut result_tok = driver::parse_document(sink, Default::default());
    result_tok.process(StrTendril::from("<a>\n"));
    result_tok.process(StrTendril::from("</a>\n"));
    result_tok.process(StrTendril::from("<b>\n"));
    result_tok.process(StrTendril::from("</b>"));
    // Actual Output
    let actual = result_tok.finish();
    // Expected Output
    let expected = vec![
        (QualName::new(None, ns!(html), local_name!("html")), 1),
        (QualName::new(None, ns!(html), local_name!("head")), 1),
        (QualName::new(None, ns!(html), local_name!("body")), 1),
        (QualName::new(None, ns!(html), local_name!("a")), 1),
        (QualName::new(None, ns!(html), local_name!("b")), 3),
    ];
    // Assertion
    assert_eq!(actual.line_vec, expected);
}
