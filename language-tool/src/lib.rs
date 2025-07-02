use std::ops::Range;
use std::path::{Path, PathBuf};
use tree_sitter::Parser;

pub use tree_sitter::Node;

pub mod loader;
pub mod tools;

pub type GrammarName = &'static str;
pub type SliceRange = Range<usize>;

pub struct File {
    path: PathBuf,
    source: String,
}

impl File {
    pub fn new(path: PathBuf, source: String) -> Self {
        Self { path, source }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn node_text(&self, item: Node<'_>) -> &str {
        item.utf8_text(self.source.as_bytes()).unwrap()
    }
}

pub type Chain = Vec<(GrammarName, Range<usize>)>;

pub trait Scanner {
    /// returning `false` skip descendents
    /// chain gives information about location
    fn scan_tree<'a>(&mut self, _file: &'a File, _item: Node<'a>, _chain: &Chain) -> bool {
        false
    }

    /// chain gives information about location
    fn scan_leaf<'a>(&mut self, file: &'a File, item: Node<'a>, chain: &Chain);
}

pub fn scan(file: &File, parser: &mut Parser, scanner: &mut impl Scanner) {
    fn flat_walk<'a>(
        file: &File,
        node: tree_sitter::Node<'a>,
        scanner: &mut impl Scanner,
        chain: &mut Chain,
    ) {
        if node.child_count() > 0 {
            let result = scanner.scan_tree(file, node, chain);
            if !result {
                let range = node.start_byte()..node.end_byte();
                chain.push((node.grammar_name(), range));

                let mut walker = node.walk();
                for child in node.children(&mut walker) {
                    flat_walk(file, child, scanner, chain);
                }
                chain.pop();
            }
        } else {
            scanner.scan_leaf(file, node, chain);
        }
    }

    let tree = parser.parse(&file.source, None).unwrap();
    let root = tree.root_node();
    flat_walk(file, root, scanner, &mut Vec::new());
}
