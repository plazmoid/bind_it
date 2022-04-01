/*!
# Description
A macro that allows using impl trait types in bindings.
Currently supported statements: `let`, `const` (with some limitations mentioned below), `static`.
This crate will be replaced by [impl_trait_in_bindings](https://github.com/rust-lang/rust/issues/63065) feature in future,
but there is still a long way to stabilize it.

## Example
```rust,nightly
#![feature(type_alias_impl_trait)]
let x: std::fmt::Display = true;

// fails, even x variable is initialized with a boolean, its type is hidden behind `Display` trait,
// and the only thing that we can do - display x
assert!(x);

// works
println!("{x}")
```

## How it works?
[Ez!](https://rust-lang.github.io/impl-trait-initiative/explainer/lbit.html)

# Minimal compiler version
`rustc 1.61.0-nightly (c5cf08d37 2022-03-30)`


# Limitations
Yes, unfortunately:
* Associated consts are not yet supported
* Only one type per impl Trait allowed. [tl;dr](https://github.com/rust-lang/rfcs/blob/master/text/2071-impl-trait-existential-types.md#guide-existential-types),
you can't write
```rust,nightly
bind_it! {
    let _: impl std::fmt::Display = if rand::random() > 0.5 {
            "qwe"
        } else {
            5u8
        };
};
```
Despite of that fact that both `&str` and `u8` implement `Display` trait, we need to determine ONE concrete type in the compile time.
 */

#![feature(type_alias_impl_trait)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::fmt::Debug;
use syn::{
    parse_macro_input, parse_quote, GenericArgument, Item, Pat, PathArguments, ReturnType, Stmt,
    Type, TypeParamBound,
};
use uuid::Uuid;

#[proc_macro]
pub fn bind_it(item: TokenStream) -> TokenStream {
    let mut input: Stmt = parse_macro_input!(item);
    let extracted_taits = match &mut input {
        Stmt::Local(ref mut local) => {
            if let Pat::Type(ref mut pt) = local.pat {
                extract_and_replace_ty(&mut *pt.ty)
            } else {
                todo!();
            }
        }
        Stmt::Item(itm) => match itm {
            Item::Static(s) => extract_and_replace_ty(&mut *s.ty),
            Item::Const(c) => extract_and_replace_ty(&mut *c.ty),
            _ => panic!("only let, static and const statements are supported"),
        },
        _ => unimplemented!(),
    };
    let mut taits = TokenStream2::new();
    taits.extend(extracted_taits);
    quote! {
        #taits
        #input
    }
    .into()
}

fn extract_and_replace_ty(ty: &mut Type) -> Vec<TokenStream2> {
    let mut extracted = vec![];
    if let Type::ImplTrait(it) = ty {
        for trait_bounds in it.bounds.iter_mut() {
            if let TypeParamBound::Trait(ref mut tb) = trait_bounds {
                let tb_generics = &mut tb
                    .path
                    .segments
                    .last_mut()
                    .expect("how path if empty wtf bug")
                    .arguments;
                match tb_generics {
                    PathArguments::None => (),
                    PathArguments::AngleBracketed(ref mut ab) => {
                        for generic in ab.args.iter_mut() {
                            let new_taits = match generic {
                                GenericArgument::Type(t) => extract_and_replace_ty(t),
                                GenericArgument::Binding(b) => extract_and_replace_ty(&mut b.ty),
                                GenericArgument::Constraint(_) => unimplemented!(),
                                GenericArgument::Const(_) => unimplemented!(),
                                GenericArgument::Lifetime(_) => vec![],
                            };
                            extracted.extend(new_taits);
                        }
                    }
                    PathArguments::Parenthesized(ref mut ps) => {
                        for fn_arg in ps.inputs.iter_mut() {
                            extracted.extend(extract_and_replace_ty(fn_arg))
                        }
                        match ps.output {
                            ReturnType::Default => (),
                            ReturnType::Type(_, ref mut ty) => {
                                extracted.extend(extract_and_replace_ty(&mut *ty))
                            }
                        }
                    }
                }
            }
        }
        let rnd = Uuid::new_v4().to_simple().to_string();
        let tait_name = format_ident!("Bind_TAIT_{}", &rnd[..16]);
        let tait = quote! {
            type #tait_name = #it;
        };
        *ty = parse_quote! {
            #tait_name
        };
        extracted.push(tait);
    }
    extracted
}

fn _p(s: impl Debug) {
    panic!("{:#?}", s);
}
