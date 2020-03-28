use crate::matcher::SelectorParser;
use cssparser::{Parser, ParserInput};
use selectors::parser::SelectorList;

struct Selection;

impl Selection {
    fn find(&self, selector: &str) -> Self {
        let mut parser_input = ParserInput::new(selector);
        let mut parser = Parser::new(&mut parser_input);

        let selector = SelectorList::parse(&SelectorParser, &mut parser).unwrap();

        return Selection {};
    }
}
