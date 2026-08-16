//! **wat is the source of truth; this crate generates the Rust.**
//!
//! One concern, one direction: a declaration written ONCE in a `.wat` file is read at BUILD
//! time and the Rust that must agree with it is emitted, so the two cannot drift — there is no
//! second copy to go stale, and nothing to remember to update.
//!
//! ## The name, and what it replaced
//!
//! This was `wat-enum-derive`, and that name described a SHAPE (enums) while the crate held two
//! opposite DIRECTIONS: `#[derive(WatEnum)]` generated accessors from a hand-written Rust enum
//! (Rust → Rust), and `wat_enum_from!` generates a Rust enum from a wat `defenum` (wat → Rust).
//! Naming the shape is why "where does a record generator go?" read as a new question when it is
//! the same concern with a different head keyword.
//!
//! `#[derive(WatEnum)]` is **DELETED** (2026-08-15). Its purpose — *"a Rust enum should not
//! hand-write its own variant list"* — is structurally superseded: under wat-as-truth no Rust
//! enum is a source at all, so there is no list for it to hand-write. `b2136b02` moved all six
//! to `wat_enum_from!` and left it with zero consumers. A gate whose success condition is its own
//! deletion is scaffolding.
//!
//! ## Why a separate crate (unchanged, and still the reason)
//!
//! `wat-macros` already depends on `wat-doc`, so a proc-macro living there could not be used BY
//! `wat-doc` — a cycle. This is the `wat-to-edn-derive` pattern: a leaf proc-macro crate
//! depending on nothing of wat's except `wat-reader`, usable from both. Read against its sibling
//! the pair states the two directions plainly — `wat-to-edn-derive` goes *to* EDN, this one comes
//! *from* the wat source.
//!
//! ## What lives here
//!
//! - [`wat_enum_from!`] — a `defenum` becomes a Rust enum.
//!
//! Anything else wat declares and Rust must agree with belongs here too, under the same rule: the
//! wat form is the single source, the Rust artifact is derived, and `include_str!` makes rustc
//! rebuild when the source moves.

use proc_macro::TokenStream;
use quote::quote;

// ─── wat_enum_from! ──────────────────────────────────────────────────────────
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

// ─── wat_record_from! ────────────────────────────────────────────────────────
//
// Arc 296. The record sibling of `wat_enum_from!`, and the second half of "wat is the
// source of truth for Rust". `b2136b02` gave the ENUMS a wat source; the aggregate
// declarations kept living in `register_builtin_types` as hand-written `AggregateDef`
// literals — the pre-b2136b02 state, one shape over.
//
// ## What it emits, and why NOT a Rust struct
//
// `wat_enum_from!` emits a Rust `enum` because Rust code MATCHES on those variants.
// Nothing in Rust matches on `:wat::kernel::Failure`'s shape; what Rust needs is for the
// TypeEnv to contain the declaration before any wat loads. So this macro emits the
// REGISTRATION — one `env.register_builtin(...)` statement — not a type.
//
// ## The one design decision, pinned
//
// It emits the field TYPE KEYWORDS AS STRINGS and lets the substrate's own
// `parse_type_expr` turn them into `TypeExpr` at registration time. A proc-macro crate
// cannot depend on the main crate (cycle), so the alternative was to REIMPLEMENT
// TypeExpr parsing here — a second parser for the same grammar, which is the exact
// duplication this macro exists to remove. Writing a second wat parser inside the macro
// that removes duplication would be self-refuting (the same reasoning `wat_enum_from!`
// records for using `wat-reader` rather than a hand-rolled scan).
//
// So: THIS macro reads the wat FORM; the EXISTING parser reads the wat TYPES. Neither
// job is done twice.
//
// ## Splices are refused, not handled
//
// `parse_aggregate_fields_with_splices` (the runtime's field walker) resolves
// `[~@:SomeSurface …]` against a live `TypeEnv`, which does not exist at compile time.
// The 13 builtin aggregates carry no splices. A splice therefore STOPS the build with a
// named error rather than being silently dropped — an emitted registration missing
// spliced fields would diverge from its own declaration and fail arc 054's
// `Existing::Equivalent` gate at load, which is a confusing place to learn about it.

#[proc_macro]
pub fn wat_record_from(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as WatRecordFromArgs);
    match expand_wat_record_from(&args) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

struct WatRecordFromArgs {
    env: syn::Ident,
    path: syn::LitStr,
    type_path: syn::LitStr,
}

