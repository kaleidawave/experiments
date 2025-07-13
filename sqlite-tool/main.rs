use std::{env, fs, path};

mod utilities;

fn main() {
    let mut args = env::args().skip(1);

    let arg = args.next().expect("path@table to database");
    let (path, table) = arg
        .split_once('@')
        .unwrap_or((&arg, file_name(&arg).unwrap_or("table")));
    let connection = sqlite3::open(path).unwrap();

    let command = args.next();

    match command.as_deref() {
        None | Some("info") | Some("--help") => {
            println!("wrapper for sqlite");
        }
        Some("insert" | "load") => {
            let path = args.next().expect("path");
            let path = path::Path::new(&path);
            let extension = path.extension().and_then(|ext| ext.to_str());
            let content = fs::read_to_string(path).unwrap();
            match extension {
                Some("csv") => {
                    todo!()
                }
                Some("csvts") => {
                    let mut current_insert_statement = None;
                    for line in content.lines() {
                        if line.is_empty() {
                            continue;
                        }
                        let is_table_declaration = utilities::strip_around(line, "---");
                        if let Some(item) = is_table_declaration {
                            let (table_name, schema) = utilities::seperate_off_parenthesised(item);
                            if let Some(schema) = schema {
                                {
                                    let query = format!(
                                        "CREATE TABLE IF NOT EXISTS {table_name} ({schema});"
                                    );
                                    connection.execute(query).unwrap();
                                }
                                {
                                    let num_values = utilities::csv::CSVColumns::new(line).count();
                                    let interpolations =
                                        utilities::repeate::Repeat::new("?", ",", num_values);
                                    let query = format!(
                                        "INSERT INTO {table_name} VALUES ({interpolations});"
                                    );
                                    let statement = connection.prepare(&query).unwrap();
                                    current_insert_statement = Some(statement);
                                }
                            } else {
                                todo!("get column length?");
                                // current_insert_statement = Some(table_name);
                            }
                        } else {
                            let statement = current_insert_statement.as_mut().unwrap();

                            // TODO is there a better way?
                            for (idx, value) in utilities::csv::CSVColumns::new(line).enumerate() {
                                if let Ok(value) = value.parse::<i64>() {
                                    statement.bind(idx + 1, value).unwrap();
                                } else if let Ok(value) = value.parse::<f64>() {
                                    statement.bind(idx + 1, value).unwrap();
                                } else {
                                    statement.bind(idx + 1, &*value).unwrap();
                                }
                            }

                            statement.next().expect("could not insert data");
                            // remove bindings
                            statement.reset().expect("could not reset statement");
                        }
                    }
                }
                Some("json") => {
                    todo!()
                }
                extension => panic!("Unknown {extension:?}"),
            }
        }
        Some("dump") => {
            todo!("pull to json, csv, etc. Also filter and limit")
        }
        Some("raw") => {
            let query = {
                use std::io;

                print!("> ");
                std::io::Write::flush(&mut io::stdout()).unwrap();
                let mut input = String::new();
                let std_in = &mut io::stdin();

                // multiline_term_input only works on windows for now
                #[cfg(target_family = "windows")]
                let _n = multiline_term_input::read_string(std_in, &mut input);

                #[cfg(target_family = "unix")]
                let _n = std_in.read_line(&mut input).unwrap();

                input
            };

            // TODO template

            connection
                .iterate(query, |pairs| {
                    if pairs.len() == 1 {
                        let value = pairs.iter().next().unwrap().1.unwrap();
                        println!("{value}");
                    } else {
                        for (idx, (name, value)) in pairs.iter().enumerate() {
                            if idx > 0 {
                                print!(", ");
                            }
                            print!("{name}={value}", value = value.unwrap_or("*null*"));
                        }
                    }
                    true
                })
                .unwrap();
        }
        Some("retrieve") => {
            let query = args.next();
            match query.as_deref() {
                Some("random") => {
                    let mut field = String::from("*"); // TODO Cow
                    let mut filter = String::new();
                    let mut pick: usize = 1;

                    let mut mark_as_read = false;
                    let mut write_to_file: Option<fs::File> = None;
                    let mut template: Option<utilities::template::Template> = None;
                    let mut _template;

                    while let Some(arg) = args.next() {
                        match arg.as_str() {
                            "--where" => {
                                filter = format!(" WHERE {query}", query = args.next().unwrap());
                            }
                            "--field" => {
                                field = args.next().unwrap();
                            }
                            "--mark-as-read" => {
                                mark_as_read = true;
                            }
                            "--write-to-file" => {
                                let path = args.next().expect("--write-to-file");

                                write_to_file = Some(fs::File::open(path).expect("invalid file"));
                            }
                            "--template" => {
                                _template = args.next().expect("template");
                                template = Some(utilities::template::Template::new(&_template));
                            }
                            "--pick" | "--limit" => {
                                pick = args
                                    .next()
                                    .unwrap()
                                    .parse()
                                    .expect("--pick or --limit field must be usize");
                            }
                            flag => panic!("unknown {flag}"),
                        }
                    }

                    let query = if mark_as_read {
                        // requires *rowid tables* to perform
                        format!(
                            "UPDATE SET read=1 FROM {table}{filter} WHERE ROWID IN (SELECT ROWID FROM {table}{filter} ORDER BY random() LIMIT {pick}) RETURNING {field};"
                        )
                    } else {
                        format!(
                            "SELECT {field} FROM {table}{filter} ORDER BY random() LIMIT {pick};"
                        )
                    };

                    let mut out: utilities::output::StdoutOrFile = match write_to_file {
                        Some(file) => utilities::output::StdoutOrFile::file(file),
                        None => utilities::output::StdoutOrFile::stdout(),
                    };
                    let mut found = false;

                    connection
                        .iterate(query, |pairs| {
                            use std::io::Write;

                            if found {
                                writeln!(&mut out, "---").unwrap();
                            }
                            if let Some(ref template) = template {
                                let output = template.interpolate(|key| {
                                    pairs
                                        .iter()
                                        .find_map(|(column, value)| {
                                            (key == *column).then_some(value.unwrap_or("*null*"))
                                        })
                                        .unwrap_or("*no column*")
                                });
                                writeln!(&mut out, "{output}").unwrap();
                            } else {
                                let is_single = pairs.len() == 1;
                                if is_single {
                                    let value = pairs.iter().next().unwrap().1.unwrap();
                                    writeln!(&mut out, "{value}").unwrap();
                                } else {
                                    for (name, value) in pairs {
                                        writeln!(
                                            &mut out,
                                            "{name}={value}",
                                            value = value.unwrap_or("*null*")
                                        )
                                        .unwrap();
                                    }
                                }
                            }
                            found = true;
                            true
                        })
                        .unwrap();
                }
                // None | Some("--help") | Some(_arg) => {
                _ => {
                    println!("random");
                }
            }
        }
        Some("information") => {
            let item = args.next();
            match item.as_deref() {
                Some("tables") => {
                    let query = "SELECT sql FROM sqlite_master;";
                    let _ = connection.iterate(query, |pairs| {
                        let value = pairs.iter().next().unwrap().1.unwrap();
                        println!("{value}");
                        true
                    });
                }
                // None | Some("--help") | Some(_arg) => {
                _ => {
                    println!("tables");
                }
            }
        }
        Some("statistic") => {
            let statistic = args.next();
            let expression = args.next().unwrap();

            let mut filter = String::new();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--subset" => {
                        filter = format!(" WHERE {query}", query = args.next().unwrap());
                    }
                    arg => panic!("unknown {arg}"),
                }
            }
            match statistic.as_deref() {
                Some("percentage") => {
                    let query = format!(
                        "SELECT cast(SUM({expression}) AS FLOAT) / COUNT(*) FROM {table}{filter};"
                    );

                    connection
                        .iterate(query, |pairs| {
                            println!("{value}", value = pairs.iter().next().unwrap().1.unwrap());
                            true
                        })
                        .unwrap();
                }
                Some("count") => {
                    let query = format!("SELECT COUNT(*) FROM {table}{filter};");

                    connection
                        .iterate(query, |pairs| {
                            println!("{value}", value = pairs.iter().next().unwrap().1.unwrap());
                            true
                        })
                        .unwrap();
                }
                // None | Some("--help") => {
                _ => {
                    println!("count, percentage");
                }
            }
        }
        Some(arg) => {
            panic!("unknown flag {arg:?}")
        }
    }
}

fn file_name(name: &str) -> Option<&str> {
    path::Path::new(name).file_stem().and_then(|s| s.to_str())
}
