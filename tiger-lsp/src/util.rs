pub use ahash::AHashMap as HashMap;
pub use ahash::AHashSet as HashSet;

use termtree::Tree;

use crate::loca::Node;

pub fn tree(node: &Node, line: &str) -> Tree<String> {
    Tree::new(format!("{}: {}  ({})", node.span.extract(line), node.kind, node.span))
        .with_leaves(node.content.iter().map(|n| tree(n, line)))
}
