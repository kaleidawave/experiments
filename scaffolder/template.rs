use std::borrow::Cow;

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

    pub fn interpolate<'a>(&'a self, mut cb: impl FnMut(&'t str) -> Cow<'a, str>) -> String {
        let mut buf = String::new();
        for (idx, item) in self.items.iter().enumerate() {
            if idx % 2 == 0 {
                buf.push_str(item);
            } else {
                buf.push_str(&cb(item));
            }
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let template = Template::new("Hello $name!");
        let values = std::collections::HashMap::from([("name", "Ben")]);
        assert_eq!(&template.interpolate(|key| values[key]), "Hello Ben!");
    }
}
