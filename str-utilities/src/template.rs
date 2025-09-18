use std::borrow::Cow;

pub struct Template<'t> {
    pub items: Vec<&'t str>,
}

const EMPTY_SLOT: &str = "";

impl<'t> Template<'t> {
    pub fn new(on: &'t str, interpolate_character: char) -> Self {
        let mut items = Vec::new();
        let mut start = 0;
        for (idx, _matched) in on.match_indices(interpolate_character) {
            let escaped = on[..idx].ends_with(interpolate_character);
            if escaped {
                items.push(&on[start..=idx]);
                items.push(EMPTY_SLOT);
                start = idx + 1;
            } else {
                items.push(&on[start..idx]);
                let rest = &on[idx..][1..];
                let name_end = rest
                    .find(|c: char| !(c.is_alphanumeric() || matches!(c, '_')))
                    .unwrap_or(rest.len());

                items.push(&rest[..name_end]);
                start = idx + 1 + name_end;
            }
        }
        items.push(&on[start..]);
        Self { items }
    }

    pub fn interpolate<'a>(&'a self, mut cb: impl FnMut(&'t str) -> Cow<'a, str>) -> String {
        let mut buf = String::new();
        for (idx, item) in self.items.iter().enumerate() {
            if idx % 2 == 0 {
                buf.push_str(item);
            } else if item != &EMPTY_SLOT {
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
        let template = Template::new("Hello $name!", '$');
        let values = std::collections::HashMap::from([("name", "Ben")]);
        assert_eq!(
            &template.interpolate(|key| values[key].into()),
            "Hello Ben!"
        );
    }

    #[test]
    fn escape() {
        let template = Template::new("Hello $name! Here is $$20!", '$');
        let values = std::collections::HashMap::from([("name", "Ben")]);
        assert_eq!(
            &template.interpolate(|key| values[key].into()),
            "Hello Ben! Here is $20!"
        );
    }
}
