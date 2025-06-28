use libloading::{Library, Symbol, library_filename};
use tree_sitter::{Language, Parser};
use tree_sitter_language::LanguageFn;

use language_tool::{Chain, Item, Scanner, scan};

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::{
    self, Config,
    termcolor::{ColorChoice, StandardStream},
};

use std::path::Path;

fn main() {
    let language = "rust";
    eprintln!("loading {language}");
    let module = std::path::Path::new("languages").join(library_filename(&language));
    let function_name = format!("tree_sitter_{language}");
    let lib = unsafe { Library::new(&module).unwrap() };
    let function = unsafe {
        lib.get::<Symbol<extern "C" fn() -> *const ()>>(function_name.as_bytes())
            .unwrap()
    };
    let language_fn = unsafe { LanguageFn::from_raw(**function) };
    eprintln!(
        "loaded {function_name} from {module}",
        module = module.display()
    );

    let mut parser = Parser::new();
    parser
        .set_language(&Language::from(language_fn))
        .expect(&format!("Error loading {language:?} grammar"));

    eprintln!("initiated parser");

    let mut args = std::env::args().skip(1);

    let first = args.next();
    match first.as_deref() {
        None | Some("info" | "--help") => {
            eprintln!("The language-tool")
        }
        Some("debug") => {
            let path = args.next().expect("path");
            // let arg = args.next().expect("path or content");
            // let source_code = if arg.contains('\n') || !std::path::Path::new(&arg).exists() {
            //     arg
            // } else {
            //     std::fs::read_to_string(arg).unwrap()
            // };

            visit_files(Path::new(&path), &mut |path| {
                eprintln!("--- {path} ---", path = path.display().to_string());
                let content = std::fs::read_to_string(&path).unwrap();

                let file = SimpleFile::new(path.display().to_string(), content.clone());
                let writer = StandardStream::stderr(ColorChoice::Always);
                let config = Config::default();

                let mut debugger = Debugger {
                    file,
                    writer,
                    config,
                };

                scan(&content, &mut parser, &mut debugger);
            })
            .expect("Could not visit files");
        }
        Some(command) => {
            eprintln!("Unknown command {command:?}");
        }
    }
}

struct Debugger {
    file: SimpleFile<String, String>,
    writer: StandardStream,
    config: Config,
}

impl<'s> Scanner<'s> for Debugger {
    fn scan_tree<'a>(&mut self, item: Item<'s, 'a>, _chain: &Chain) -> bool {
        if let "enum_item" | "struct_item" = item.grammar_name() {
            let name = item.raw_node.child_by_field_name("name").unwrap();
            let name = &item.total_source[name.start_byte()..name.end_byte()];
            let label = Label::primary((), item.range()).with_message(format!("Found {name}"));
            let diagnostic = Diagnostic::note().with_labels(vec![label]);
            term::emit(
                &mut self.writer.lock(),
                &self.config,
                &self.file,
                &diagnostic,
            )
            .unwrap();
            true
        } else {
            false
        }
    }

    fn scan_leaf<'a>(&mut self, _item: Item<'s, 'a>, _chain: &Chain) {}
    //     if item.grammar_name() == "identifier" {
    //         let diagnostic = Diagnostic::error().with_labels(vec![
    //             Label::primary((), item.range())
    //                 .with_message(format!("Found tag {item:?}", item = item.source())),
    //         ]);
    //         term::emit(
    //             &mut self.writer.lock(),
    //             &self.config,
    //             &self.file,
    //             &diagnostic,
    //         ).unwrap();
    //     }
    // }
}

// struct Debugger;

// impl<'s> Scanner<'s> for Debugger {
//     fn scan_tree<'a>(&mut self, item: Item<'s, 'a>, chain: &Chain) -> bool {
//         let grammar_name = item.grammar_name();
//         // if grammar_name == "scoped_type_identifier" {
//         //     let mut walker = item.raw_node.walk();
//         //     if let Some(struct_parent) = item
//         //         .raw_node
//         //         .parent()
//         //         .and_then(|node| (node.grammar_name() == "struct_expression").then_some(node))
//         //     {
//         //         for child in item.raw_node.children(&mut walker) {
//         //             if child.grammar_name() == "identifier" {
//         //                 let name = &item.total_source[child.start_byte()..child.end_byte()];
//         //                 eprint!("::{name}");
//         //             }
//         //         }
//         //         eprintln!();
//         //         let field_initializer = struct_parent.child_by_field_name("body");
//         //         if let Some(field_initializer) = field_initializer {
//         //             let item=&item.total_source[field_initializer.start_byte()..field_initializer.end_byte()];
//         //             eprintln!("{item} (wrap in Box::new)");
//         //         }
//         //     }
//         //     true
//         // } else if grammar_name == "enum_variant" {
//         //     let variant_name = item.raw_node.child_by_field_name("name").unwrap();
//         //     let body = item.raw_node.child_by_field_name("body").unwrap();

//         //     let parent = item.raw_node.parent().unwrap().parent().unwrap();
//         //     let enum_name = parent.child_by_field_name("name").unwrap();

//         //     eprintln!(
//         //         "enum {enum_name} variant {variant_name} with {body}",
//         //         enum_name = &item.total_source[enum_name.start_byte()..enum_name.end_byte()],
//         //         variant_name = &item.total_source[variant_name.start_byte()..variant_name.end_byte()],
//         //         body = &item.total_source[body.start_byte()..body.end_byte()]
//         //     );
//         //     true
//         // } else {
//         //     false
//         // }
//     }

//     fn scan_leaf<'a>(&mut self, _item: Item<'s, 'a>, _chain: &Chain) {
//         // if item.grammar_name() == "identifier" {
//         //     let identifier = item.source();
//         //     eprintln!("{chain:?} -> {identifier}");
//         //     let ctx = chain.last().unwrap().0;
//         //     match ctx {
//         //         "enum_variant" => {
//         //             let range = chain.last().unwrap().1.clone();
//         //             let source = &item.total_source[range];
//         //             eprintln!("-> {source:?}");
//         //             // if let Some(next) = item.next_sibling() {
//         //             // } else {
//         //             //     eprintln!("no next")
//         //             // }
//         //         }
//         //         "scoped_type_identifier" => {
//         //             // if identifier.starts_with(|chr: char| chr.is_uppercase()) {
//         //             //     let braced_enum_member = item.raw_node.next_sibling().is_some_and(|node| {
//         //             //         node.grammar_name() == "::"
//         //             //             && node.next_sibling().is_some_and(|node| {
//         //             //                 node.grammar_name() == "identifier"
//         //             //                     && &item.total_source[node.range().start_byte..])
//         //             //                         .starts_with(|chr: char| chr.is_uppercase())
//         //             //             })
//         //             //     }));
//         //             //     // TODO want range
//         //             //     if braced_enum_member {
//         //             //         eprintln!("FOUND BRACED ENUM MEMBER {identifier}")
//         //             //     }
//         //             // }
//         //         }
//         //         _ => {}
//         //     }
//         // }
//     }
// }

pub fn visit_files(
    path: &std::path::Path,
    cb: &mut dyn FnMut(&std::path::Path),
) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_files(&path, cb)?;
            } else {
                cb(&path);
            }
        }
    } else if path.is_file() {
        let skip = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_none_or(|ext| !matches!(ext, "rs"));
        if !skip {
            cb(&path)
        }
    } else if path.is_symlink() {
        let path = std::fs::read_link(path)?;
        visit_files(&path, cb)?;
    }
    Ok(())
}
