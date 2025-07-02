use rust_type_sizes::item_from_input;

#[test]
fn main() {
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

    assert_eq!(items[4].total.size, 624);
}
