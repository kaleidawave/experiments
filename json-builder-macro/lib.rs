pub struct JSON<'a>(pub &'a dyn ToJSON);

#[macro_export]
macro_rules! json {
    {$( $key:ident : $val:expr ),* $(,)?} => {{
		let mut buf = String::new();
		let mut builder = Builder::new(&mut buf);
		let pairs: &[(&str, JSON)] = &[$(
			(stringify!($key), JSON(&$val)),
		)*];
		for (key, value) in pairs {
			builder.add(key, value);
		}
		builder.end();
		buf
	}};
}

pub struct Builder<'a> {
    started: bool,
    buf: &'a mut String,
}

impl<'a> Builder<'a> {
    pub fn new(buf: &'a mut String) -> Self {
        buf.push('{');
        Self {
            started: false,
            buf,
        }
    }

    pub fn add(&mut self, key: &str, value: impl ToJSON) {
        if self.started {
            self.buf.push(',');
        }
        self.buf.push('"');
        // TODO escape
        self.buf.push_str(key);
        self.buf.push_str("\":");
        ToJSON::append(&value, self.buf);
        self.started = true;
    }

    pub fn end(self) {
        self.buf.push('}');
    }
}

// TODO depth
pub trait ToJSON {
    fn append(&self, buf: &mut String);
}

impl<'a> ToJSON for &'a str {
    fn append(&self, buf: &mut String) {
        buf.push('"');
        buf.push_str(&escape_json_string(self));
        buf.push('"')
    }
}

impl ToJSON for String {
    fn append(&self, buf: &mut String) {
        ToJSON::append(&self.as_str(), buf)
    }
}

impl<T: ToJSON> ToJSON for &[T] {
    fn append(&self, buf: &mut String) {
        buf.push('[');
        for (idx, item) in self.iter().enumerate() {
            if idx > 0 {
                buf.push(',')
            }
            ToJSON::append(item, buf)
        }
        buf.push(']');
    }
}

impl<T: ToJSON> ToJSON for Vec<T> {
    fn append(&self, buf: &mut String) {
        ToJSON::append(&self.as_slice(), buf)
    }
}

impl<K: AsRef<str>, V: ToJSON> ToJSON for std::collections::HashMap<K, V> {
    fn append(&self, buf: &mut String) {
        buf.push('{');
        for (idx, (key, value)) in self.iter().enumerate() {
            if idx > 0 {
                buf.push(',')
            }
            buf.push('"');
            buf.push_str(&escape_json_string(key.as_ref()));
            buf.push_str("\":");
            ToJSON::append(value, buf);
        }
        buf.push('}');
    }
}

impl ToJSON for bool {
    fn append(&self, buf: &mut String) {
        buf.push_str(match self {
            true => "true",
            false => "false",
        })
    }
}

macro_rules! create_json_from_to_string_implementation {
    ($($T:ty),*) => {
        $(
            impl ToJSON for $T {
                fn append(&self, buf: &mut String) {
                    buf.push_str(&self.to_string())
                }
            }
        )*
    }
}

// For all number types
create_json_from_to_string_implementation![u8, u16, u32, u64, i8, i16, i32, i64, f32, f64];

impl ToJSON for JSON<'_> {
    fn append(&self, buf: &mut String) {
        self.0.append(buf)
    }
}

impl ToJSON for &'_ JSON<'_> {
    fn append(&self, buf: &mut String) {
        self.0.append(buf)
    }
}

pub fn escape_json_string(on: &str) -> std::borrow::Cow<'_, str> {
    let mut result = std::borrow::Cow::Borrowed("");
    let mut start = 0;
    for (index, matched) in on.match_indices(['\"', '\n', '\t', '\\']) {
        result += &on[start..index];
        result += "\\";
        // I think this is correct?
        result += match matched {
            "\"" => "\"",
            "\\" => "\\",
            "\n" => "n",
            "\t" => "t",
            _ => unreachable!()
        };
        start = index + 1;
    }
    result += &on[start..];
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_object() {
        let object = json! {
            x: 78u32,
            y: 72.4f64,
            z: "thing"
        };
        assert_eq!(object, r#"{"x":78,"y":72.4,"z":"thing"}"#);
    }

    #[test]
    fn escaping() {
        let object = json! {
            x: 78u32,
            y: 72.4f64,
            z: "thing\nover two lines"
        };
        assert_eq!(
            object,
            "{\"x\":78,\"y\":72.4,\"z\":\"thing\\nover two lines\"}"
        );
    }

    #[test]
    fn vec() {
        let z = vec!["thing", "here"];
        let object = json! {
            x: 78u32,
            y: 72.4f64,
            z: z
        };
        assert_eq!(object, r#"{"x":78,"y":72.4,"z":["thing","here"]}"#);
    }

    #[test]
    fn hash_map() {
        let values = std::collections::HashMap::from_iter([("k1", "v1"), ("k2", "v2")]);
        let object = json! {
            kind: "map",
            values: values
        };
        // because HashMap order is randomised, we test either cases
        let possibles = [
            r#"{"kind":"map","values":{"k1":"v1","k2":"v2"}}"#,
            r#"{"kind":"map","values":{"k2":"v2","k1":"v1"}}"#,
        ];
        assert!(possibles.contains(&object.as_str()));
    }
}
