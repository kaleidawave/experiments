mod template;

use std::borrow::Cow;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);

    let code = std::fs::read_to_string(args.next().expect("expected file")).unwrap();
    let location_arg = args.next().unwrap_or(".".into());
    let location = std::path::Path::new(&location_arg);

    std::fs::create_dir_all(&location)?;

    let mut map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    let err = simple_yaml_parser::parse(&code, |keys, value| {
        if let &[simple_yaml_parser::YAMLKey::Slice(path)] = keys {
            if let simple_yaml_parser::RootYAMLValue::MultiLineString(value) = value {
                let template = template::Template::new(&value.on);
                let result = template.interpolate(|key| {
                    if key == "name" {
                        Cow::Borrowed(&location_arg)
                    } else if let Some(existing) = map.get(key) {
                        Cow::Owned(existing.clone())
                    } else {
                        let mut out = String::new();
                        print!("> {key}=");
                        let _ = std::io::stdout().flush();
                        let _ = multiline_term_input::read_string(&mut std::io::stdin(), &mut out);
                        map.insert(key.to_owned(), out.clone());
                        Cow::Owned(out)
                    }
                });

                let path = location.join(path);
                eprintln!("Writing to {path}", path = path.display());
                let result = std::fs::write(path, result);
                let _ = dbg!(result);
            } else {
                eprintln!("err: {value:?}");
            }
        } else {
            eprintln!("err: {keys:?}");
        }
    });

    if let Err(err) = err {
        eprintln!("{err:?}");
    }

    Ok(())
}
