//! HTML parsing，querying and manipulation with CSS selectors.

#![deny(missing_docs)]
mod document;
mod dom_tree;
mod element;
mod manipulation;
mod matcher;
mod property;
mod query;
mod selection;
mod traversal;

pub use document::Document;
pub use dom_tree::Node;
pub use dom_tree::NodeId;
#[doc(hidden)]
pub use dom_tree::SerializableNodeRef;
pub use selection::Selection;