impl syn::parse::Parse for WatRecordFromArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let env: syn::Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let path: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let type_path: syn::LitStr = input.parse()?;
        Ok(WatRecordFromArgs { env, path, type_path })
    }
}

fn expand_wat_record_from(args: &WatRecordFromArgs) -> syn::Result<TokenStream> {
    let rel = args.path.value();
    let manifest = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new_spanned(&args.path, "CARGO_MANIFEST_DIR unset — cannot resolve the wat file")
    })?;
    let abs: PathBuf = PathBuf::from(&manifest).join(&rel);
    let src = std::fs::read_to_string(&abs).map_err(|e| {
        syn::Error::new_spanned(&args.path, format!("cannot read `{}`: {e}", abs.display()))
    })?;

    let want = args.type_path.value();

    // THE REAL PARSER, never a hand-rolled scan — same rule as `wat_enum_from!`.
    let forms = wat_reader::parse_all_with_file(&src, &abs.to_string_lossy()).map_err(|e| {
        syn::Error::new_spanned(&args.path, format!("wat parse error in `{}`: {e:?}", abs.display()))
    })?;

    let mut nature: Option<&'static str> = None;
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut found = false;

    for form in &forms {
        let wat_reader::WatAST::List(items, _) = form else { continue };
        let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
        let this_nature = match head.as_str() {
            ":wat::core::defrecord" => "Record",
            ":wat::core::defstruct" => "Struct",
            _ => continue,
        };
        let Some(wat_reader::WatAST::Keyword(tp, _)) = items.get(1) else { continue };
        if tp != &want { continue }

        let Some(wat_reader::WatAST::Vector(items3, _)) = items.get(2) else {
            return Err(syn::Error::new_spanned(
                &args.type_path,
                format!("`{want}` in `{}`: expected a `[name <- :Type …]` field VECTOR as the third form", abs.display()),
            ));
        };

        // Triples: Symbol(name) Symbol(<-|:-) Keyword(:Type). No splices (see header).
        let mut i = 0usize;
        while i < items3.len() {
            match &items3[i] {
                wat_reader::WatAST::Symbol(id, _) if id.as_str() == "~@" => {
                    return Err(syn::Error::new_spanned(
                        &args.type_path,
                        format!("`{want}` uses a SURFACE SPLICE — splices resolve against a live TypeEnv, which does not exist at compile time. Declare the fields flat, or register this type by hand."),
                    ));
                }
                wat_reader::WatAST::Symbol(name, _) => {
                    let arrow = items3.get(i + 1);
                    let ty = items3.get(i + 2);
                    let (Some(wat_reader::WatAST::Symbol(a, _)), Some(wat_reader::WatAST::Keyword(t, _))) = (arrow, ty) else {
                        return Err(syn::Error::new_spanned(
                            &args.type_path,
                            format!("`{want}`: field `{}` is not a `name <- :Type` triple", name.as_str()),
                        ));
                    };
                    if a.as_str() != "<-" && a.as_str() != ":-" {
                        return Err(syn::Error::new_spanned(
                            &args.type_path,
                            format!("`{want}`: field `{}` uses arrow `{}` — expected `<-` or `:-`", name.as_str(), a.as_str()),
                        ));
                    }
                    fields.push((name.as_str().to_string(), t.clone()));
                    i += 3;
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        &args.type_path,
                        format!("`{want}`: unexpected item in the field vector: {other:?}"),
                    ));
                }
            }
        }
        nature = Some(this_nature);
        found = true;
        break;
    }

    if !found {
        return Err(syn::Error::new_spanned(
            &args.type_path,
            format!("no `(:wat::core::defrecord {want} …)` or `(:wat::core::defstruct {want} …)` in `{}` — wat is the source of truth, so the registration cannot be generated without it", abs.display()),
        ));
    }

    let env = &args.env;
    let type_path = &args.type_path;
    let nature_ident = syn::Ident::new(nature.unwrap(), proc_macro2::Span::call_site());
    let fnames: Vec<&String> = fields.iter().map(|(n, _)| n).collect();
    let ftypes: Vec<&String> = fields.iter().map(|(_, t)| t).collect();

    Ok(quote! {
        {
            // Rebuild when the wat file changes — without this the generated registration
            // goes stale against its own source, the very class this exists to remove.
            const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #rel));
            #env.register_builtin(crate::types::TypeDef::Aggregate(crate::types::AggregateDef {
                name: #type_path.into(),
                type_params: ::std::vec::Vec::new(),
                nature: crate::types::Nature::#nature_ident,
                restrictions: ::core::option::Option::None,
                fields: ::std::vec![
                    #(
                        (
                            #fnames.into(),
                            // The SUBSTRATE's own type parser — not a second one written here.
                            crate::types::parse_type_expr(#ftypes).unwrap_or_else(|e| panic!(
                                "wat_record_from!({}): field `{}` has type `{}` which the type parser rejects: {e:?}",
                                #type_path, #fnames, #ftypes,
                            )),
                        ),
                    )*
                ],
            }));
        }
    }
    .into())
}

