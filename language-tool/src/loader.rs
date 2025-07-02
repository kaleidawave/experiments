use libloading::{Library, Symbol, library_filename};
use tree_sitter::{Language, Parser as TreeSitterParser};
use tree_sitter_language::LanguageFn;

// unsafe extern "C" {
//     fn tree_sitter_rust() -> *const ();
// }

pub struct Parser {
    #[allow(unused)]
    library: Library,
    pub parser: TreeSitterParser,
}

pub fn get_rust_parser() -> Parser {
    let language = "rust";
    eprintln!("loading {language}");
    let module = std::path::Path::new("languages").join(library_filename(language));
    let function_name = format!("tree_sitter_{language}");
    let library = unsafe { Library::new(&module).unwrap() };
    let function = unsafe {
        library
            .get::<Symbol<extern "C" fn() -> *const ()>>(function_name.as_bytes())
            .unwrap()
    };
    let language_fn = unsafe { LanguageFn::from_raw(**function) };
    eprintln!(
        "loaded {function_name} from {module}",
        module = module.display()
    );

    // let language_fn = unsafe { LanguageFn::from_raw(tree_sitter_rust) };

    let mut parser = TreeSitterParser::new();
    parser
        .set_language(&Language::from(language_fn))
        .unwrap_or_else(|_| panic!("Error loading {language:?} grammar"));

    Parser { library, parser }
}
