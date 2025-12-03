use simple_markdown_parser::emit::{FeatureEmitter, markdown_to_html};
use simple_markdown_parser::{Frontmatter, ParseOptions, simple_yaml_parser};
use std::path::Path;

fn main() {
    todo!();
}

#[derive(Debug)]
pub struct Template {
    includes: Vec<Include>,
    buf: String,
    // TODO
    footer: String,
    header: String,
}

#[derive(Debug)]
pub enum Kind {
    StyleSheet,
    Script,
}

#[derive(Debug)]
pub enum Include {
    Reference { path: String, kind: Kind },
    Content { content: String, kind: Kind },
}

use std::fmt::Write;

impl Template {
    pub fn new(includes: Vec<Include>, header: String, footer: String) -> Self {
        let start = r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport"content="width=device-width,initial-scale=1.0">"#;
        let buf = start.to_owned();
        Self {
            buf,
            header,
            footer,
            includes,
        }
    }

    pub fn title(&mut self, title: &str) {
        // TODO metadata
        write!(&mut self.buf, "<title>{title}</title>").unwrap();
    }

    pub fn finish_head(&mut self) {
        for include in self.includes.drain(..) {
            match include {
                Include::Content {
                    content,
                    kind: Kind::StyleSheet,
                } => {
                    write!(&mut self.buf, "<style>{content}</style>").unwrap();
                }
                include => todo!("{include:?}"),
            }
        }
        write!(&mut self.buf, "</head>").unwrap();
        write!(
            &mut self.buf,
            "<header>{header}</header>",
            header = &self.header
        )
        .unwrap();
    }

    pub fn finish(mut self, path: &str) {
        write!(
            &mut self.buf,
            "<footer>{footer}</footer>",
            footer = self.footer
        )
        .unwrap();
        write!(&mut self.buf, "</body></html>").unwrap();
        std::fs::write(path, self.buf).unwrap();
    }
}

impl Write for Template {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        String::write_str(&mut self.buf, s)
    }
}

impl FeatureEmitter for Template {
    fn frontmatter(&mut self, frontmatter: Frontmatter<'_>) {
        let _ = frontmatter.parse_yaml(|key, value| {
            if let [simple_yaml_parser::YAMLKey::Slice("title")] = key {
                let simple_yaml_parser::RootYAMLValue::String(value) = value else {
                    panic!("bad value");
                };
                self.title(&value);
            } else {
                println!("TODO {key:?} -> {value:?}");
            }
        });
        self.finish_head();
        // TODO
    }

    fn code_block(&mut self, language: &str, code: &str) {
        eprintln!("TODO code block {language} {code:?}");
    }

    fn mathematics(&mut self, code: &str, display: bool) {
        eprintln!("TODO {code} {display:?}");
    }

    fn command(&mut self, name: &str, args: Vec<(&str, &str)>, inner: &str, options: ParseOptions) {
        eprintln!("TODO command {name}");
    }

    fn interpolation(&mut self, expression: &str) {
        eprintln!("TODO interpolation{expression}");
    }
}