// ─── wat_field_names_from! ───────────────────────────────────────────────────
//
// Arc 296 G. The field NAMES of a wat-declared aggregate, as a `&'static [&'static str]`,
// generated from the same `.wat` form `wat_record_from!` reads.
//
// ## Why this exists — and why the hand-written version was REJECTED
//
// G makes a `Value::Aggregate` carry its own field names, so rendering never needs a
// registry lookup and `:field-N` has no way to be produced. Most construction sites hold
// a registry and take names from the `AggregateDef`. But ~30 sites in the runtime build a
// value of a STATICALLY KNOWN type with no registry in scope (a `Fault` from a raised
// error, a `Failure` from a caught panic, rete's `Token`).
//
// The first attempt at G gave those sites a `static_field_names!("message", "location",
// "causes")` macro — a hand-transcription of a declaration that already exists. The
// builder stopped it: *"we did that exact move recently?"* A literal there is a second
// place the names are written, and a right-count/wrong-name literal would render
// CONFIDENTLY and wrongly — worse than the `:field-N` this arc is annihilating, because it
// looks like an answer.
//
// So the names come from the same wat form as everything else. There is no arm of this
// design in which a human types a field name into Rust.
//
// ## Shape
//
//     wat_field_names_from!(FAULT_FIELDS, "wat/core.wat", ":wat::core::Fault");
//     // → const FAULT_FIELDS: &[&str] = &["message", "location", "causes"];
//
// Module position (a `const` item), unlike `wat_record_from!` which is a statement. Same
// reader, same `include_str!` rebuild tracking, same STOP on splices.

#[proc_macro]
pub fn wat_field_names_from(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as WatFieldNamesArgs);
    match expand_wat_field_names(&args) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

struct WatFieldNamesArgs {
    ident: syn::Ident,
    path: syn::LitStr,
    type_path: syn::LitStr,
}

impl syn::parse::Parse for WatFieldNamesArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let path: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let type_path: syn::LitStr = input.parse()?;
        Ok(WatFieldNamesArgs { ident, path, type_path })
    }
}

fn expand_wat_field_names(args: &WatFieldNamesArgs) -> syn::Result<TokenStream> {
    let rel = args.path.value();
    let manifest = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new_spanned(&args.path, "CARGO_MANIFEST_DIR unset — cannot resolve the wat file")
    })?;
    let abs: PathBuf = PathBuf::from(&manifest).join(&rel);
    let src = std::fs::read_to_string(&abs).map_err(|e| {
        syn::Error::new_spanned(&args.path, format!("cannot read `{}`: {e}", abs.display()))
    })?;

    let want = args.type_path.value();
    let names = field_names_of(&src, &abs.to_string_lossy(), &want)
        .map_err(|m| syn::Error::new_spanned(&args.type_path, m))?;

    let ident = &args.ident;
    let type_path = &args.type_path;
    let n: Vec<&String> = names.iter().collect();

    Ok(quote! {
        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #rel));

        #[doc = concat!("GENERATED field names for `", #type_path, "`, read from its wat declaration.")]
        #[doc = ""]
        #[doc = "⛔ Do NOT hand-write these. **wat is the source of truth** — the names live in the"]
        #[doc = "`.wat` declaration and this const follows. A hand-written list here would be a"]
        #[doc = "second place the names are stated, and a right-count/wrong-name list renders"]
        #[doc = "confidently and wrongly — worse than the `:field-N` arc 296 annihilated."]
        pub(crate) const #ident: &[&str] = &[ #( #n ),* ];
    }
    .into())
}

