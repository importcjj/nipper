use crate::document::NodeRef;
use crate::document::Tree;

use std::cell::RefCell;

pub struct Selection<'a, T> {
    nodes: Vec<NodeRef<'a, T>>,
    tree: RefCell<Tree<T>>,
}

// impl<'a, T> Selection<'a, T> {
//     fn find(&self, selector: &str) -> Self {
//         let mut parser_input = ParserInput::new(selector);
//         let mut parser = Parser::new(&mut parser_input);

//         let selector = SelectorList::parse(&SelectorParser, &mut parser).unwrap();

//         return Selection {};
//     }
// }
