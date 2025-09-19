#![doc = include_str!("README.md")]

/// Wraps a JSON structure
pub struct JSON<'a>(pub &'a dyn ToJSON);

/// Takes input similar to JavaScript object-notation and generates string of JSON.
///
/// ```rust
/// let object = json_builder_macro::json! {
///     x: 78u32,
///     y: 72.4f64,
///     z: "thing"
/// };
/// assert_eq!(object, r#"{"x":78,"y":72.4,"z":"thing"}"#);
/// ```
#[macro_export]
macro_rules! json {
    {$( $key:ident : $val:expr ),* $(,)?} => {{
		let mut buf = String::new();
		let mut builder = $crate::Builder::new(&mut buf);
		let pairs: &[(&str, $crate::JSON)] = &[$(
			(stringify!($key), $crate::JSON(&$val)),
		)*];
		for (key, value) in pairs {
			builder.add(key, value);
		}
		builder.end();
		buf
	}};
}

/// For *building up a JSON* object
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
        ToJSON::append_as_json_string(&value, self.buf);
        self.started = true;
    }

    pub fn end(self) {
        self.buf.push('}');
    }
}

/// Represents a JSON object that can be serialized to JSON
///
/// TODO depth for pretty
pub trait ToJSON {
    fn as_json_string(&self) -> String {
        let mut buf = String::new();
        ToJSON::append_as_json_string(self, &mut buf);
        buf
    }

    fn append_as_json_string(&self, buf: &mut String);
}

impl ToJSON for &str {
    fn append_as_json_string(&self, buf: &mut String) {
        buf.push('"');
        buf.push_str(&escape_json_string(self));
        buf.push('"')
    }
}

impl ToJSON for String {
    fn append_as_json_string(&self, buf: &mut String) {
        ToJSON::append_as_json_string(&self.as_str(), buf)
    }
}

impl<T: ToJSON> ToJSON for &[T] {
    fn append_as_json_string(&self, buf: &mut String) {
        buf.push('[');
        for (idx, item) in self.iter().enumerate() {
            if idx > 0 {
                buf.push(',')
            }
            ToJSON::append_as_json_string(item, buf)
        }
        buf.push(']');
    }
}

impl<T: ToJSON> ToJSON for Vec<T> {
    fn append_as_json_string(&self, buf: &mut String) {
        ToJSON::append_as_json_string(&self.as_slice(), buf)
    }
}

impl<T1: ToJSON, T2: ToJSON> ToJSON for (T1, T2) {
    fn append_as_json_string(&self, buf: &mut String) {
        buf.push('[');
        ToJSON::append_as_json_string(&self.0, buf);
        buf.push(',');
        ToJSON::append_as_json_string(&self.1, buf);
        buf.push(']');
    }
}

impl<T1: ToJSON, T2: ToJSON, T3: ToJSON> ToJSON for (T1, T2, T3) {
    fn append_as_json_string(&self, buf: &mut String) {
        buf.push('[');
        ToJSON::append_as_json_string(&self.0, buf);
        buf.push(',');
        ToJSON::append_as_json_string(&self.1, buf);
        buf.push(',');
        ToJSON::append_as_json_string(&self.2, buf);
        buf.push(']');
    }
}

impl<K: AsRef<str>, V: ToJSON> ToJSON for std::collections::HashMap<K, V> {
    fn append_as_json_string(&self, buf: &mut String) {
        buf.push('{');
        for (idx, (key, value)) in self.iter().enumerate() {
            if idx > 0 {
                buf.push(',')
            }
            buf.push('"');
            buf.push_str(&escape_json_string(key.as_ref()));
            buf.push_str("\":");
            ToJSON::append_as_json_string(value, buf);
        }
        buf.push('}');
    }
}

impl ToJSON for bool {
    fn append_as_json_string(&self, buf: &mut String) {
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
                fn append_as_json_string(&self, buf: &mut String) {
                    buf.push_str(&self.to_string())
                }
            }
        )*
    }
}

// For all number types
create_json_from_to_string_implementation![u8, u16, u32, u64, i8, i16, i32, i64, f32, f64];

impl ToJSON for JSON<'_> {
    fn append_as_json_string(&self, buf: &mut String) {
        self.0.append_as_json_string(buf)
    }
}

impl ToJSON for &'_ JSON<'_> {
    fn append_as_json_string(&self, buf: &mut String) {
        self.0.append_as_json_string(buf)
    }
}

/// Escapes string content to be valid JSON
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
            _ => unreachable!(),
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