/// The `(name, _)` field list of a `defrecord`/`defstruct` in `src`, names only.
///
/// Shared by `wat_record_from!` and `wat_field_names_from!` so the two cannot disagree about
/// what a declaration says — one walk, two emitters.
fn field_names_of(src: &str, file: &str, want: &str) -> Result<Vec<String>, String> {
    let forms = wat_reader::parse_all_with_file(src, file)
        .map_err(|e| format!("wat parse error in `{file}`: {e:?}"))?;
    for form in &forms {
        let wat_reader::WatAST::List(items, _) = form else { continue };
        let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
        if head != ":wat::core::defrecord" && head != ":wat::core::defstruct" { continue }
        let Some(wat_reader::WatAST::Keyword(tp, _)) = items.get(1) else { continue };
        if tp != want { continue }
        let Some(wat_reader::WatAST::Vector(fs, _)) = items.get(2) else {
            return Err(format!("`{want}`: expected a `[name <- :Type …]` field vector"));
        };
        let mut out = Vec::new();
        let mut i = 0usize;
        while i < fs.len() {
            match &fs[i] {
                wat_reader::WatAST::Symbol(id, _) if id.as_str() == "~@" => {
                    return Err(format!("`{want}` uses a SURFACE SPLICE — splices resolve against a live TypeEnv, which does not exist at compile time"));
                }
                wat_reader::WatAST::Symbol(name, _) => {
                    out.push(name.as_str().to_string());
                    i += 3;
                }
                other => return Err(format!("`{want}`: unexpected item in the field vector: {other:?}")),
            }
        }
        return Ok(out);
    }
    Err(format!("no `defrecord`/`defstruct` for `{want}` in `{file}` — wat is the source of truth, so the names cannot be generated without it"))
}

// ─── wat_enum_field_names_from! ────────────────────────────────────────────────
//
// Arc 296 G′. The field NAMES of one TAGGED variant of a wat-declared enum, as a
// `&'static [&'static str]`, read directly from the `.wat` `defenum` declaration's SOURCE
// TEXT at build time. `wat_field_names_from!` (above) covers `defrecord`/`defstruct`;
// `defenum` needs its own reader because an enum's field list is PER VARIANT, not one
// list per type.
//
// ## Why compile-time source text, not a runtime `TypeEnv` lookup
//
// A handful of builtin enums are declared via `defenum` in the bundled `.wat` stdlib
// (`:wat::spawn::ServiceEvent` in `wat/spawn.wat`, `:wat::sqlite::Cell` in
// `wat/sqlite.wat`) rather than as a `types.rs::register_builtin_types` Rust literal, so
// they are ABSENT from the cheap `TypeEnv::with_builtins()` that `builtin_enum_variant_names`
// (`runtime.rs`) uses for the Rust-registered ones. The first fix for that gap called
// `crate::freeze::env::build_env(vec![])` lazily behind a `OnceLock`, on the theory that the
// full baked-stdlib environment is the one registry that carries every `.wat`-declared type
// too. Measured (a `#[test]` in `runtime.rs`, isolated, `--test-threads=1`): it DEADLOCKS —
// `OnceLock::get_or_init`'s closure runs the full stdlib macro-expansion pipeline, which
// re-enters a call needing the SAME lock before the first call returns (`OnceLock` treats
// reentrant `get_or_init` as a hang, not an error — confirmed via `/proc/<pid>/stat`: zero
// CPU time accruing, all threads parked in `futex_do_wait`). Reading the `.wat` SOURCE TEXT
// at compile time — what `wat_field_names_from!` already does for records — has no such
// hazard: there is no runtime environment to build, so there is nothing to re-enter.
//
// ## Shape
//
//     wat_enum_field_names_from!(SERVICE_EVENT_MESSAGE_FIELDS, "wat/spawn.wat",
//         ":wat::spawn::ServiceEvent", "Message");
//     // → const SERVICE_EVENT_MESSAGE_FIELDS: &[&str] = &["idx", "msg"];

#[proc_macro]
pub fn wat_enum_field_names_from(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as WatEnumFieldNamesArgs);
    match expand_wat_enum_field_names(&args) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

struct WatEnumFieldNamesArgs {
    ident: syn::Ident,
    path: syn::LitStr,
    type_path: syn::LitStr,
    variant: syn::LitStr,
}

impl syn::parse::Parse for WatEnumFieldNamesArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let path: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let type_path: syn::LitStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let variant: syn::LitStr = input.parse()?;
        Ok(WatEnumFieldNamesArgs { ident, path, type_path, variant })
    }
}

