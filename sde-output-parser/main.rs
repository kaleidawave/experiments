use std::fs;
use std::io::{BufRead, BufReader};

const FILE: &str = "temp1928328.txt";

fn main() {
    let mut args = std::env::args().skip(1);

    let path = args.next().expect("no path or run");

    if path == "run" {
        // TODO combinations etc here
        let mut command = std::process::Command::new("sde");
        command.args(["-omix", FILE, "-mix_filter_no_shared_libs", "--"]);
        command.args(args);
        let _ = command.output().unwrap();
        let file = fs::File::open(FILE).unwrap();
        parse_and_print(BufReader::new(file));
        fs::remove_file(FILE).unwrap();
    } else {
        let file = fs::File::open(path).unwrap();
        parse_and_print(BufReader::new(file));
    }
}

fn parse_and_print(out: impl BufRead) {
    let mut parts = sde_output_parser::parse(out);

    let max_count = {
        let mut max_count = 0;
        for (key, _) in parts.iter() {
            max_count = std::cmp::max(max_count, key.len());
        }
        std::cmp::min(max_count, 50)
    };

    parts.sort_by_key(|(name, _)| name.clone());

    for (section, count) in parts {
        let section: std::borrow::Cow<'_, str> = if section.len() > 50 {
            std::borrow::Cow::Owned(format!("{prefix}...", prefix = &section[..47]))
        } else {
            std::borrow::Cow::Borrowed(&section)
        };
        let fill =
            &"                                                  "[..max_count - section.len()];
        println!("{section}:{fill} {count:?}");
    }
}
