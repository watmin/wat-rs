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
// to a module) exposing `pub fn wat_sources() -> &'static
// [wat::WatSource]`. Omit `deps:` or write `deps: []` for
// no external deps.
//
// Expands to `fn main() -> Result<(), ::wat::harness::HarnessError>`
// calling `::wat::compose_and_run(source, &[deps.wat_sources()...])`.
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
    /// Arc 018 — optional. When absent, expansion defaults to
    /// `include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    /// "/wat/main.wat"))` — the opinionated entry location.
    source: Option<syn::Expr>,
    deps: Vec<Path>,
    /// Arc 017 — optional `loader: "..."` string-literal.
    /// Arc 018 default rule:
    /// - `loader` explicit: always honored.
    /// - `loader` absent, `source` absent: defaults to `"wat"`
    ///   (ScopedLoader at `<crate>/wat`, matching the implicit
    ///   `wat/main.wat` entry).
    /// - `loader` absent, `source` explicit: defaults to None
    ///   (InMemoryLoader — preserves pre-018 behavior for
    ///   single-file consumers).
    loader: Option<LitStr>,
}

impl Parse for MainInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // All three keys are optional (arc 018). Accept them in any
        // order; each at most once. Empty braces `wat::main! {}` is
        // the maximally-opinionated form.
        let mut source: Option<syn::Expr> = None;
        let mut deps: Vec<Path> = Vec::new();
        let mut deps_seen = false;
        let mut loader: Option<LitStr> = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            let key_str = key.to_string();
            input.parse::<syn::Token![:]>()?;
            match key_str.as_str() {
                "source" => {
                    if source.is_some() {
                        return Err(Error::new(key.span(), "duplicate `source:` arg"));
                    }
                    source = Some(input.parse()?);
                }
                "deps" => {
                    if deps_seen {
                        return Err(Error::new(key.span(), "duplicate `deps:` arg"));
                    }
                    deps_seen = true;
                    let content;
                    syn::bracketed!(content in input);
                    let parsed: Punctuated<Path, syn::Token![,]> =
                        content.parse_terminated(Path::parse_mod_style, syn::Token![,])?;
                    deps = parsed.into_iter().collect();
                }
                "loader" => {
                    if loader.is_some() {
                        return Err(Error::new(key.span(), "duplicate `loader:` arg"));
                    }
                    let lit: LitStr = input.parse().map_err(|e| {
                        Error::new(
                            e.span(),
                            "`loader:` expects a string literal — the ScopedLoader root path",
                        )
                    })?;
                    loader = Some(lit);
                }
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!(
                            "unknown `{}:` arg for wat::main!; expected `source:`, `deps:`, or `loader:`",
                            other
                        ),
                    ));
                }
            }
            // Accept the separating comma (optional after the last arg).
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            } else if !input.is_empty() {
                return Err(input.error("expected `,` between wat::main! args"));
            }
        }

        Ok(MainInput {
            source,
            deps,
            loader,
        })
    }
}

