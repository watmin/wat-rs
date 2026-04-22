//! Procedural macros for wat-rs.
//!
//! # `#[wat_dispatch]`
//!
//! Annotates a Rust `impl` block and generates the shim code that
//! exposes the type's methods to wat source via the `:rust::*`
//! namespace. See `wat-rs/docs/arc/2026/04/002-rust-interop-macro/MACRO-DESIGN.md`
//! for the full design.
//!
//! ```text
//! #[wat_dispatch(path = ":rust::lru::LruCache", scope = "thread_owned")]
//! impl<K: Hash + Eq, V: Clone> lru::LruCache<K, V> {
//!     fn new(cap: i64) -> Self { ... }
//!     fn put(&mut self, k: K, v: V) { ... }
//!     fn get(&mut self, k: K) -> Option<V> { ... }
//! }
//! ```
//!
//! Generates per-method dispatch and scheme fns, plus a `register()`
//! fn that wires everything into a `wat::rust_deps::RustDepsBuilder`.
//!
//! This is the BOOTSTRAP stage — attribute parsing only, no codegen
//! yet. Codegen lands in task #193; scope handling in #194.

use proc_macro::TokenStream;
use syn::{parse_macro_input, Error, ItemImpl, LitStr};

mod codegen;

/// The scope modes a shim can declare for its returned `Self` type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    /// Plain `Arc<T>` — no scope guard. For immutable / shareable
    /// Rust types (query results, immutable snapshots, etc.).
    Shared,
    /// `Arc<T>` + thread-id guard. Every op asserts the current
    /// thread is the owner before touching the interior. Used for
    /// single-thread-owned mutable state (lru::LruCache,
    /// rusqlite::Connection in some configs).
    ThreadOwned,
    /// Ownership transfers out of the `Arc` on first use.
    /// Subsequent access errors. Used for prepared-statement
    /// bindings, one-shot tokens.
    OwnedMove,
}

impl Scope {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "shared" => Some(Scope::Shared),
            "thread_owned" => Some(Scope::ThreadOwned),
            "owned_move" => Some(Scope::OwnedMove),
            _ => None,
        }
    }
}

/// Parsed `#[wat_dispatch(...)]` attribute arguments.
#[derive(Debug)]
pub(crate) struct WatDispatchAttr {
    /// The wat-level path the type is surfaced under, e.g.
    /// `:rust::lru::LruCache`. Required.
    pub path: String,
    /// Scope mode for Self-returning methods. Defaults to `shared`
    /// when omitted.
    pub scope: Scope,
    /// Phantom type-parameter names, e.g. `["K", "V"]` for LruCache.
    /// When non-empty, the macro emits the self-type as
    /// `TypeExpr::Parametric { head: <path>, args: [fresh_var; N] }`
    /// so wat-level annotations with `<K,V>` can unify. Empty = emit
    /// `TypeExpr::Path(<path>)`, used for types without phantom
    /// generics.
    pub type_params: Vec<String>,
}

impl syn::parse::Parse for WatDispatchAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Expect: path = "...", [scope = "..."], [type_params = "..."]
        let mut path: Option<String> = None;
        let mut scope: Scope = Scope::Shared;
        let mut type_params: Vec<String> = Vec::new();

        let pairs = input.parse_terminated(KeyValue::parse, syn::Token![,])?;
        for kv in pairs {
            match kv.key.to_string().as_str() {
                "path" => {
                    if path.is_some() {
                        return Err(Error::new_spanned(
                            &kv.key,
                            "duplicate `path` in wat_dispatch attribute",
                        ));
                    }
                    path = Some(kv.value.value());
                }
                "scope" => {
                    let s = kv.value.value();
                    scope = Scope::parse(&s).ok_or_else(|| {
                        Error::new_spanned(
                            &kv.value,
                            format!(
                                "invalid scope `{}`; expected one of: shared, thread_owned, owned_move",
                                s
                            ),
                        )
                    })?;
                }
                "type_params" => {
                    let s = kv.value.value();
                    // Comma-separated list of identifiers, e.g. "K,V".
                    type_params = s
                        .split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect();
                }
                other => {
                    return Err(Error::new_spanned(
                        &kv.key,
                        format!(
                            "unknown wat_dispatch argument `{}`; expected: path, scope, type_params",
                            other
                        ),
                    ));
                }
            }
        }

        let path = path.ok_or_else(|| {
            Error::new(
                input.span(),
                "wat_dispatch requires `path = \":rust::...\"`",
            )
        })?;

        Ok(WatDispatchAttr {
            path,
            scope,
            type_params,
        })
    }
}

/// Internal: one `key = "value"` pair inside the attribute.
struct KeyValue {
    key: syn::Ident,
    #[allow(dead_code)]
    eq: syn::Token![=],
    value: LitStr,
}

