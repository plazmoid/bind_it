# Description
A macro that allows using impl trait types in bindings.
Currently supported statements: `let`, `const` (with some limitations mentioned below), `static`.
This crate will be replaced by [impl_trait_in_bindings](https://github.com/rust-lang/rust/issues/63065) feature in future,
but there is still a long way to stabilize it.

# Example
```rust,nightly
#![feature(type_alias_impl_trait)]

#[macro_use]
extern crate bind_it;

fn main() {
    bind_it!( let x: impl std::fmt::Display = true; );

    // fails, even x variable is initialized with a boolean, its type is hidden behind `Display` trait,
    // and the only thing that we can do - display x
    // assert!(x);

    // works
    println!("{x}")
}
```

# How it works?
[Ez!](https://rust-lang.github.io/impl-trait-initiative/explainer/lbit.html)

# Minimal compiler version
`rustc 1.61.0-nightly (c5cf08d37 2022-03-30)` with `#![feature(type_alias_impl_trait)]` enabled


# Limitations
* Currently only one item per macro supported
* Associated consts are not yet supported
* Only one type per impl Trait allowed. [tl;dr](https://stackoverflow.com/questions/52001592/why-can-impl-trait-not-be-used-to-return-multiple-conditional-types),
you can't write
```rust,nightly,no_run
bind_it! {
    let _: impl std::fmt::Display = if rand::random() > 0.5 {
            "qwe"
        } else {
            5u8
        };
};
```
Despite of that fact that both `&str` and `u8` implement `Display` trait, we need to determine ONE concrete type in the compile time.