/// Declarative entry for wat-embedding Rust binaries. See module
/// docs.
#[proc_macro]
pub fn main(input: TokenStream) -> TokenStream {
    let MainInput {
        source,
        deps,
        loader,
    } = parse_macro_input!(input as MainInput);

    // Each dep is called twice — once for wat_sources() (wat
    // source side), once for register (Rust shim side). The two-
    // part external-crate contract per arc 013 slice 4a (renamed
    // from stdlib_sources in arc 015 slice 4).
    let stdlib_calls: Vec<TokenStream2> = deps
        .iter()
        .map(|p| quote! { #p::wat_sources() })
        .collect();
    let register_paths: Vec<TokenStream2> = deps
        .iter()
        .map(|p| quote! { #p::register })
        .collect();

    // Arc 018 — opinionated defaults.
    // `source` absent → implicit `include_str!(<crate>/wat/main.wat)`.
    // `loader` absent AND `source` absent → implicit `"wat"`.
    // `loader` absent AND `source` explicit → no loader (InMemoryLoader).
    let source_implicit = source.is_none();
    let source_expr: TokenStream2 = match source {
        Some(expr) => quote! { #expr },
        None => quote! {
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/wat/main.wat"))
        },
    };
    let effective_loader: Option<TokenStream2> = match (loader, source_implicit) {
        (Some(loader_lit), _) => Some(quote! { #loader_lit }),
        (None, true) => Some(quote! { "wat" }),
        (None, false) => None,
    };

    let expanded = match effective_loader {
        None => quote! {
            fn main() -> ::std::result::Result<(), ::wat::harness::HarnessError> {
                ::wat::compose_and_run(
                    #source_expr,
                    &[ #(#stdlib_calls),* ],
                    &[ #(#register_paths),* ],
                )
            }
        },
        Some(loader_expr) => quote! {
            fn main() -> ::std::result::Result<(), ::wat::harness::HarnessError> {
                // `loader:` is always resolved relative to the consumer
                // crate's source directory (CARGO_MANIFEST_DIR). This
                // makes `cargo run -p <crate>` from the workspace root
                // work identically to running from the crate's own dir
                // — the source tree's wat/ location is stable. Users
                // who need absolute or cwd-relative paths drop to
                // `Harness::from_source_with_deps_and_loader`.
                let __wat_loader_root = concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/",
                    #loader_expr
                );
                let __wat_loader: ::std::sync::Arc<
                    dyn ::wat::load::SourceLoader,
                > = ::std::sync::Arc::new(
                    ::wat::load::ScopedLoader::new(__wat_loader_root).map_err(|e| {
                        ::wat::harness::HarnessError::Startup(
                            ::wat::freeze::StartupError::Load(
                                ::wat::load::LoadError::from(e),
                            ),
                        )
                    })?,
                );
                ::wat::compose_and_run_with_loader(
                    #source_expr,
                    &[ #(#stdlib_calls),* ],
                    &[ #(#register_paths),* ],
                    __wat_loader,
                )
            }
        },
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
// exposing `pub fn wat_sources()` and `pub fn register(...)` per
// the arc 013 external-wat-crate contract. Omit or write `deps: []`
// for no external deps.
//
// Expands to `#[test] fn wat_suite()` calling
// `::wat::test_runner::run_and_assert(path, &[deps::wat_sources()...],
// &[deps::register...])`. On failure, the panic carries all
// individual test failure summaries — cargo's `#[test]` harness
// captures stdout + panic message and surfaces them.
//
// **Viewing per-wat-test output.** Cargo's default captures
// stdout from successful `#[test] fn`s; you'll see only
// `test wat_suite ... ok` at the outer layer. To see the inner
// "running N tests / test file.wat :: name ... ok (Xms)" lines
// that the runner prints, run with libtest's passthrough flags:
//
//     cargo test -- --nocapture       # stream output live
//     cargo test -- --show-output     # print captured output after each test
//
// Standard Cargo convention — silent on success by default, loud
// on failure. On failure the panic payload already includes every
// failing test's summary, so `cargo test` alone gives you what
// you need to debug.
//
// Cargo compiles each `tests/*.rs` file to its own test binary. One
// binary = one consistent dep set (first-call-wins install). Multiple
// test files with different dep sets live in separate
// `tests/*.rs` files; Cargo builds and runs each binary independently.

struct TestSuiteInput {
    path: syn::Expr,
    deps: Vec<Path>,
    /// Arc 017 — optional `loader: "..."` string-literal arg.
    /// Absent: macro expands to `run_and_assert` (FsLoader default
    /// — unrestricted filesystem, backward compatible). Present:
    /// expands to `run_and_assert_with_loader` with a ScopedLoader
    /// rooted at CARGO_MANIFEST_DIR/<path>, clamping every test
    /// file's `(load!)` to that scope.
    loader: Option<LitStr>,
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

        // Optional trailing: `, deps: [...]` and/or `, loader: "..."`.
        // Either order, each at most once.
        let mut deps: Vec<Path> = Vec::new();
        let mut loader: Option<LitStr> = None;

        while input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: syn::Ident = input.parse()?;
            let key_str = key.to_string();
            input.parse::<syn::Token![:]>()?;
            match key_str.as_str() {
                "deps" => {
                    if !deps.is_empty() {
                        return Err(Error::new(key.span(), "duplicate `deps:` arg"));
                    }
                    let content;
                    syn::bracketed!(content in input);
                    let parsed: Punctuated<Path, syn::Token![,]> =
                        content.parse_terminated(Path::parse_mod_style, syn::Token![,])?;
                    deps = parsed.into_iter().collect();
                }
                "loader" => {
                    if loader.is_some() {
                        return Err(Error::new(key.span(), "duplicate `loader:` arg"));
                    }
                    let lit: LitStr = input.parse().map_err(|e| {
                        Error::new(
                            e.span(),
                            "`loader:` expects a string literal — the ScopedLoader root path",
                        )
                    })?;
                    loader = Some(lit);
                }
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!(
                            "unknown `{}:` arg for wat::test_suite!; expected `deps:` or `loader:`",
                            other
                        ),
                    ));
                }
            }
        }

        if !input.is_empty() {
            return Err(input.error("unexpected tokens after wat::test_suite! args"));
        }

        Ok(TestSuiteInput {
            path,
            deps,
            loader,
        })
    }
}

/// Declarative test-suite entry — expands to `#[test] fn wat_suite()`.
/// See module docs.
#[proc_macro]
pub fn test_suite(input: TokenStream) -> TokenStream {
    let TestSuiteInput {
        path,
        deps,
        loader,
    } = parse_macro_input!(input as TestSuiteInput);

    let stdlib_calls: Vec<TokenStream2> = deps
        .iter()
        .map(|p| quote! { #p::wat_sources() })
        .collect();
    let register_paths: Vec<TokenStream2> = deps
        .iter()
        .map(|p| quote! { #p::register })
        .collect();

    let expanded = match loader {
        None => quote! {
            #[test]
            fn wat_suite() {
                ::wat::test_runner::run_and_assert(
                    ::std::path::Path::new(#path),
                    &[ #(#stdlib_calls),* ],
                    &[ #(#register_paths),* ],
                );
            }
        },
        Some(loader_lit) => quote! {
            #[test]
            fn wat_suite() {
                // Same CARGO_MANIFEST_DIR-relative convention as
                // `wat::main! { ..., loader: "..." }` — stable
                // regardless of cwd.
                let __wat_loader_root = concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/",
                    #loader_lit
                );
                let __wat_loader: ::std::sync::Arc<
                    dyn ::wat::load::SourceLoader,
                > = ::std::sync::Arc::new(
                    ::wat::load::ScopedLoader::new(__wat_loader_root)
                        .expect("wat::test_suite! loader path must exist"),
                );
                ::wat::test_runner::run_and_assert_with_loader(
                    ::std::path::Path::new(#path),
                    &[ #(#stdlib_calls),* ],
                    &[ #(#register_paths),* ],
                    __wat_loader,
                );
            }
        },
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