fn expand_wat_enum_field_names(args: &WatEnumFieldNamesArgs) -> syn::Result<TokenStream> {
    let rel = args.path.value();
    let manifest = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new_spanned(&args.path, "CARGO_MANIFEST_DIR unset — cannot resolve the wat file")
    })?;
    let abs: PathBuf = PathBuf::from(&manifest).join(&rel);
    let src = std::fs::read_to_string(&abs).map_err(|e| {
        syn::Error::new_spanned(&args.path, format!("cannot read `{}`: {e}", abs.display()))
    })?;

    let want_type = args.type_path.value();
    let want_variant = args.variant.value();
    let names = enum_variant_field_names_of(&src, &abs.to_string_lossy(), &want_type, &want_variant)
        .map_err(|m| syn::Error::new_spanned(&args.variant, m))?;

    let ident = &args.ident;
    let type_path = &args.type_path;
    let variant = &args.variant;
    let n: Vec<&String> = names.iter().collect();

    Ok(quote! {
        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #rel));

        #[doc = concat!("GENERATED field names for `", #type_path, "::", #variant, "`, read from its wat `defenum` declaration.")]
        #[doc = ""]
        #[doc = "⛔ Do NOT hand-write these. **wat is the source of truth** — the names live in the"]
        #[doc = "`.wat` declaration and this const follows."]
        pub(crate) const #ident: &[&str] = &[ #( #n ),* ];
    }
    .into())
}

/// The `[name <- :Type …]` field list of ONE tagged variant inside a `defenum` in `src`.
/// Mirrors `field_names_of`'s walk (`wat_record_from!`'s reader) but for `defenum`'s
/// per-variant shape: a flat keyword/vector sequence after the mandatory purity marker,
/// the SAME shape `expand_wat_enum_from` (`wat_enum_from!`) already walks to collect
/// variant NAMES — this walk additionally opens the Vector to read one variant's ARGSPEC.
fn enum_variant_field_names_of(src: &str, file: &str, want_type: &str, want_variant: &str) -> Result<Vec<String>, String> {
    let forms = wat_reader::parse_all_with_file(src, file)
        .map_err(|e| format!("wat parse error in `{file}`: {e:?}"))?;
    for form in &forms {
        let wat_reader::WatAST::List(items, _) = form else { continue };
        let Some(wat_reader::WatAST::Keyword(head, _)) = items.first() else { continue };
        if head != ":wat::core::defenum" { continue }
        let Some(wat_reader::WatAST::Keyword(tp, _)) = items.get(1) else { continue };
        if tp != want_type { continue }
        // items[2] is the mandatory purity marker; variants follow as a flat
        // keyword/vector sequence (one-token lookahead: a Vector immediately after a
        // variant keyword makes it Tagged; anything else makes it Unit).
        let mut i = 3usize;
        while i < items.len() {
            let wat_reader::WatAST::Keyword(vname, _) = &items[i] else {
                return Err(format!("`{want_type}`: expected a variant keyword at position {i}"));
            };
            let vname_bare = vname.trim_start_matches(':');
            let next_is_vector = matches!(items.get(i + 1), Some(wat_reader::WatAST::Vector(_, _)));
            if !next_is_vector {
                // Unit variant.
                if vname_bare == want_variant {
                    return Err(format!(
                        "`{want_type}::{want_variant}` is a UNIT variant (no fields) — \
                         `wat_enum_field_names_from!` is for TAGGED variants only"
                    ));
                }
                i += 1;
                continue;
            }
            let Some(wat_reader::WatAST::Vector(fs, _)) = items.get(i + 1) else { unreachable!() };
            if vname_bare == want_variant {
                let mut out = Vec::new();
                let mut fi = 0usize;
                while fi < fs.len() {
                    match &fs[fi] {
                        wat_reader::WatAST::Symbol(id, _) if id.as_str() == "~@" => {
                            return Err(format!(
                                "`{want_type}::{want_variant}` uses a SURFACE SPLICE — splices \
                                 resolve against a live TypeEnv, which does not exist at compile time"
                            ));
                        }
                        wat_reader::WatAST::Symbol(name, _) => {
                            out.push(name.as_str().to_string());
                            fi += 3;
                        }
                        other => return Err(format!(
                            "`{want_type}::{want_variant}`: unexpected item in the field vector: {other:?}"
                        )),
                    }
                }
                return Ok(out);
            }
            i += 2;
        }
        return Err(format!("`defenum {want_type}` has no variant `{want_variant}`"));
    }
    Err(format!("no `(:wat::core::defenum {want_type} …)` in `{file}` — wat is the source of truth, so the names cannot be generated without it"))
}