impl syn::parse::Parse for KeyValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(KeyValue {
            key: input.parse()?,
            eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

/// `#[wat_dispatch(path = "...", scope = "...")]` — shim generator
/// for Rust `impl` blocks. See module docs.
#[proc_macro_attribute]
pub fn wat_dispatch(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_attr = parse_macro_input!(attr as WatDispatchAttr);
    let parsed_impl = parse_macro_input!(item as ItemImpl);

    match codegen::emit(&parsed_attr, &parsed_impl) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ─── wat::main! — arc 013 slice 3 ────────────────────────────────────────
//
// Declarative entry point for Rust binaries that embed wat programs.
// Usage:
//
//     wat::main! {
//         source: include_str!("program.wat"),
//         deps: [wat_lru, wat_reqwest, wat_sqlite],
//     }
//
// `source:` is an expression (typically `include_str!`). `deps:` is
// an optional bracketed path list — each element is a crate (or path
// to a module) exposing `pub fn stdlib_sources() -> &'static
// [wat::stdlib::StdlibFile]`. Omit `deps:` or write `deps: []` for
// no external deps.
//
// Expands to `fn main() -> Result<(), ::wat::harness::HarnessError>`
// calling `::wat::compose_and_run(source, &[deps.stdlib_sources()...])`.
//
// Requires the consumer's Cargo.toml to have a dep named `wat` (the
// crate isn't configurable here). Users renaming the wat dep write
// their own main against the public Harness API.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::Path;

struct MainInput {
    source: syn::Expr,
    deps: Vec<Path>,
}

impl Parse for MainInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Required: `source: <expr>`
        let source_key: syn::Ident = input.parse()?;
        if source_key != "source" {
            return Err(Error::new(source_key.span(), "expected `source:` first"));
        }
        input.parse::<syn::Token![:]>()?;
        let source: syn::Expr = input.parse()?;

        // Optional: `, deps: [path, path, ...]`
        let mut deps: Vec<Path> = Vec::new();
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            if !input.is_empty() {
                let deps_key: syn::Ident = input.parse()?;
                if deps_key != "deps" {
                    return Err(Error::new(
                        deps_key.span(),
                        "expected `deps:` after `source:`",
                    ));
                }
                input.parse::<syn::Token![:]>()?;

                let content;
                syn::bracketed!(content in input);
                let parsed: Punctuated<Path, syn::Token![,]> =
                    content.parse_terminated(Path::parse_mod_style, syn::Token![,])?;
                deps = parsed.into_iter().collect();

                // Optional trailing comma after the bracketed list.
                let _ = input.parse::<syn::Token![,]>();
            }
        }

        if !input.is_empty() {
            return Err(input.error("unexpected tokens after wat::main! args"));
        }

        Ok(MainInput { source, deps })
    }
}

