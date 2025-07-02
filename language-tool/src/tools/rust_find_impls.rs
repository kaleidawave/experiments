use crate::{Chain, File, Node, Scanner, SliceRange};

#[derive(Default, Debug)]
pub struct FindRustImplementations {
    pub filter: Option<String>,
    pub found: Vec<(String, String, SliceRange)>,
}

impl Scanner for FindRustImplementations {
    fn scan_tree<'a>(&mut self, file: &'a File, node: Node<'a>, _chain: &Chain) -> bool {
        if let "source_file" = node.grammar_name() {
            let mut walker = node.walk();
            for child in node.children(&mut walker) {
                if let "impl_item" = child.grammar_name() {
                    if let Some(trait_item) = child.child_by_field_name("trait") {
                        let trait_name = file.node_text(trait_item);
                        if let Some(ref filter) = self.filter {
                            if !trait_name.ends_with(filter) {
                                continue;
                            }
                        }
                        let on_item = child.child_by_field_name("type").unwrap();
                        let name = file.node_text(on_item);
                        self.found.push((
                            trait_name.to_owned(),
                            name.to_owned(),
                            child.byte_range(),
                        ));
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
