use libloading::{Library, Symbol};
use tree_sitter::{Language, Parser};
use tree_sitter_language::LanguageFn;

fn main() {
    let language = "rust";
    eprintln!("loading {language}");
    let module = format!("languages/{language}.dll");
    let function = format!("tree_sitter_{language}");
    let language_fn: LanguageFn = unsafe {
        let lib = Library::new(&module).unwrap();
        let function = lib
            .get::<Symbol<extern "C" fn() -> *const ()>>(function.as_bytes())
            .unwrap();
        LanguageFn::from_raw(**function)
    };
    eprintln!("loaded {function} from {module}");

    let mut parser = Parser::new();
    parser
        .set_language(&Language::from(language_fn))
        .expect(&format!("Error loading {language:?} grammar"));

    eprintln!("initiated parser");

    let source_code = "let x = 2;";

    let tree = parser.parse(source_code, None).unwrap();
    let root = tree.root_node();

    eprintln!("root: {:?}", root.kind());
}
