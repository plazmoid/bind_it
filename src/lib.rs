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
