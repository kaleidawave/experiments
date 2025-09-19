A simple macro for *generating* JSON

```rust
let object = json_builder_macro::json! {
    x: 78u32,
    y: 72.4f64,
    z: "thing"
};
assert_eq!(object, r#"{"x":78,"y":72.4,"z":"thing"}"#);
```

Also contains traits

```rust
let map = std::collections::HashMap::from_iter([("k1", "v1"), ("k2", "v2")]);
let out = &json_builder_macro::ToJSON::as_json_string(&map);
let valid = out == r#"{"k1":"v1","k2":"v2"}"# || out == r#"{"k2":"v2","k1":"v1"}"#;
assert!(valid);
```

and string JSON escaping

```rust
assert_eq!(&json_builder_macro::escape_json_string(r#"Hello "World""#), r#"Hello \"World\""#)
```