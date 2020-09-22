qualify-derive
===

Simple utility for wrapping derive macros that do not qualify paths properly.

## When is this needed?
For example, [specs v0.16.1](https://docs.rs/specs/0.16.1/specs) has a derive macro `Component`.
If the `#[storage]` is not passed with the macro,
it generates a line `type Storage = DenseVecStorage;`,
which causes compile errors when `DenseVecStorage` is not already imported.

It is annoying to manually add an import for a derive macro all the time.

## How to use
Create a *new* crate with this Cargo.toml (proc macros won't work in the current crate):

```toml
[package]
name = # the usual [package] stuff

[lib]
proc-macro = true

[dependencies]
qualify-derive = "0.1.0"
```

Then create `src/lib.rs` with the following cntents:

```rust

qualify_derive::declare! {
    your_attribute_name derives ::full_path_to::TargetDeriveMacro;
    use the_paths::you_want_to::import_automatically;
    use import_groups_are::not_supported;
		// use foo::{bar, qux}; // this does not work
    attr this_line_is_optional
}
```

This declares a proc-macro-attribute called `your_attribute_name`.
In downstream crates, you can use `#[your_attribute_name]`
in place of `#[derive(::full_path_to::TargetDeriveMacro)]`.

`attr this_line_is_optional` will allow downstream crates to use `#[your_attribute_name(some content here)]`,
which is expanded into `#[derive(::full_path_to::TargetDeriveMacro)] #[this_line_is_optional(some content here)]`.

## Example
See `test-macro` and `test-lib`.

## Limitations
- Types that use attributes generated by this crate are known to have issues with `cargo fix` when they are unused.
	`cargo fix` **removes the attribute** if the type is unused.
- The generated attributes would wrap the user type inside an anonymous module.
	`super` would refer to the module that the type itself is declared in.
	However, all symbols in the `super` scope are automatically imported down to the scope of the inner module,
	so it is not necessary to use `super::` inside the type;
	but any use of `super` would be incorrect.
	Use `super::super` or absolute paths from `crate::` instead.