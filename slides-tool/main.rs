use simple_markdown_parser::utilities::extract_slides;

use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args().skip(1).rev().collect();
    let content = if let Some(path) = args.pop() {
        let content = std::fs::read_to_string(path)?;
        content
    } else {
        "# Hello world".to_string()
    };

    let out = args
        .pop()
        .map(std::path::PathBuf::from)
        .unwrap_or("./private/html/demo.html".into());

    let style_;

    let style_file = args.windows(2).find_map(|slice| {
        matches!(slice.get(1).map(String::as_str), Some("--style")).then_some(&slice[0])
    });
    let style = if let Some(style) = style_file {
        style_ = std::fs::read_to_string(style)?;
        style_.as_str()
    } else {
        include_str!("./include/slides.css")
    };

    let script = include_str!("./include/slides.js");

    let mathematics_and_code_highlighting = args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--render-math-and-code"));

    let additional_script = if mathematics_and_code_highlighting {
        r#"
        <script type="module">
            import { codeToHtml } from 'https://esm.sh/shiki@3.0.0'
            import renderMathInElement from "https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/contrib/auto-render.mjs";

            document.querySelectorAll("pre[data-language]").forEach(async element => {
                const language = element.getAttribute("data-language");
                element.innerHTML = await codeToHtml(element.innerText, {
                    lang: language,
                    theme: 'light-plus'
                })
            });

            document.querySelectorAll(".mathematics.inline").forEach(elem => {
                renderMathInElement(elem, { delimiters: [{ left: "$", right: "$", display: false }] });
            })
            document.querySelectorAll(".mathematics.block").forEach(elem => {
                renderMathInElement(elem, { delimiters: [{ left: "$$", right: "$$", display: true }] });
            })
        </script>
        "#
    } else {
        ""
    };

    let slides = extract_slides(&content);

    let mut file = std::fs::File::create(out).unwrap();
    writeln!(
        file,
        r#"<!DOCTYPE html>
        <html lang="en">
            <head><meta charset="UTF-8">
            <meta name="viewport".unwrap() content="width=device-width,initial-scale=1.0">
            <link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='1 -13 18 16'%3E%3Ctext%3E⭐%3C/text%3E%3C/svg%3E" />
            <title>Slides</title>
            <script type="module">
                {script}
            </script>
            {additional_script}
            <style>
                {style}
            </style>
        </head>
        <body>
            <div id="slides" data-paginated>"#
    ).unwrap();

    for slide in &slides {
        {
            let content = slide.markdown_content.trim();
            let is_frontmatter = content.starts_with("---") && content.ends_with("---");
            if is_frontmatter {
                continue;
            }
        }

        writeln!(file, "<section>")?;
        join(&slide.location(), &mut file);
        if let Some(last) = slide.location.last() {
            if !last.ends_with("#hidden") {
                writeln!(file, "<h3>{last}</h3>").unwrap();
            }
        }

        let options = simple_markdown_parser::ParseOptions::default();

        let _ = simple_markdown_parser::extras::emit::markdown_to_html(
            &slide.markdown_content,
            &mut file,
            &mut simple_markdown_parser::extras::emit::BlankFeatureEmitter,
            options,
            0,
        );

        writeln!(file, "</section>")?;
    }
    let controls = r#"<div id="controls">
        <button id="prev">previous</button>
        <button id="next">next</button>
    </div>"#;

    writeln!(
        file,
        "</div>
{controls}
</body>
</html>"
    )?;

    Ok(())
}

// TODO account for empty items
fn join<T: std::fmt::Display>(slice: &[T], on: &mut impl std::io::Write) {
    if let [slice @ .., _] = slice {
        write!(on, "<h5 class=\"location\">").unwrap();
        let mut iter = slice.iter();
        if let Some(part) = iter.next() {
            write!(on, "{part}").unwrap();
            for part in iter {
                write!(on, " / {part}").unwrap();
            }
        }
        write!(on, "</h5>").unwrap();
    }
}