/// Declarative entry for wat-embedding Rust binaries. See module
/// docs.
#[proc_macro]
pub fn main(input: TokenStream) -> TokenStream {
    let MainInput { source, deps } = parse_macro_input!(input as MainInput);

    // Each dep is called twice — once for stdlib_sources() (wat
    // source side), once for register (Rust shim side). The two-
    // part external-crate contract per arc 013 slice 4a.
    let stdlib_calls: Vec<TokenStream2> = deps
        .iter()
        .map(|p| quote! { #p::stdlib_sources() })
        .collect();
    let register_paths: Vec<TokenStream2> = deps
        .iter()
        .map(|p| quote! { #p::register })
        .collect();

    let expanded = quote! {
        fn main() -> ::std::result::Result<(), ::wat::harness::HarnessError> {
            ::wat::compose_and_run(
                #source,
                &[ #(#stdlib_calls),* ],
                &[ #(#register_paths),* ],
            )
        }
    };

    expanded.into()
}

// ─── wat::test_suite! — arc 015 slice 2 ──────────────────────────────────
//
// Declarative test-suite entry for Rust binaries / libraries that want
// `cargo test` to discover and run a directory of `.wat` test files
// with external-wat-crate composition.
//
// Usage (inside any `tests/*.rs` integration test file):
//
//     wat::test_suite! {
//         path: "wat-tests",
//         deps: [wat_lru, wat_reqwest, wat_sqlite],
//     }
//
// `path:` is an expression (typically a string literal). It's resolved
// relative to CARGO_MANIFEST_DIR — Cargo's convention for integration
// tests' working directory is the crate root. `deps:` is an optional
// bracketed path list — each element is a crate (or path to a module)
// exposing `pub fn stdlib_sources()` and `pub fn register(...)` per
// the arc 013 external-wat-crate contract. Omit or write `deps: []`
// for no external deps.
//
// Expands to `#[test] fn wat_suite()` calling
// `::wat::test_runner::run_and_assert(path, &[deps::stdlib_sources()...],
// &[deps::register...])`. On failure, the panic carries all
// individual test failure summaries — cargo's `#[test]` harness
// captures stdout + panic message and surfaces them.
//
// Cargo compiles each `tests/*.rs` file to its own test binary. One
// binary = one consistent dep set (first-call-wins install). Multiple
// test files with different dep sets live in separate
// `tests/*.rs` files; Cargo builds and runs each binary independently.

struct TestSuiteInput {
    path: syn::Expr,
    deps: Vec<Path>,
}

impl Parse for TestSuiteInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Required: `path: <expr>`
        let path_key: syn::Ident = input.parse()?;
        if path_key != "path" {
            return Err(Error::new(path_key.span(), "expected `path:` first"));
        }
        input.parse::<syn::Token![:]>()?;
        let path: syn::Expr = input.parse()?;

        // Optional: `, deps: [path, path, ...]`
        let mut deps: Vec<Path> = Vec::new();
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            if !input.is_empty() {
                let deps_key: syn::Ident = input.parse()?;
                if deps_key != "deps" {
                    return Err(Error::new(
                        deps_key.span(),
                        "expected `deps:` after `path:`",
                    ));
                }
                input.parse::<syn::Token![:]>()?;

                let content;
                syn::bracketed!(content in input);
                let parsed: Punctuated<Path, syn::Token![,]> =
                    content.parse_terminated(Path::parse_mod_style, syn::Token![,])?;
                deps = parsed.into_iter().collect();

                let _ = input.parse::<syn::Token![,]>();
            }
        }

        if !input.is_empty() {
            return Err(input.error("unexpected tokens after wat::test_suite! args"));
        }

        Ok(TestSuiteInput { path, deps })
    }
}

/// Declarative test-suite entry — expands to `#[test] fn wat_suite()`.
/// See module docs.
#[proc_macro]
pub fn test_suite(input: TokenStream) -> TokenStream {
    let TestSuiteInput { path, deps } = parse_macro_input!(input as TestSuiteInput);

    let stdlib_calls: Vec<TokenStream2> = deps
        .iter()
        .map(|p| quote! { #p::stdlib_sources() })
        .collect();
    let register_paths: Vec<TokenStream2> = deps
        .iter()
        .map(|p| quote! { #p::register })
        .collect();

    let expanded = quote! {
        #[test]
        fn wat_suite() {
            ::wat::test_runner::run_and_assert(
                ::std::path::Path::new(#path),
                &[ #(#stdlib_calls),* ],
                &[ #(#register_paths),* ],
            );
        }
    };

    expanded.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> syn::Result<WatDispatchAttr> {
        syn::parse_str(input)
    }

    #[test]
    fn parse_path_only_defaults_to_shared_scope() {
        let attr = parse(r#"path = ":rust::lru::LruCache""#).expect("parse ok");
        assert_eq!(attr.path, ":rust::lru::LruCache");
        assert_eq!(attr.scope, Scope::Shared);
    }

    #[test]
    fn parse_path_and_scope() {
        let attr = parse(r#"path = ":rust::lru::LruCache", scope = "thread_owned""#)
            .expect("parse ok");
        assert_eq!(attr.path, ":rust::lru::LruCache");
        assert_eq!(attr.scope, Scope::ThreadOwned);
    }

    #[test]
    fn parse_scope_owned_move() {
        let attr =
            parse(r#"path = ":rust::x::Y", scope = "owned_move""#).expect("parse ok");
        assert_eq!(attr.scope, Scope::OwnedMove);
    }

    #[test]
    fn parse_order_path_or_scope_first_both_work() {
        let a =
            parse(r#"path = ":rust::a::B", scope = "shared""#).expect("parse a");
        let b =
            parse(r#"scope = "shared", path = ":rust::a::B""#).expect("parse b");
        assert_eq!(a.path, b.path);
        assert_eq!(a.scope, b.scope);
    }

    #[test]
    fn missing_path_rejected() {
        let err = parse(r#"scope = "shared""#).unwrap_err();
        assert!(err.to_string().contains("path = \":rust::..."));
    }

    #[test]
    fn invalid_scope_rejected() {
        let err = parse(r#"path = ":rust::x::Y", scope = "bogus""#).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid scope"));
        assert!(msg.contains("shared"));
        assert!(msg.contains("thread_owned"));
        assert!(msg.contains("owned_move"));
    }

    #[test]
    fn unknown_key_rejected() {
        let err =
            parse(r#"path = ":rust::x::Y", mystery = "?""#).unwrap_err();
        assert!(err.to_string().contains("unknown wat_dispatch argument"));
    }

    #[test]
    fn duplicate_path_rejected() {
        let err = parse(r#"path = ":rust::a::B", path = ":rust::c::D""#).unwrap_err();
        assert!(err.to_string().contains("duplicate `path`"));
    }
}

