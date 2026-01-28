Originally from <https://gist.github.com/kaleidawave/6d544630c3288fd072630929c922752e>

This requires `#![feature(try_trait_v2)]`

Test with `cargo +nightly t`

This works

```rust
# use early_return_struct::Early;
fn get_first() -> Early<u8> {
	let () = Early::None?;
	let () = Early::Some(5)?;
	Early::Some(2)
}

assert_eq!(get_first(), Early::Some(5));
```

But this does not

```compile_fail
# use early_return_struct::Early;
fn get_first() -> Result<Early<u8>, ()> {
	let () = Early::Some(5)?;
	if false {
		return Err(());
	}
	Ok(Early::default())
}

assert_eq!(get_first(), Ok(Early::Some(5)));
```

---

> TODO does the [`yeet`](https://github.com/rust-lang/rust/issues/96374) proposal have any use here?