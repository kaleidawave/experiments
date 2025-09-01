A simple macro for *generating* JSON

```rust
let object = json! {
    x: 78u32,
    y: 72.4f64,
    z: "thing"
};
assert_eq!(object, r#"{"x":78,"y":72.4,"z":"thing"}"#);
```

