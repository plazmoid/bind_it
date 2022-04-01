#![feature(type_alias_impl_trait)]

use std::future::{ready, Future};
use std::iter::{DoubleEndedIterator, IntoIterator};

#[macro_use]
extern crate bind_it;

#[test]
fn let_simple() {
    bind_it! { let rng: impl Iterator<Item = u32> = 5..9; }
    for n in rng {
        println!("{n}")
    }
}

#[test]
fn let_path() {
    bind_it!( let num: impl std::string::ToString = 5; );
    // assert_eq!(num + 5, 10); // can't add an int to impl ToString
    assert_eq!(num.to_string(), String::from("5"));
}

bind_it! {
    static STR_REVERSER: impl Fn(impl IntoIterator<IntoIter = impl DoubleEndedIterator<Item = impl Into<String>>>) -> String =
        |s| s.into_iter().map(|s1| s1.into()).rev().collect();
}

#[test]
fn static_fn_trait() {
    assert_eq!(&STR_REVERSER("xol".chars()), "lox");
}

#[tokio::test]
async fn async_nested() {
    bind_it! {
        let fut: impl Future<Output = impl Future<Output = (impl Into<usize>, impl std::fmt::Display)> + 'static> + 'static = async {
            ready((5u8, true))
        };
    }
    let (into_usize, displayable) = fut.await.await;

    // we cannot check equality of concrete and opaque types, so these asserts fail
    // assert_eq!(into_usize, 5u8);
    // assert_eq!(displayable, true);

    assert_eq!(into_usize.into(), 5usize);
    assert_eq!(displayable.to_string(), "true");
}

#[test]
fn inside_other_types() {
    bind_it! { let _: Option<&impl std::string::ToString> = Some(&false); }
    bind_it! { let _: (impl std::string::ToString, _) = (false, true); }
    bind_it! { let _: *mut impl Into<()> = &mut () as *mut _; }
}
