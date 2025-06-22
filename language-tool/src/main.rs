use libloading::{Library, Symbol, library_filename};
use tree_sitter::{Language, Parser};
use tree_sitter_language::LanguageFn;

fn main() {
    let language = "rust";
    eprintln!("loading {language}");
    let module = format!("languages/{language}");
    let function_name = format!("tree_sitter_{language}");
    let lib = unsafe { Library::new(library_filename(&module)).unwrap() };
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
    println!("--- source_code ---\n{source_code}\n---");

    let transformed = Renamer::new("X".into(), "ABC".into()).run(source_code, &mut parser);

    println!();
    println!("--- transformed ---\n{transformed}\n---");
}

#[derive(Debug)]
struct Item<'a> {
    pub grammar_name: &'static str,
    pub source: &'a str,
    pub range: (usize, usize),
}

trait Scanner<'a> {
    fn recieve_item(&mut self, item: Item<'a>);
}

struct Renamer {
    from: String,
    to: String,
}

impl Renamer {
    pub fn new(from: String, to: String) -> Self {
        Self { from, to }
    }

    pub fn run(&self, on: &str, parser: &mut Parser) -> String {
        struct Finder<'f> {
            from: &'f str,
            matches: Vec<(usize, usize)>,
        }

        impl<'a, 'f> Scanner<'a> for Finder<'f> {
            fn recieve_item(&mut self, item: Item<'a>) {
                if item.grammar_name == "identifier" && item.source == self.from {
                    self.matches.push(item.range)
                }
            }
        }

        let mut scanner = Finder {
            from: &self.from,
            matches: Vec::new(),
        };
        scan(on, parser, &mut scanner);

        let mut last = 0;
        // TODO `Cow`
        let mut s = String::new();
        for (l, r) in scanner.matches.into_iter() {
            s.push_str(&on[last..l]);
            s.push_str(&self.to);
            last = r;
        }
        s.push_str(&on[last..]);
        s
    }
}

fn scan<'a>(source_code: &'a str, parser: &mut Parser, scanner: &mut impl Scanner<'a>) {
    fn flat_walk<'a, 's>(
        source_code: &'s str,
        node: tree_sitter::Node<'a>,
        scanner: &mut impl Scanner<'s>,
    ) {
        if node.child_count() > 0 {
            let mut walker = node.walk();
            for child in node.children(&mut walker) {
                flat_walk(source_code, child, scanner);
            }
        } else {
            scanner.recieve_item(Item {
                grammar_name: node.grammar_name(),
                source: &source_code[node.start_byte()..node.end_byte()],
                range: (node.start_byte(), node.end_byte()),
            });
        }
    }

    let tree = parser.parse(source_code, None).unwrap();
    let root = tree.root_node();
    flat_walk(source_code, root, scanner);
}
