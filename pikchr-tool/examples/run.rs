use pikchr_tool::{utilities, Shape, Command, Connection};
 
 fn main() {
    let diagram = std::fs::read_to_string(
        std::env::args().nth(1).expect("expected first argument")
    )
    .unwrap();

    let mut commands = Vec::new();
    for o_line in utilities::LinesWithContinuation::new(&diagram, &["\\"]) {
        let line = o_line.trim_start();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let first = line
            .find(|chr: char| !chr.is_alphanumeric())
            .unwrap_or(line.len());
        let (kw, after) = line.split_at(first);
        let (kw, after) = if let Some(after) = after.strip_prefix(':') {
            commands.push(Command::Label(kw));
            let after = after.trim_start();
            let first = after
                .find(|chr: char| !chr.is_alphanumeric())
                .unwrap_or(after.len());
            after.split_at(first)
        } else {
            (kw, after)
        };

        let after = after.trim_start();

        /// TODO Cow and append while
        fn parse_text(after: &str) -> (Option<&str>, &str) {
            if let Some(after) = after.strip_prefix('"') {
                let (text, after) = after.split_once('"').unwrap();
                (Some(text), after)
            } else {
                (None, after)
            }
        }

        match kw {
            "box" | "circle" | "oval" | "ellipse" | "cylinder" | "file" | "diamond" => {
                let shape = match kw {
                    "box" => Shape::Box,
                    "circle" => Shape::Circle,
                    "ellipse" => Shape::Ellipse,
                    "oval" => Shape::Oval,
                    "cylinder" => Shape::Cylinder,
                    "file" => Shape::File,
                    "diamond" => Shape::Diamond,
                    shape => unreachable!("{shape}"),
                };
                let (text, after) = parse_text(after);
                let after = after.trim_start();
                commands.push(Command::Shape { shape, text, after });
            }
            "line" | "arrow" | "spline" => {
				let connection = match kw {
                    "line" => Connection::Line,
                    "arrow" => Connection::Arrow,
                    "spline" => Connection::Spline,
                    connection => unreachable!("{connection}"),
                };
                // TODO after
                commands.push(Command::Connection {
					connection,
					from: None
				});
            }
            kw => {
                commands.push(Command::Other { kw, after });
            }
        }
    }
    for command in commands {
        println!("{command:?}");
    }
}