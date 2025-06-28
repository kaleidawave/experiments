// use libloading::{Library, Symbol, library_filename};
use std::ops::Range;
use tree_sitter::{Node, Parser}; // use tree_sitter::{Language, Node, Parser};
// use tree_sitter_language::LanguageFn;

pub type GrammarName = &'static str;

#[derive(Debug)]
pub struct Item<'s, 'a> {
    pub total_source: &'s str,
    pub raw_node: Node<'a>,
}

impl<'s, 'a> Item<'s, 'a> {
    pub fn source(&self) -> &'s str {
        &self.total_source[self.range()]
    }

    pub fn range(&self) -> Range<usize> {
        self.raw_node.start_byte()..self.raw_node.end_byte()
    }

    pub fn grammar_name(&self) -> &'static str {
        self.raw_node.grammar_name()
    }
}

pub type Chain = Vec<(GrammarName, Range<usize>)>;

pub trait Scanner<'s> {
    /// returning `false` skip descendents
    /// chain gives information about location
    fn scan_tree<'a>(&mut self, _item: Item<'s, 'a>, _chain: &Chain) -> bool {
        false
    }

    /// chain gives information about location
    fn scan_leaf<'a>(&mut self, item: Item<'s, 'a>, chain: &Chain);
}

pub fn scan<'a>(source_code: &'a str, parser: &mut Parser, scanner: &mut impl Scanner<'a>) {
    fn flat_walk<'a, 's>(
        source_code: &'s str,
        node: tree_sitter::Node<'a>,
        scanner: &mut impl Scanner<'s>,
        chain: &mut Chain,
    ) {
        if node.child_count() > 0 {
            let item = Item {
                total_source: source_code,
                raw_node: node,
            };
            let result = scanner.scan_tree(item, chain);
            if !result {
                let range = node.start_byte()..node.end_byte();
                chain.push((node.grammar_name(), range));

                let mut walker = node.walk();
                for child in node.children(&mut walker) {
                    flat_walk(source_code, child, scanner, chain);
                }
                chain.pop();
            }
        } else {
            let item = Item {
                total_source: source_code,
                raw_node: node,
            };
            scanner.scan_leaf(item, chain);
        }
    }

    let tree = parser.parse(source_code, None).unwrap();
    let root = tree.root_node();
    flat_walk(source_code, root, scanner, &mut Vec::new());
}
