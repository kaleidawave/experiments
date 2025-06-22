use libloading::{Library, Symbol};
use tree_sitter::{Language, Parser};
use tree_sitter_language::LanguageFn;

fn main() {
    let language = "rust";
    eprintln!("loading {language}");
    let module = format!("languages/{language}.dll");
    let function_name = format!("tree_sitter_{language}");
    let lib = unsafe { Library::new(&module).unwrap() };
    let function = unsafe {
        lib.get::<Symbol<extern "C" fn() -> *const ()>>(function_name.as_bytes())
            .unwrap()
    };
    let language_fn = unsafe { LanguageFn::from_raw(**function) };
    eprintln!("loaded {function_name} from {module}");

    let mut parser = Parser::new();
    parser
        .set_language(&Language::from(language_fn))
        .expect(&format!("Error loading {language:?} grammar"));

    eprintln!("initiated parser");

    let source_code = "
enum X {
    Some(Box<X>),
    None
}

let x = X::None;
"
    .trim();

    let list = scan(source_code, &mut parser);
    println!("--- source_code ---\n{source_code}\n---");
    for item in list {
        println!("{item:?}");
    }
}

#[derive(Debug)]
struct Item<'a> {
    pub grammar_name: &'static str,
    pub source: &'a str,
    pub range: (usize, usize),
}

fn scan<'a>(source_code: &'a str, parser: &mut Parser) -> Vec<Item<'a>> {
    fn flat_walk<'a, 's>(
        source_code: &'s str,
        node: tree_sitter::Node<'a>,
        list: &mut Vec<Item<'s>>,
    ) {
        if node.child_count() > 0 {
            let mut walker = node.walk();
            for child in node.children(&mut walker) {
                flat_walk(source_code, child, list);
            }
        } else {
            list.push(Item {
                grammar_name: node.grammar_name(),
                source: &source_code[node.start_byte()..node.end_byte()],
                range: (node.start_byte(), node.end_byte()),
            });
        }
    }

    let tree = parser.parse(source_code, None).unwrap();
    let root = tree.root_node();

    let mut list = Vec::new();
    flat_walk(source_code, root, &mut list);
    list
}
