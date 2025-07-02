#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Item {
    pub total: Field,
    pub kind: Kind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field {
    pub name: String,
    pub size: usize,
    pub alignment: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FieldOrPadding {
    Field(Field),
    Padding(usize),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Variant {
    pub total: Field,
    pub fields: Vec<FieldOrPadding>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Kind {
    EnumItem {
        discriminant: usize,
        variants: Vec<Variant>,
    },
    StructItem {
        fields: Vec<FieldOrPadding>,
    },
}

pub fn item_from_input(out: &str, skip: impl Fn(&str, usize) -> bool) -> Option<Item> {
    let Some(out) = out.strip_prefix("print-type-size type: `") else {
        panic!("did not start with 'print-type-size type: `'");
    };

    let (name, out) = out.split_once("`: ").expect("find name");
    let (size, out) = out.split_once(", alignment: ").unwrap_or(("", out));
    let mut rest = out.lines();
    let alignment = rest.next().unwrap();

    let size = size
        .strip_suffix(" bytes")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let alignment = alignment
        .strip_suffix(" bytes")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if skip(name, size) {
        return None;
    }

    let total = Field {
        name: name.to_owned(),
        size,
        alignment,
    };

    let mut variant = None;
    let mut kind = if let Some(line) = rest.next() {
        dbg!(line);
        let line = line.strip_prefix("print-type-size     ").unwrap();
        if let Some(line) = line.strip_prefix("discriminant: ") {
            let bytes = line
                .strip_suffix(" bytes")
                .unwrap()
                .parse()
                .expect("invalid byte count");
            Kind::EnumItem {
                discriminant: bytes,
                variants: Vec::new(),
            }
        } else if let Some(line) = line.strip_prefix("variant `") {
            let (name, size) = line.split_once("`: ").unwrap();
            let size = size
                .strip_suffix(" bytes")
                .unwrap()
                .parse()
                .expect("invalid byte count");
            let alignment = 0;
            let total = Field {
                name: name.to_owned(),
                size,
                alignment,
            };
            variant = Some(Variant {
                total,
                fields: Vec::new(),
            });
            Kind::EnumItem {
                discriminant: 0,
                variants: Vec::new(),
            }
        } else {
            let Some(line) = line.strip_prefix("field `.") else {
                panic!("{line:?}")
            };
            let (name, size) = line.split_once("`: ").unwrap();
            let size = size
                .strip_suffix(" bytes")
                .unwrap()
                .parse()
                .expect("invalid byte count");
            let alignment = 0;
            let field = Field {
                name: name.to_owned(),
                size,
                alignment,
            };
            Kind::StructItem {
                fields: vec![FieldOrPadding::Field(field)],
            }
        }
    } else {
        Kind::StructItem {
            fields: Vec::default(),
        }
    };

    match kind {
        Kind::StructItem { ref mut fields, .. } => {
            for line in rest {
                let Some(line) = line.strip_prefix("print-type-size     ") else {
                    break;
                };
                let field = if let Some(line) = line.strip_prefix("field `.") {
                    let (name, size) = line.split_once("`: ").unwrap();
                    let size = parse_byte_literal(size);
                    let alignment = 0;
                    let field = Field {
                        name: name.to_owned(),
                        size,
                        alignment,
                    };
                    FieldOrPadding::Field(field)
                } else if let Some(size) = line
                    .strip_prefix("end padding: ")
                    .or_else(|| line.strip_prefix("padding: "))
                {
                    let size = parse_byte_literal(size);
                    FieldOrPadding::Padding(size)
                } else {
                    eprintln!("unknown {line:?}");
                    continue;
                };
                fields.push(field);
            }
        }
        Kind::EnumItem {
            ref mut variants, ..
        } => {
            let mut variant = variant;
            for line in rest {
                let Some(line) = line.strip_prefix("print-type-size     ") else {
                    break;
                };
                if let Some(line) = line.strip_prefix("variant `") {
                    if let Some(variant) = variant.take() {
                        variants.push(variant);
                    }
                    let (name, size) = line.split_once("`: ").unwrap();
                    let size = parse_byte_literal(size);
                    let alignment = 0;
                    let total = Field {
                        name: name.to_owned(),
                        size,
                        alignment,
                    };
                    variant = Some(Variant {
                        total,
                        fields: Vec::new(),
                    });
                } else if let Some(line) = line.strip_prefix("    field `.") {
                    let (name, size) = line.split_once("`: ").unwrap();
                    let (size, alignment) = size.split_once(", alignment: ").unwrap_or((size, ""));
                    let size = parse_byte_literal(size);
                    let alignment = if !alignment.is_empty() {
                        parse_byte_literal(alignment)
                    } else {
                        0
                    };
                    let field = Field {
                        name: name.to_owned(),
                        size,
                        alignment,
                    };
                    variant
                        .as_mut()
                        .unwrap()
                        .fields
                        .push(FieldOrPadding::Field(field));
                } else if let Some(line) = line.strip_prefix("    padding: ") {
                    let size = parse_byte_literal(line);
                    variant
                        .as_mut()
                        .unwrap()
                        .fields
                        .push(FieldOrPadding::Padding(size));
                } else {
                    eprintln!("enum todo {line:?}");
                    continue;
                }
            }
            if let Some(variant) = variant.take() {
                variants.push(variant);
            }
        }
    }

    Some(Item { total, kind })
}

fn parse_byte_literal(on: &str) -> usize {
    if let Some(size) = on.strip_suffix(" bytes") {
        parse_byte_count(size)
    } else {
        eprintln!("invalid byte name {on:?}");
        0
    }
}

fn parse_byte_count(on: &str) -> usize {
    if let Ok(size) = on.parse() {
        size
    } else {
        eprintln!("invalid byte count {on:?}");
        0
    }
}

#[cfg(test)]
mod tests {
    use super::{item_from_input, Field, Kind, FieldOrPadding};

    #[test]
    fn parse() {
        let out = &["print-type-size type: `Item<'_, '_>`: 48 bytes, alignment: 8 bytes
print-type-size     field `.total_source`: 16 bytes
print-type-size     field `.raw_node`: 32 bytes",

    "print-type-size type: `tree_sitter::Node<'_>`: 32 bytes, alignment: 8 bytes
print-type-size     field `.0`: 32 bytes
print-type-size     field `.1`: 0 bytes",

    "print-type-size type: `result::Result<(), std::fmt::Error>`: 1 bytes, alignment: 1 bytes
print-type-size     discriminant: 1 bytes
print-type-size     variant `Ok`: 0 bytes
print-type-size         field `.0`: 0 bytes
print-type-size     variant `Err`: 0 bytes
print-type-size         field `.0`: 0 bytes",

    "print-type-size type: `result::Result<std::fs::ReadDir, std::io::Error>`: 624 bytes, alignment: 8 bytes
print-type-size     variant `Ok`: 624 bytes
print-type-size         field `.0`: 624 bytes
print-type-size     variant `Err`: 16 bytes
print-type-size         padding: 8 bytes
print-type-size         field `.0`: 8 bytes, alignment: 8 bytes",

    "print-type-size type: `sys::fs::windows::ReadDir`: 624 bytes, alignment: 8 bytes
print-type-size     field `.handle`: 16 bytes
print-type-size     field `.root`: 8 bytes
print-type-size     field `.first`: 596 bytes
print-type-size     end padding: 4 bytes",

    "print-type-size type: `ffi::c_void`: 1 bytes, alignment: 1 bytes
print-type-size     discriminant: 1 bytes
print-type-size     variant `__variant1`: 0 bytes
print-type-size     variant `__variant2`: 0 bytes"];

        let items: Vec<_> = out
            .iter()
            .map(|part| item_from_input(part, |_, _| false).unwrap())
            .collect();

        assert_eq!(&items[0].total.name, "Item<'_, '_>");
        assert_eq!(items[0].total.size, 48);

        assert_eq!(items[4].total.name, "sys::fs::windows::ReadDir");
        assert_eq!(items[4].total.size, 624);
        assert_eq!(
            items[4].kind,
            Kind::StructItem {
                fields: vec![
                    FieldOrPadding::Field(Field {
                        name: "handle".to_owned(),
                        size: 16,
                        alignment: 0
                    }),
                    FieldOrPadding::Field(Field {
                        name: "root".to_owned(),
                        size: 8,
                        alignment: 0
                    }),
                    FieldOrPadding::Field(Field {
                        name: "first".to_owned(),
                        size: 596,
                        alignment: 0
                    }),
                    FieldOrPadding::Padding(4),
                ]
            }
        );
    }
}
