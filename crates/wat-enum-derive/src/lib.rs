//! `#[derive(WatEnum)]` — the variant list stops having a hand-written form.
//!
//! ## The defect this exists to remove
//!
//! wat carries ten closed-domain unit enums (`Category`, `Purity`, `Determinism`,
//! `Kind`, `DefinedIn`, `Layer`, and the `Runtime*` mirrors). Each hand-wrote the
//! same three things beside the enum: a `variants()` string list, an `as_str()`
//! match, and a `FromStr` match. Fifteen hand-written functions, none of them
//! policed by the compiler.
//!
//! On 2026-08-15 that class fired three times in fifteen minutes while adding
//! variants, and — worse — the drift gate written that morning slept through it:
//! it compared `CATEGORY_LEGAL_VALUES` against `Category::variants()`, and
//! `variants()` was **itself a hand-written list**, so both went stale identically
//! and agreed with each other while the enum grew past both.
//!
//! **A gate comparing two hand-lists is a hand-list.** The rung above a gate is a
//! shape the mistake cannot be written down in: derive the list FROM the enum, and
//! there is nothing to forget because there is no list to forget it from.
//!
//! ## Why a separate crate
//!
//! `wat-macros` already depends on `wat-doc`, so a derive living there could not be
//! used BY `wat-doc` — a cycle. This is the `wat-to-edn-derive` pattern: a leaf
//! proc-macro crate depending on nothing of wat's, usable from both.
//!
//! ## What it generates
//!
//! ```ignore
//! #[derive(WatEnum)]
//! #[wat_enum(type_path = ":wat::runtime::Kind")]   // optional
//! pub enum Kind { Macro, Fn, Intrinsic, SpecialForm }
//! ```
//!
//! - `fn variants() -> &'static [&'static str]` — every variant, in declaration order
//! - `fn as_str(&self) -> &'static str`
//! - `impl FromStr` with `type Err = ()`
//! - `const WAT_TYPE_PATH: &'static str` — only when the attribute is given; it is
//!   what lets a test compare the Rust enum against its `defenum` mirror in
//!   `wat/runtime-meta.wat` instead of a comment claiming they match.
//!
//! Visibility is inherited from the enum, so a `pub(crate)` enum gets `pub(crate)`
//! methods.
//!
//! ⚠ **It deliberately does NOT generate an `all()` returning `[Self::A, Self::B]`** —
//! an array of every variant is a hand-list's shape again, and nothing needs one.
//!
//! ⚠ **The derive DOES make every variant non-dead**, and the first draft of this
//! comment claimed the opposite. `as_str`'s match arms only READ a variant, which
//! does not satisfy `dead_code` — but the generated `FromStr` **constructs** them
//! (`"Macro" => Ok(Self::Macro)`). Applying the derive to `Kind`/`DefinedIn`/`Layer`
//! therefore turned four `#[expect(dead_code)]` annotations unfulfilled, and they
//! were removed as no-longer-true. Reason about BOTH generated impls before
//! assuming a variant stays dead.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(WatEnum, attributes(wat_enum))]
pub fn derive_wat_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let vis = &input.vis;

    let Data::Enum(data) = &input.data else {
        return syn::Error::new_spanned(&input.ident, "WatEnum can only derive on an enum")
            .to_compile_error()
            .into();
    };

    // Unit variants only — a variant with data has no single `&'static str`
    // spelling, and inventing one would be exactly the kind of lie this derive
    // exists to prevent. `Arity { Exact(usize), Variadic }` is the real case.
    let mut idents = Vec::new();
    for v in &data.variants {
        if !matches!(v.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                &v.ident,
                "WatEnum requires all-unit variants — a variant carrying data has no \
                 single string spelling; write its accessors by hand rather than \
                 having the derive invent one",
            )
            .to_compile_error()
            .into();
        }
        idents.push(&v.ident);
    }
    if idents.is_empty() {
        return syn::Error::new_spanned(name, "WatEnum on an enum with no variants")
            .to_compile_error()
            .into();
    }

    let names: Vec<String> = idents.iter().map(|i| i.to_string()).collect();

    // Optional `#[wat_enum(type_path = "...")]` — the wat-side `defenum` this
    // mirrors, so the mirror can be CHECKED rather than asserted in a comment.
    let mut type_path: Option<String> = None;
    for attr in &input.attrs {
        if !attr.path().is_ident("wat_enum") {
            continue;
        }
        let parsed = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("type_path") {
                type_path = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                Ok(())
            } else {
                Err(meta.error("unknown wat_enum option; expected `type_path = \"...\"`"))
            }
        });
        if let Err(e) = parsed {
            return e.to_compile_error().into();
        }
    }

    let type_path_const = match type_path {
        Some(p) => quote! {
            /// The wat `defenum` this enum mirrors. Generated from
            /// `#[wat_enum(type_path = ...)]` so a test can compare the two
            /// instead of a comment claiming they match.
            #vis const WAT_TYPE_PATH: &'static str = #p;
        },
        None => quote! {},
    };

    quote! {
        impl #name {
            #type_path_const

            /// Every variant's spelling, in declaration order. Generated by
            /// `#[derive(WatEnum)]` — there is no hand-written list to go stale.
            #vis fn variants() -> &'static [&'static str] {
                &[ #( #names ),* ]
            }

            /// This variant's spelling. Exhaustive by construction.
            #vis fn as_str(&self) -> &'static str {
                match self { #( Self::#idents => #names, )* }
            }
        }

        impl ::core::str::FromStr for #name {
            type Err = ();
            fn from_str(s: &str) -> ::core::result::Result<Self, ()> {
                match s {
                    #( #names => ::core::result::Result::Ok(Self::#idents), )*
                    _ => ::core::result::Result::Err(()),
                }
            }
        }
    }
    .into()
}
