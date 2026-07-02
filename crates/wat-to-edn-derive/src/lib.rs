//! PROBE STUB — arc 296 stone A disconfirming probe.
//!
//! This proves the crate-graph shape before the real surgery: a `proc-macro`
//! crate that depends on NOTHING of wat's (only syn/quote/proc-macro2), is
//! re-exported by wat-edn under the `derive` feature, and is usable as
//! `#[derive(wat_edn::ProbeToEdn)]` from a dependent (wat-reader) — with a
//! `#[to_edn(...)]` helper attribute — and no dependency cycle.
//!
//! Once the probe is green, the real `ToEdn` derive (moved out of `wat-macros`,
//! from `to_edn_derive.rs`) fills this crate and `ProbeToEdn` is replaced by
//! the real `ToEdn`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ProbeToEdn, attributes(to_edn))]
pub fn derive_probe_to_edn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_g, ty_g, where_g) = input.generics.split_for_impl();
    quote! {
        impl #impl_g ::wat_edn::ProbeToEdn for #name #ty_g #where_g {
            fn probe_to_edn(&self) -> ::wat_edn::OwnedValue {
                ::wat_edn::OwnedValue::Nil
            }
        }
    }
    .into()
}
