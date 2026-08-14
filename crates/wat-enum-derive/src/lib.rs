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

// ─── wat_enum_from! — WAT IS THE SOURCE OF TRUTH ─────────────────────────────
//
// Builder ruling, 2026-08-15: *"your instinct was to use wat as a source of truth
// for rust code..... that's my pick."*
//
// `#[derive(WatEnum)]` above still derives Rust-from-Rust: the enum is written by
// hand and the accessors follow. This inverts it. The `defenum` in the `.wat` file
// IS the list; the Rust enum is generated from it. There is then exactly ONE
// list, it is written in wat, and the host language conforms to the language it
// hosts.
//
// What that dissolves: `every_rust_enum_matches_its_wat_defenum` — a test written
// hours before this to compare the two. A generated enum cannot drift from its
// generator, so the gate's success condition is its own deletion.
//
// ## Two constraints the implementation had to meet, both measured
//
// 1. **The real parser, not a hand-rolled scan.** `wat-macros` already depends on
//    `wat-reader` for exactly this reason (its manifest: *"so discovery can use the
//    REAL parser, eliminating the hand-rolled lexer"*). The variant list comes from
//    `parse_all_with_file`. Writing a second wat parser inside a macro that exists
//    to remove duplication would be self-refuting.
//
// 2. **The lexer DISCARDS comments** (`lexer.rs:42` — "Line comments … skipped"),
//    so per-variant doc text cannot come from the AST. It is read from the raw
//    source instead: the `;;` lines immediately above a variant become its `///`.
//    Two readers over one file is a smell, and it is a deliberate one — the
//    structure comes from the parser (correctness), the prose from the text layer
//    (the only place it survives). If `defenum` ever carries docs as data, this
//    half goes away.
//
// ## Rebuild-on-change
//
// The expansion emits `const _: &str = include_str!(...)` so rustc tracks the
// `.wat` file. Without it the generated enum would go stale against its own
// source — which is this entire failure class again, one level up.

use std::path::PathBuf;

#[proc_macro]
pub fn wat_enum_from(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as WatEnumFromArgs);
    match expand_wat_enum_from(&args) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

struct WatEnumFromArgs {
    vis: syn::Visibility,
    ident: syn::Ident,
    path: syn::LitStr,
    type_path: syn::LitStr,
}

impl syn::parse::Parse for WatEnumFromArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis: syn::Visibility = input.parse()?;
        input.parse::<syn::Token![enum]>()?;
        let ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let path: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let type_path: syn::LitStr = input.parse()?;
        Ok(WatEnumFromArgs { vis, ident, path, type_path })
    }
}

fn expand_wat_enum_from(args: &WatEnumFromArgs) -> syn::Result<TokenStream> {
    let rel = args.path.value();
    let manifest = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new_spanned(&args.path, "CARGO_MANIFEST_DIR unset — cannot resolve the wat file")
    })?;
    let abs: PathBuf = PathBuf::from(&manifest).join(&rel);
    let src = std::fs::read_to_string(&abs).map_err(|e| {
        syn::Error::new_spanned(&args.path, format!("cannot read `{}`: {e}", abs.display()))
    })?;

    let want = args.type_path.value();

    // ── STRUCTURE: the real parser, never a hand-rolled scan ──────────────
    let forms = wat_reader::parse_all_with_file(&src, &abs.to_string_lossy())
        .map_err(|e| syn::Error::new_spanned(&args.path, format!("wat parse error in `{}`: {e:?}", abs.display())))?;

    let mut variants: Vec<String> = Vec::new();
    let mut found = false;
    for form in &forms {
        let wat_reader::WatAST::List(items, _) = form else { continue };
        let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
        if head != ":wat::core::defenum" { continue }
        let Some(wat_reader::WatAST::Keyword(tp, _)) = items.get(1) else { continue };
        if tp != &want { continue }
        found = true;
        // items[2] is the purity marker (`:wat::enum::Pure`); variants follow.
        for it in items.iter().skip(3) {
            if let wat_reader::WatAST::Keyword(k, _) = it {
                variants.push(k.trim_start_matches(':').to_string());
            }
        }
        break;
    }
    if !found {
        return Err(syn::Error::new_spanned(
            &args.type_path,
            format!("no `(:wat::core::defenum {want} …)` in `{}` — wat is the source of truth, so the enum cannot be generated without it", abs.display()),
        ));
    }
    if variants.is_empty() {
        return Err(syn::Error::new_spanned(&args.type_path, format!("`defenum {want}` declares no variants")));
    }

    // ── PROSE: the text layer, because the lexer discards comments ────────
    // The `;;` lines immediately above a variant become its `///`.
    let mut docs: Vec<Vec<String>> = vec![Vec::new(); variants.len()];
    let mut pending: Vec<String> = Vec::new();
    let mut idx = 0usize;
    let mut inside = false;
    for line in src.lines() {
        let t = line.trim();
        if t.starts_with(&format!("(:wat::core::defenum {want}")) { inside = true; continue }
        if !inside { continue }
        if let Some(c) = t.strip_prefix(";;") {
            pending.push(c.trim().to_string());
            continue;
        }
        if let Some(v) = t.trim_end_matches(')').strip_prefix(':') {
            if idx < variants.len() && v == variants[idx] {
                docs[idx] = std::mem::take(&mut pending);
                idx += 1;
            }
        }
        if t.ends_with(')') { break }
    }

    let ident = &args.ident;
    let vis = &args.vis;
    let type_path = &args.type_path;
    let idents: Vec<syn::Ident> = variants.iter().map(|v| syn::Ident::new(v, ident.span())).collect();
    let names: Vec<&String> = variants.iter().collect();
    let doc_attrs: Vec<proc_macro2::TokenStream> = docs
        .iter()
        .map(|lines| {
            let ls = lines.iter().map(|l| quote! { #[doc = #l] });
            quote! { #(#ls)* }
        })
        .collect();

    Ok(quote! {
        // Rebuild when the wat file changes. Without this the generated enum goes
        // stale against its own source — the very class this exists to remove.
        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #rel));

        #[doc = concat!("GENERATED from `(:wat::core::defenum ", #type_path, " …)`.")]
        #[doc = ""]
        #[doc = "⛔ Do NOT edit these variants here. **wat is the source of truth** (builder"]
        #[doc = "ruling, 2026-08-15) — add or remove a variant in the `.wat` file and this"]
        #[doc = "enum follows. There is exactly one list and it is written in wat."]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #vis enum #ident {
            #( #doc_attrs #idents, )*
        }

        impl #ident {
            #[doc = "The wat `defenum` this enum was generated FROM."]
            #vis const WAT_TYPE_PATH: &'static str = #type_path;

            #[doc = "Every variant's spelling, in the order the wat `defenum` declares them."]
            #vis fn variants() -> &'static [&'static str] { &[ #( #names ),* ] }

            #[doc = "This variant's spelling. Exhaustive by construction."]
            #vis fn as_str(&self) -> &'static str {
                match self { #( Self::#idents => #names, )* }
            }
        }

        impl ::core::str::FromStr for #ident {
            type Err = ();
            fn from_str(s: &str) -> ::core::result::Result<Self, ()> {
                match s {
                    #( #names => ::core::result::Result::Ok(Self::#idents), )*
                    _ => ::core::result::Result::Err(()),
                }
            }
        }
    }
    .into())
}
