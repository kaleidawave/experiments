use std::ops::Range;

use crate::{Chain, File, Node, Scanner};

#[derive(Default, Debug)]
pub struct FindRustMods {
    pub files: Vec<(String, String)>,
    pub remove: Vec<Range<usize>>,
}

impl Scanner for FindRustMods {
    fn scan_tree<'a>(&mut self, file: &'a File, node: Node<'a>, _chain: &Chain) -> bool {
        if let "source_file" = node.grammar_name() {
            let mut walker = node.walk();
            for child in node.children(&mut walker) {
                if let "mod_item" = child.grammar_name() {
                    if let Some(body) = child.child_by_field_name("body") {
                        let name = child.child_by_field_name("name").unwrap();
                        let name = file.node_text(&name);

                        self.remove.push(body.byte_range());

                        let body = file.node_text(&body);
                        let body = body[1..body.len().saturating_sub(2)].to_owned();
                        self.files.push((name.to_owned(), body));
                    }
                }
            }
            true
        } else {
            false
        }
    }

    fn scan_leaf<'a>(&mut self, _file: &'a File, _node: Node<'a>, _chain: &Chain) {}
}
