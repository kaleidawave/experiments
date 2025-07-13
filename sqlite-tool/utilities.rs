pub static _KEYWORDS: &[&[u8]] = &[
    b"abort",
    b"action",
    b"add",
    b"after",
    b"all",
    b"alter",
    b"always",
    b"analyze",
    b"and",
    b"as",
    b"asc",
    b"attach",
    b"autoincrement",
    b"before",
    b"begin",
    b"between",
    b"by",
    b"cascade",
    b"case",
    b"cast",
    b"check",
    b"collate",
    b"column",
    b"commit",
    b"conflict",
    b"constraint",
    b"create",
    b"cross",
    b"current",
    b"current_date",
    b"current_time",
    b"current_timestamp",
    b"database",
    b"default",
    b"deferrable",
    b"deferred",
    b"delete",
    b"desc",
    b"detach",
    b"distinct",
    b"do",
    b"drop",
    b"each",
    b"else",
    b"end",
    b"escape",
    b"except",
    b"exclude",
    b"exclusive",
    b"exists",
    b"explain",
    b"fail",
    b"filter",
    b"first",
    b"following",
    b"for",
    b"foreign",
    b"from",
    b"full",
    b"generated",
    b"glob",
    b"group",
    b"groups",
    b"having",
    b"if",
    b"ignore",
    b"immediate",
    b"in",
    b"index",
    b"indexed",
    b"initially",
    b"inner",
    b"insert",
    b"instead",
    b"intersect",
    b"into",
    b"is",
    b"isnull",
    b"join",
    b"key",
    b"last",
    b"left",
    b"like",
    b"limit",
    b"match",
    b"materialized",
    b"natural",
    b"no",
    b"not",
    b"nothing",
    b"notnull",
    b"null",
    b"nulls",
    b"of",
    b"offset",
    b"on",
    b"or",
    b"order",
    b"others",
    b"outer",
    b"over",
    b"partition",
    b"plan",
    b"pragma",
    b"preceding",
    b"primary",
    b"query",
    b"raise",
    b"range",
    b"recursive",
    b"references",
    b"regexp",
    b"reindex",
    b"release",
    b"rename",
    b"replace",
    b"restrict",
    b"returning",
    b"right",
    b"rollback",
    b"row",
    b"rows",
    b"savepoint",
    b"select",
    b"set",
    b"table",
    b"temp",
    b"temporary",
    b"then",
    b"ties",
    b"to",
    b"transaction",
    b"trigger",
    b"unbounded",
    b"union",
    b"unique",
    b"update",
    b"using",
    b"vacuum",
    b"values",
    b"view",
    b"virtual",
    b"when",
    b"where",
    b"window",
    b"with",
    b"without",
];

pub(crate) fn strip_around<'a>(on: &'a str, around: &str) -> Option<&'a str> {
    on.strip_prefix(around)
        .and_then(|on| on.strip_suffix(around))
}

pub(crate) fn seperate_off_parenthesised(on: &str) -> (&str, Option<&str>) {
    let on = on.trim();
    if let Some((on, item)) = on.strip_suffix(')').and_then(|on| on.split_once('(')) {
        (on.trim_end(), Some(item))
    } else {
        (on, None)
    }
}

pub mod csv {
    use std::borrow::Cow;

    pub struct CSVColumns<'a> {
        on: &'a str,
        last: usize,
    }

    impl<'a> CSVColumns<'a> {
        pub fn new(on: &'a str) -> Self {
            Self { on, last: 0 }
        }
    }

    impl<'a> Iterator for CSVColumns<'a> {
        type Item = Cow<'a, str>;

        fn next(&mut self) -> Option<Self::Item> {
            let start = self.last;
            let rest = self.on.get(start..)?;

            if let Some(rest) = rest.strip_prefix('"') {
                let next = rest.find('"').expect("no end") + 1;
                self.last += next + 1;
                // TODO process escapes etc
                // for (idx, matched) in rest.match_indices('\\') { match matched { "\"" => { was_string = true; in_string = !in_string; } "," => { if !in_string { self.last += idx + 1; return Some(Cow::Borrowed()); } } item => unreachable!("{item:?}") } }
                Some(Cow::Borrowed(&rest[..next]))
            } else {
                let next = rest.find(',').unwrap_or(rest.len());
                self.last += next + 1;
                Some(Cow::Borrowed(&rest[..next]))
            }
        }
    }
}

pub mod repeate {
    use std::fmt;

    pub struct Repeat<T, U> {
        item: T,
        delimeter: U,
        count: usize,
    }

    impl<T, U> Repeat<T, U>
    where
        T: fmt::Display,
        U: fmt::Display,
    {
        pub fn new(item: T, delimeter: U, count: usize) -> Self {
            Self {
                item,
                delimeter,
                count,
            }
        }
    }

    impl<T, U> fmt::Display for Repeat<T, U>
    where
        T: fmt::Display,
        U: fmt::Display,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self {
                item,
                delimeter,
                count,
            } = self;
            for i in 0..*count {
                if i > 0 {
                    write!(f, "{delimeter}")?;
                }
                write!(f, "{item}")?;
            }
            Ok(())
        }
    }
}

pub mod escape {}

pub mod template {
    pub struct Template<'t> {
        pub items: Vec<&'t str>,
    }

    impl<'t> Template<'t> {
        pub fn new(on: &'t str) -> Self {
            let mut items = Vec::new();
            let mut start = 0;
            for (idx, _matched) in on.match_indices('$') {
                // let escaped = on[..idx].ends_with("\\");
                // if escaped {
                items.push(&on[start..idx]);

                let rest = &on[idx..][1..];
                let name_end = rest
                    .find(|c: char| !(c.is_alphanumeric() || matches!(c, '_')))
                    .unwrap_or(rest.len());
                items.push(&rest[..name_end]);

                start = idx + 1 + name_end;
                // }
            }
            items.push(&on[start..]);
            Self { items }
        }

        pub fn interpolate<'a>(&'a self, mut cb: impl FnMut(&'t str) -> &'a str) -> String {
            let mut buf = String::new();
            for (idx, item) in self.items.iter().enumerate() {
                if idx % 2 == 0 {
                    buf.push_str(item);
                } else {
                    buf.push_str(cb(item));
                }
            }
            buf
        }
    }
}

pub mod output {
    pub enum StdoutOrFile {
        File(std::fs::File),
        Stdout(std::io::Stdout),
    }

    impl StdoutOrFile {
        pub fn stdout() -> Self {
            Self::Stdout(std::io::stdout())
        }

        // pub fn file_from_path(path: &std::path::Path) -> Self {
        //     Self::File(std::fs::File::open(path).unwrap())
        // }

        pub fn file(file: std::fs::File) -> Self {
            Self::File(file)
        }
    }

    impl std::io::Write for StdoutOrFile {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            match self {
                Self::File(file) => file.write(buf),
                Self::Stdout(out) => out.write(buf),
            }
        }

        fn flush(&mut self) -> std::io::Result<()> {
            match self {
                Self::File(file) => file.flush(),
                Self::Stdout(out) => out.flush(),
            }
        }
    }
}
