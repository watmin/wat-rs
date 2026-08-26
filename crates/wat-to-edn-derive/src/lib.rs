//! `#[derive(ToEdn)]` — structural EDN body generator for kind-enums and structs.
//!
//! This crate is the companion derive macro for `wat-edn`'s `ToEdn` trait.
//! It has NO dependencies on `wat` or `wat-reader`, so any crate that
//! depends on `wat-edn` (including `wat-reader` itself) can
//! `#[derive(wat_edn::ToEdn)]` without a dependency cycle.
//!
//! ## Supported shapes
//!
//! - **Struct variant** `Foo { a_b: T, c: U }` →
//!   `#wat.kernel/Foo {:a-b <a_b.to_edn()> :c <c.to_edn()>}`.
//!   Snake-case field idents map to kebab-case EDN keys. Declaration order
//!   is preserved.
//! - **Unit variant** `Bar` →
//!   `#wat.kernel/Bar {}` (empty map body).
//! - **Single-field tuple variant** `Fetch(T)` with `#[to_edn(key = "cause")]`
//!   on the variant → `#wat.kernel/Fetch {:cause <__0.to_edn()>}`.
//!   The `key` is REQUIRED (the field has no Rust ident).
//!   Multi-field tuple variants and keyless single-field tuples are a `compile_error!`.
//! - **Named-field struct** `struct Foo { a: T, b: U }` →
//!   `#<ns>/<Name> {:a <a.to_edn()> :b <b.to_edn()>}`.
//!   Namespace controlled by `#[to_edn(namespace = <path>)]`; defaults to
//!   `"wat.kernel"`.
//!
//! ## The helper attribute `#[to_edn(...)]` (Strike 2a)
//!
//! ### Field-level directives
//!
//! - **`#[to_edn(key = "kebab-key")]`** — Override the default snake→kebab EDN
//!   key. Value MUST be a `LitStr`.
//! - **`#[to_edn(via = path::to::fn)]`** — Instead of calling `field.to_edn()`,
//!   emit `(key, path::to::fn(field))`. Fn signature: `fn(&FieldType) -> ::wat_edn::OwnedValue`.
//! - **Span fields** — A field whose type's last path segment is `Span` is
//!   emitted via `crate::edn::contract::push_span_field`. The key defaults to snake→kebab
//!   or the `#[to_edn(key="…")]` override.
//!
//! ### Variant-level directives
//!
//! - **`#[to_edn(literal(k1 = "v1", k2 = "v2", …))]`** — Prepend synthetic
//!   constant string fields before the field-derived pairs.
//! - **`#[to_edn(via(key = "k", fn = path::to::fn, args(a, b, c)))]`** —
//!   Append a computed field after the field pairs.
//! - **`#[to_edn(key = "…")]`** — Name the EDN key for a single-field tuple
//!   variant's nameless field.
//!
//! ### Grammar constraint (the top rung)
//!
//! Every value in a `#[to_edn(...)]` annotation is grammar-constrained to a
//! safe token class (`LitStr` or bare path). Unknown keys emit `compile_error!`.
//!
//! ## Wall
//!
//! Every field value (except Span and `via`-overridden fields) is serialized
//! via `field.to_edn()`. If a field type does not implement
//! `::wat_edn::ToEdn`, rustc emits:
//!
//! ```text
//! error[E0277]: the trait bound `T: ToEdn` is not satisfied
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::Parse as _;
use syn::{Data, DeriveInput, Fields};

// ── Entry points ───────────────────────────────────────────────────────────────

/// `#[derive(ToEdn)]` — structural EDN body generator for kind-enums and structs.
///
/// Generates `impl ::wat_edn::ToEdn for <Type>` from the Rust type so there
/// is no hand-written `to_edn()` match body. An embedded field whose type does
/// not implement `ToEdn` is a compile error (the structural wall arc 296 promises).
///
/// Write-only: does NOT submit an `EdnSchema` entry — the type can be emitted
/// but not read back.  Use `#[derive(Edn)]` for types that must round-trip.
///
/// ## Helper attribute `#[to_edn(...)]`
///
/// Declared so the compiler does not reject `#[to_edn(...)]` on fields/variants.
/// See the crate-level docs for the full attribute grammar.
#[proc_macro_derive(ToEdn, attributes(to_edn))]
pub fn derive_to_edn(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as syn::DeriveInput);
    derive_to_edn_inner(parsed).into()
}

/// `#[derive(Edn)]` — the round-trip derive (arc 296 stone D).
///
/// Generates BOTH:
/// 1. `impl ::wat_edn::ToEdn for <Type>` (the write half — identical to `ToEdn`).
/// 2. An `::inventory::submit!(::wat_edn::EdnSchema { … })` block (the register
///    half) so the reader (`reconstruct_record` in `edn/render.rs`) can reconstruct
///    the type from its EDN form without any hand-written registration.
///
/// The consumer crate must have `inventory = "0.3"` as a direct dependency so
/// `::inventory::submit!` in the generated code resolves.
///
/// ## STOP-2
///
/// If any non-skipped field's Rust type has no known wat type-path mapping,
/// `#[derive(Edn)]` emits a `compile_error!` — the type is not safely
/// round-trippable until a mapping is added.  Use `#[to_edn(skip)]` on
/// fields that are intentionally write-only.
#[proc_macro_derive(Edn, attributes(to_edn))]
pub fn derive_edn(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as syn::DeriveInput);
    // Generate the write impl (identical to ToEdn).
    let write_impl = derive_to_edn_inner(parsed.clone());
    // Generate the EdnSchema submit block (the register half).
    let schema_submit = derive_edn_schema(&parsed);
    quote! {
        #write_impl
        #schema_submit
    }
    .into()
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Convert a snake_case identifier string to kebab-case EDN key.
///
/// `setter_head` → `"setter-head"`, `field` → `"field"`, `a_b` → `"a-b"`.
fn snake_to_kebab(s: &str) -> String {
    s.replace('_', "-")
}

// ── EdnSchema code generator (arc 296 stone D) ────────────────────────────────

/// Map a Rust `syn::Type` to the canonical wat type-path string for use in an
/// `EdnSchema.fields` entry.
///
/// Only plain named types (no generics) with known wat equivalents are mapped.
/// Everything else is a STOP-2: the caller should either add a mapping or
/// annotate the field with `#[to_edn(skip)]`.
///
/// The returned `&'static str` is a string literal embedded in the submit block.
fn rust_type_to_wat_path(ty: &syn::Type) -> Result<&'static str, TokenStream2> {
    let syn::Type::Path(tp) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "#[derive(Edn)] STOP-2: field type is not a simple path (reference, \
             generic wrapper, etc.) — no automatic wat type-path mapping exists. \
             Add #[to_edn(skip)] to exclude from the schema, or use a plain named type.",
        )
        .to_compile_error());
    };

    if tp.qself.is_some() {
        return Err(syn::Error::new_spanned(
            ty,
            "#[derive(Edn)] STOP-2: qualified self types are not supported in schema generation.",
        )
        .to_compile_error());
    }

    let seg = tp
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new_spanned(ty, "#[derive(Edn)]: empty type path").to_compile_error())?;

    // Reject generics: `Option<T>`, `Arc<T>`, etc.
    match &seg.arguments {
        syn::PathArguments::None => {}
        _ => {
            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "#[derive(Edn)] STOP-2: generic type `{}<…>` has no automatic wat \
                     type-path mapping. Add #[to_edn(skip)] to exclude from the schema.",
                    seg.ident,
                ),
            )
            .to_compile_error());
        }
    }

    match seg.ident.to_string().as_str() {
        "i64"    => Ok(":wat::core::i64"),
        "i32"    => Ok(":wat::core::i64"),
        "u32"    => Ok(":wat::core::i64"),
        "usize"  => Ok(":wat::core::i64"),
        "bool"   => Ok(":wat::core::bool"),
        "String" => Ok(":wat::core::String"),
        other    => Err(syn::Error::new_spanned(
            ty,
            format!(
                "#[derive(Edn)] STOP-2: Rust type `{}` has no registered wat type-path \
                 mapping. Add a mapping in `rust_type_to_wat_path` (for standard types) \
                 or annotate the field with #[to_edn(skip)].",
                other,
            ),
        )
        .to_compile_error()),
    }
}

/// Generate `::inventory::submit!(::wat_edn::EdnSchema { … })` blocks for all
/// tagged types produced by the `Edn` derive.
///
/// - **Struct**: one submit block covering all non-skipped, non-via named fields.
/// - **Enum**: one submit block per variant, covering that variant's fields.
///
/// STOP-2 is surfaced as a `compile_error!` token stream when a field's Rust
/// type has no known wat type-path mapping or when a `via`-annotated field is
/// encountered (the logical type is ambiguous without a mapping override).
fn derive_edn_schema(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let name_str = name.to_string();

    let enum_attr = match parse_enum_attrs(input) {
        Ok(a)  => a,
        Err(e) => return e,
    };
    let namespace_tokens: TokenStream2 = match enum_attr.namespace {
        Some(path) => quote! { #path },
        None       => quote! { "wat.kernel" },
    };

    match &input.data {
        // ── Struct: one EdnSchema submit ─────────────────────────────────────
        Data::Struct(data_struct) => {
            let named = match &data_struct.fields {
                Fields::Named(f) => &f.named,
                _ => {
                    return syn::Error::new_spanned(
                        name,
                        "#[derive(Edn)] supports named-field structs only; \
                         tuple/unit structs have no field names to emit.",
                    )
                    .to_compile_error();
                }
            };

            let mut field_pairs: Vec<TokenStream2> = Vec::new();
            for f in named {
                let fid = f.ident.as_ref().expect("named field has ident");

                let field_attr = match parse_field_attrs(f) {
                    Ok(a)  => a,
                    Err(e) => return e,
                };

                // skip fields are intentionally excluded from the schema.
                if field_attr.skip {
                    continue;
                }

                // via fields use custom serialization — the logical type is
                // ambiguous without an annotation (STOP-2).
                if field_attr.via_fn.is_some() {
                    return syn::Error::new_spanned(
                        f,
                        "#[derive(Edn)] STOP-2: field with #[to_edn(via = …)] has an \
                         unknown logical wat type; the schema entry cannot be generated \
                         automatically. Add #[to_edn(skip)] to exclude this field from \
                         the schema.",
                    )
                    .to_compile_error();
                }

                let edn_key = field_attr
                    .key_override
                    .unwrap_or_else(|| snake_to_kebab(&fid.to_string()));

                let wat_path = match rust_type_to_wat_path(&f.ty) {
                    Ok(p)  => p,
                    Err(e) => return e,
                };

                field_pairs.push(quote! { (#edn_key, #wat_path) });
            }

            quote! {
                ::inventory::submit! {
                    ::wat_edn::EdnSchema {
                        tag_ns:   #namespace_tokens,
                        tag_name: #name_str,
                        fields:   &[#(#field_pairs),*],
                    }
                }
            }
        }

        // ── Enum: one EdnSchema submit per variant ────────────────────────────
        Data::Enum(data_enum) => {
            let mut submits: Vec<TokenStream2> = Vec::new();

            for variant in &data_enum.variants {
                let variant_name_str = variant.ident.to_string();

                match &variant.fields {
                    // Named-field variant: same logic as struct.
                    Fields::Named(named_fields) => {
                        let mut field_pairs: Vec<TokenStream2> = Vec::new();
                        for f in &named_fields.named {
                            let fid = f.ident.as_ref().expect("named field has ident");
                            let field_attr = match parse_field_attrs(f) {
                                Ok(a)  => a,
                                Err(e) => return e,
                            };
                            if field_attr.skip {
                                continue;
                            }
                            if field_attr.via_fn.is_some() {
                                return syn::Error::new_spanned(
                                    f,
                                    "#[derive(Edn)] STOP-2: via field in enum variant \
                                     has ambiguous logical type; add #[to_edn(skip)].",
                                )
                                .to_compile_error();
                            }
                            let edn_key = field_attr
                                .key_override
                                .unwrap_or_else(|| snake_to_kebab(&fid.to_string()));
                            let wat_path = match rust_type_to_wat_path(&f.ty) {
                                Ok(p)  => p,
                                Err(e) => return e,
                            };
                            field_pairs.push(quote! { (#edn_key, #wat_path) });
                        }
                        submits.push(quote! {
                            ::inventory::submit! {
                                ::wat_edn::EdnSchema {
                                    tag_ns:   #namespace_tokens,
                                    tag_name: #variant_name_str,
                                    fields:   &[#(#field_pairs),*],
                                }
                            }
                        });
                    }

                    // Unit variant: empty fields.
                    Fields::Unit => {
                        submits.push(quote! {
                            ::inventory::submit! {
                                ::wat_edn::EdnSchema {
                                    tag_ns:   #namespace_tokens,
                                    tag_name: #variant_name_str,
                                    fields:   &[],
                                }
                            }
                        });
                    }

                    // Single-field tuple variant: one field named by #[to_edn(key)].
                    Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                        let variant_attr = match parse_variant_attrs(variant) {
                            Ok(a)  => a,
                            Err(e) => return e,
                        };
                        let edn_key = match variant_attr.key {
                            Some(k) => k,
                            None    => return syn::Error::new_spanned(
                                variant,
                                "#[derive(Edn)] STOP-2: single-field tuple variant requires \
                                 #[to_edn(key = \"…\")] to name the schema field.",
                            )
                            .to_compile_error(),
                        };
                        let field = f.unnamed.iter().next().expect("len == 1");
                        let field_attr = match parse_field_attrs(field) {
                            Ok(a)  => a,
                            Err(e) => return e,
                        };
                        if field_attr.skip {
                            submits.push(quote! {
                                ::inventory::submit! {
                                    ::wat_edn::EdnSchema {
                                        tag_ns:   #namespace_tokens,
                                        tag_name: #variant_name_str,
                                        fields:   &[],
                                    }
                                }
                            });
                        } else if field_attr.via_fn.is_some() {
                            return syn::Error::new_spanned(
                                variant,
                                "#[derive(Edn)] STOP-2: via field in tuple variant has \
                                 ambiguous logical type; add #[to_edn(skip)].",
                            )
                            .to_compile_error();
                        } else {
                            let wat_path = match rust_type_to_wat_path(&field.ty) {
                                Ok(p)  => p,
                                Err(e) => return e,
                            };
                            submits.push(quote! {
                                ::inventory::submit! {
                                    ::wat_edn::EdnSchema {
                                        tag_ns:   #namespace_tokens,
                                        tag_name: #variant_name_str,
                                        fields:   &[(#edn_key, #wat_path)],
                                    }
                                }
                            });
                        }
                    }

                    // Multi-field tuple: no safe key assignment.
                    Fields::Unnamed(_) => {
                        return syn::Error::new_spanned(
                            variant,
                            "#[derive(Edn)] STOP-2: multi-field tuple variants have \
                             no safe key assignment; convert to a named-field variant.",
                        )
                        .to_compile_error();
                    }
                }
            }

            quote! { #(#submits)* }
        }

        _ => syn::Error::new_spanned(
            name,
            "#[derive(Edn)] is supported on enums and named-field structs only.",
        )
        .to_compile_error(),
    }
}

// Stone B (arc 296): `is_span_type` deleted — `Span: ToEdn` via `#[derive]`
// in `wat-reader`, so the derive emits `.to_edn()` on Span fields exactly as
// for any other ToEdn type. No special-casing needed. FACTVM NON PACTVM.

// ── Attribute data structures ─────────────────────────────────────────────────

/// Enum-level `#[to_edn(...)]` annotations (applied to the enum itself, not a
/// variant or field).
struct EnumAttr {
    /// `namespace = <path>`: the Rust path (e.g. `crate::error_ns::CHECK`) that
    /// resolves to the namespace string used for every variant's EDN tag.
    /// Absent → defaults to the back-compat `"wat.kernel"` literal.
    namespace: Option<syn::Path>,
}

/// Field-level `#[to_edn(...)]` annotations, collected from all `#[to_edn]`
/// attrs on a single field.
struct FieldAttr {
    /// `key = "..."`: override the EDN keyword key for this field.
    key_override: Option<String>,
    /// `via = path`: call `path(field)` (where `field: &FieldType`) instead of
    /// `field.to_edn()`. Returns `::wat_edn::OwnedValue`.
    via_fn: Option<syn::Path>,
    /// `skip`: do NOT emit this field as a plain pair. The field ident is still
    /// bound in the match arm (so it remains available as an argument to a
    /// variant-level `via(args(...))`), but no `(:key value)` pair is pushed for
    /// it.
    skip: bool,
}

/// Variant-level `#[to_edn(...)]` annotations, collected from all `#[to_edn]`
/// attrs on a single variant.
struct VariantAttr {
    /// `literal(k = "v", …)`: synthetic constant string pairs to PREPEND.
    literal_pairs: Vec<(String, String)>,
    /// `via(key="k", fn=path, args(a,b,c))`: computed field to APPEND.
    computed_via: Option<ComputedVia>,
    /// `key = "…"`: the EDN key for a single-field tuple variant's nameless
    /// field. Required when the variant is `Foo(T)` (single unnamed field).
    /// Illegal on Named or Unit variants — use field-level `#[to_edn(key)]`.
    key: Option<String>,
}

/// Parsed `#[to_edn(via(key = "k", fn = path, args(a, b, c)))]` on a variant.
struct ComputedVia {
    key: String,
    fn_path: syn::Path,
    args: Vec<syn::Ident>,
}

// ── Enum attribute parser ─────────────────────────────────────────────────────

/// Parse all `#[to_edn(...)]` attributes on the ENUM itself into an `EnumAttr`.
///
/// Allowed form:
/// - `namespace = <path>` → the Rust path to the namespace const (e.g. `crate::error_ns::CHECK`).
///   Value MUST be a bare path (ident or `a::b::c`); a string literal is rejected.
fn parse_enum_attrs(input: &DeriveInput) -> Result<EnumAttr, TokenStream2> {
    let mut namespace: Option<syn::Path> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("to_edn") {
            continue;
        }
        // parse_args_with parses the tokens inside `#[to_edn(...)]`.
        struct NamespaceParse(syn::Path);
        impl syn::parse::Parse for NamespaceParse {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let ident: syn::Ident = input.parse().map_err(|e| {
                    syn::Error::new(
                        e.span(),
                        "#[to_edn(...)] on an enum: expected a directive name; \
                         allowed enum-level directive: namespace",
                    )
                })?;
                if ident != "namespace" {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "unknown #[to_edn(...)] directive `{}` on enum; \
                             allowed enum-level directive: namespace",
                            ident
                        ),
                    ));
                }
                input.parse::<syn::Token![=]>()?;
                // Value MUST be a bare path — reject string literal explicitly.
                if input.peek(syn::LitStr) {
                    let lit: syn::LitStr = input.parse().unwrap();
                    return Err(syn::Error::new_spanned(
                        lit,
                        "#[to_edn(namespace = ...)] value must be a bare path \
                         (e.g. crate::error_ns::CHECK), not a string literal; \
                         inline string literals are forbidden to close the smuggle hole",
                    ));
                }
                let path: syn::Path = input.parse().map_err(|e| {
                    syn::Error::new(
                        e.span(),
                        "#[to_edn(namespace = ...)] value must be a bare path \
                         (e.g. crate::error_ns::CHECK)",
                    )
                })?;
                if !input.is_empty() {
                    return Err(syn::Error::new(
                        input.span(),
                        "#[to_edn(namespace = ...)] expects a bare path only; \
                         trailing tokens are forbidden",
                    ));
                }
                Ok(NamespaceParse(path))
            }
        }

        let parsed = attr
            .parse_args::<NamespaceParse>()
            .map_err(|e| e.to_compile_error())?;
        if namespace.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "duplicate `namespace` in #[to_edn(...)] on enum",
            )
            .to_compile_error());
        }
        namespace = Some(parsed.0);
    }

    Ok(EnumAttr { namespace })
}

// ── Field attribute parser ────────────────────────────────────────────────────

/// Parse all `#[to_edn(...)]` attributes on a FIELD into a `FieldAttr`.
fn parse_field_attrs(field: &syn::Field) -> Result<FieldAttr, TokenStream2> {
    let mut key_override: Option<String> = None;
    let mut via_fn: Option<syn::Path> = None;
    let mut skip = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("to_edn") {
            continue;
        }
        let parsed = attr
            .parse_args_with(parse_field_directive)
            .map_err(|e| e.to_compile_error())?;

        match parsed {
            FieldDirective::Key(k) => {
                if key_override.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "duplicate `key` in #[to_edn(...)]",
                    )
                    .to_compile_error());
                }
                key_override = Some(k);
            }
            FieldDirective::Via(p) => {
                if via_fn.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "duplicate `via` in #[to_edn(...)]",
                    )
                    .to_compile_error());
                }
                via_fn = Some(p);
            }
            FieldDirective::Skip => {
                if skip {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "duplicate `skip` in #[to_edn(...)]",
                    )
                    .to_compile_error());
                }
                skip = true;
            }
        }
    }

    if skip && (key_override.is_some() || via_fn.is_some()) {
        return Err(syn::Error::new_spanned(
            field,
            "#[to_edn(skip)] cannot combine with `key` or `via` on the same field \
             (skip suppresses the field's plain pair entirely; a variant-level \
             `via(...)` owns the key instead)",
        )
        .to_compile_error());
    }

    Ok(FieldAttr {
        key_override,
        via_fn,
        skip,
    })
}

/// Discriminated output of one field-level `#[to_edn(...)]` parse.
enum FieldDirective {
    Key(String),
    Via(syn::Path),
    Skip,
}

/// Parse the token stream inside a field-level `#[to_edn(...)]`.
///
/// Allowed forms:
/// - `key = "string"` → `FieldDirective::Key`
/// - `via = bare_path` → `FieldDirective::Via`
///
/// Everything else is a `compile_error!`.
fn parse_field_directive(
    stream: syn::parse::ParseStream,
) -> syn::Result<FieldDirective> {
    let ident: syn::Ident = stream.parse().map_err(|e| {
        syn::Error::new(
            e.span(),
            "#[to_edn(...)] expects a directive name; \
             allowed field-level directives: key, via, skip",
        )
    })?;

    match ident.to_string().as_str() {
        "skip" => {
            // Bare word: `#[to_edn(skip)]`. No `=`, no args.
            if !stream.is_empty() {
                return Err(syn::Error::new(
                    stream.span(),
                    "#[to_edn(skip)] takes no value or arguments; \
                     write it bare (e.g. #[to_edn(skip)])",
                ));
            }
            Ok(FieldDirective::Skip)
        }

        "key" => {
            stream.parse::<syn::Token![=]>()?;
            // Value MUST be a string literal.
            if stream.peek(syn::LitInt) {
                let lit: syn::LitInt = stream.parse().unwrap();
                return Err(syn::Error::new_spanned(
                    lit,
                    "#[to_edn(key = ...)] value must be a string literal, not an integer \
                     (e.g. key = \"my-key\"); \
                     non-string values are forbidden by the grammar",
                ));
            }
            let lit: syn::LitStr = stream.parse().map_err(|e| {
                syn::Error::new(
                    e.span(),
                    "#[to_edn(key = ...)] value must be a string literal \
                     (e.g. key = \"my-key\")",
                )
            })?;
            Ok(FieldDirective::Key(lit.value()))
        }

        "via" => {
            if stream.peek(syn::token::Paren) {
                // They wrote via(...) on a field — that is a VARIANT-level form.
                return Err(syn::Error::new(
                    ident.span(),
                    "#[to_edn(via(...))] is a variant-level directive; \
                     on a field use #[to_edn(via = path::to::fn)]",
                ));
            }
            stream.parse::<syn::Token![=]>()?;
            // Value MUST be a bare path — reject string literal explicitly.
            if stream.peek(syn::LitStr) {
                let lit: syn::LitStr = stream.parse().unwrap();
                return Err(syn::Error::new_spanned(
                    lit,
                    "#[to_edn(via = ...)] value must be a bare path \
                     (e.g. via = my_fn or via = module::helper), not a string literal; \
                     allowed field-level directives: key, via, skip",
                ));
            }
            if stream.peek(syn::LitInt) {
                let lit: syn::LitInt = stream.parse().unwrap();
                return Err(syn::Error::new_spanned(
                    lit,
                    "#[to_edn(via = ...)] value must be a bare path \
                     (e.g. via = my_fn), not a literal; \
                     allowed field-level directives: key, via, skip",
                ));
            }
            let path: syn::Path = stream.parse().map_err(|e| {
                syn::Error::new(
                    e.span(),
                    "#[to_edn(via = ...)] value must be a bare path \
                     (e.g. via = my_fn or via = module::helper); \
                     inline expressions are forbidden",
                )
            })?;
            // Reject trailing tokens: via = xs.join(", ") parses `xs` as
            // a one-segment path and leaves `.join(", ")` here.
            if !stream.is_empty() {
                return Err(syn::Error::new(
                    stream.span(),
                    "#[to_edn(via = ...)] expects a bare path only; \
                     a method call or field access (e.g. xs.join(\", \")) \
                     is an inline expression and is forbidden; \
                     allowed field-level directives: key, via, skip",
                ));
            }
            Ok(FieldDirective::Via(path))
        }

        "literal" => {
            Err(syn::Error::new(
                ident.span(),
                "#[to_edn(literal(...))] is a variant-level directive (prepends synthetic \
                 constant fields); it cannot appear on a field; \
                 allowed field-level directives: key, via, skip",
            ))
        }

        other => Err(syn::Error::new(
            ident.span(),
            format!(
                "unknown #[to_edn(...)] directive `{}`; \
                 allowed field-level directives: key, via, skip",
                other
            ),
        )),
    }
}

// ── Variant attribute parser ──────────────────────────────────────────────────

/// Parse all `#[to_edn(...)]` attributes on a VARIANT into a `VariantAttr`.
fn parse_variant_attrs(variant: &syn::Variant) -> Result<VariantAttr, TokenStream2> {
    let mut literal_pairs: Vec<(String, String)> = Vec::new();
    let mut computed_via: Option<ComputedVia> = None;
    let mut key: Option<String> = None;

    for attr in &variant.attrs {
        if !attr.path().is_ident("to_edn") {
            continue;
        }
        let parsed = attr
            .parse_args_with(parse_variant_directive)
            .map_err(|e| e.to_compile_error())?;

        match parsed {
            VariantDirective::Literal(pairs) => {
                // Multiple literal attrs on the same variant merge in order.
                literal_pairs.extend(pairs);
            }
            VariantDirective::ComputedVia(cv) => {
                if computed_via.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "duplicate variant-level `via(...)` in #[to_edn(...)]",
                    )
                    .to_compile_error());
                }
                computed_via = Some(cv);
            }
            VariantDirective::Key(k) => {
                if key.is_some() {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "duplicate `key` in #[to_edn(...)] on variant",
                    )
                    .to_compile_error());
                }
                key = Some(k);
            }
        }
    }

    Ok(VariantAttr {
        literal_pairs,
        computed_via,
        key,
    })
}

/// Discriminated output of one variant-level `#[to_edn(...)]` parse.
enum VariantDirective {
    Literal(Vec<(String, String)>),
    ComputedVia(ComputedVia),
    /// `key = "…"`: EDN key for a single-field tuple variant's nameless field.
    Key(String),
}

/// Parse the token stream inside a variant-level `#[to_edn(...)]`.
///
/// Allowed forms:
/// - `literal(k = "v", …)` → `VariantDirective::Literal`
/// - `via(key = "k", fn = path, args(a, b, c))` → `VariantDirective::ComputedVia`
///
/// Everything else is a `compile_error!`.
fn parse_variant_directive(
    stream: syn::parse::ParseStream,
) -> syn::Result<VariantDirective> {
    let ident: syn::Ident = stream.parse().map_err(|e| {
        syn::Error::new(
            e.span(),
            "#[to_edn(...)] expects a directive name; \
             allowed variant-level directives: literal, via",
        )
    })?;

    match ident.to_string().as_str() {
        "literal" => {
            let content;
            syn::parenthesized!(content in stream);
            let pairs = parse_literal_pairs(&content)?;
            Ok(VariantDirective::Literal(pairs))
        }

        "via" => {
            // variant-level via(key = "k", fn = path, args(a, b, c))
            if stream.peek(syn::Token![=]) {
                return Err(syn::Error::new(
                    ident.span(),
                    "#[to_edn(via = ...)] is a field-level directive; \
                     on a variant use #[to_edn(via(key = \"k\", fn = path, args(a, b, c)))]",
                ));
            }
            let content;
            syn::parenthesized!(content in stream);
            let cv = parse_computed_via(&content)?;
            Ok(VariantDirective::ComputedVia(cv))
        }

        "key" => {
            // Variant-level `key = "…"`: names the single field of a tuple
            // variant. Illegal on Named/Unit variants (checked in the codegen).
            stream.parse::<syn::Token![=]>()?;
            if stream.peek(syn::LitInt) {
                let lit: syn::LitInt = stream.parse().unwrap();
                return Err(syn::Error::new_spanned(
                    lit,
                    "#[to_edn(key = ...)] value must be a string literal, not an integer \
                     (e.g. key = \"cause\"); \
                     allowed variant-level directives: literal, via, key",
                ));
            }
            let lit: syn::LitStr = stream.parse().map_err(|e| {
                syn::Error::new(
                    e.span(),
                    "#[to_edn(key = ...)] value must be a string literal \
                     (e.g. key = \"cause\"); \
                     on a variant this names the single field of a tuple variant; \
                     allowed variant-level directives: literal, via, key",
                )
            })?;
            Ok(VariantDirective::Key(lit.value()))
        }

        other => Err(syn::Error::new(
            ident.span(),
            format!(
                "unknown #[to_edn(...)] directive `{}`; \
                 allowed variant-level directives: literal, via, key",
                other
            ),
        )),
    }
}

/// Parse the body of `literal(k1 = "v1", k2 = "v2", …)`.
fn parse_literal_pairs(
    content: syn::parse::ParseStream,
) -> syn::Result<Vec<(String, String)>> {
    let mut pairs: Vec<(String, String)> = Vec::new();
    while !content.is_empty() {
        // Key is a bare ident (becomes the EDN keyword name after snake→kebab).
        let key_ident: syn::Ident = content.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                "expected a bare identifier as the literal key \
                 (e.g. literal(primitive = \":()\")); \
                 literal keys must be valid Rust identifiers",
            )
        })?;
        content.parse::<syn::Token![=]>()?;
        // Value MUST be a string literal.
        let val: syn::LitStr = content.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                "#[to_edn(literal(k = ...))] value must be a string literal \
                 (e.g. literal(primitive = \":()\")); \
                 non-string values are forbidden",
            )
        })?;
        pairs.push((snake_to_kebab(&key_ident.to_string()), val.value()));
        if content.peek(syn::Token![,]) {
            content.parse::<syn::Token![,]>()?;
        } else if !content.is_empty() {
            return Err(content.error("expected `,` between literal pairs"));
        }
    }
    Ok(pairs)
}

/// Parse the body of `via(key = "k", fn = path, args(a, b, c))`.
fn parse_computed_via(content: syn::parse::ParseStream) -> syn::Result<ComputedVia> {
    let mut key: Option<String> = None;
    let mut fn_path: Option<syn::Path> = None;
    let mut args: Option<Vec<syn::Ident>> = None;

    while !content.is_empty() {
        // `fn` is a keyword; check for it first.
        if content.peek(syn::Token![fn]) {
            content.parse::<syn::Token![fn]>()?;
            content.parse::<syn::Token![=]>()?;
            // Must be a bare path.
            if content.peek(syn::LitStr) {
                let lit: syn::LitStr = content.parse().unwrap();
                return Err(syn::Error::new_spanned(
                    lit,
                    "via(fn = ...) requires a bare path, not a string literal",
                ));
            }
            let p: syn::Path = content.parse().map_err(|e| {
                syn::Error::new(
                    e.span(),
                    "via(fn = ...) requires a bare path (e.g. fn = my_fn)",
                )
            })?;
            fn_path = Some(p);
        } else {
            let term_ident: syn::Ident = content.parse().map_err(|e| {
                syn::Error::new(
                    e.span(),
                    "expected a keyword in via(...); allowed: key, fn, args",
                )
            })?;

            match term_ident.to_string().as_str() {
                "key" => {
                    content.parse::<syn::Token![=]>()?;
                    let lit: syn::LitStr = content.parse().map_err(|e| {
                        syn::Error::new(
                            e.span(),
                            "via(key = ...) requires a string literal (e.g. key = \"hints\")",
                        )
                    })?;
                    key = Some(lit.value());
                }
                "args" => {
                    let args_content;
                    syn::parenthesized!(args_content in content);
                    let punctuated = args_content.parse_terminated(
                        syn::Ident::parse,
                        syn::Token![,],
                    )?;
                    args = Some(punctuated.into_iter().collect());
                }
                other => {
                    return Err(syn::Error::new(
                        term_ident.span(),
                        format!(
                            "unknown key `{}` in #[to_edn(via(...))]; allowed: key, fn, args",
                            other
                        ),
                    ));
                }
            }
        }

        if content.peek(syn::Token![,]) {
            content.parse::<syn::Token![,]>()?;
        } else if !content.is_empty() {
            return Err(content.error("expected `,` between via(...) args"));
        }
    }

    let key = key.ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[to_edn(via(...))] requires `key = \"...\"` (the EDN key for the computed field)",
        )
    })?;
    let fn_path = fn_path.ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[to_edn(via(...))] requires `fn = path` (the path to the compute function)",
        )
    })?;
    let args = args.unwrap_or_default();

    Ok(ComputedVia { key, fn_path, args })
}

// ── Code generator ────────────────────────────────────────────────────────────

/// Inner implementation called by the `#[proc_macro_derive(ToEdn)]` entry point.
///
/// Returns a `TokenStream2` that is the `impl ::wat_edn::ToEdn for <Type>` block,
/// or a `compile_error!` when the input is not a supported shape.
///
/// Generated code uses absolute paths (`::wat_edn::ToEdn`, `::wat_edn::OwnedValue`,
/// etc.) so the impl resolves correctly in any consumer crate, not just in `wat`.
fn derive_to_edn_inner(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    // ── Parse enum-level #[to_edn(...)] attrs ───────────────────────────────
    let enum_attr = match parse_enum_attrs(&input) {
        Ok(a) => a,
        Err(e) => return e,
    };
    // `namespace_tokens` is the token tree emitted as the first arg of
    // `::wat_edn::Tag::ns(#namespace_tokens, #variant_name_str)`.
    // If `#[to_edn(namespace = <path>)]` is present, the path is emitted as-is
    // (a reference to a `&str` const, never a baked literal).
    // If absent: back-compat default — the literal `"wat.kernel"`.
    let namespace_tokens: TokenStream2 = match enum_attr.namespace {
        Some(path) => quote! { #path },
        None => quote! { "wat.kernel" },
    };

    // ── Only enums and named-field structs are supported ────────────────────
    let data_enum = match &input.data {
        Data::Enum(e) => e,
        // ── STRUCT: a struct → ONE tagged record #<ns>/<Name> {fields}. ──
        Data::Struct(data_struct) => {
            let named = match &data_struct.fields {
                Fields::Named(f) => &f.named,
                _ => {
                    return syn::Error::new_spanned(
                        name,
                        "ToEdn struct derive supports named-field structs only",
                    )
                    .to_compile_error();
                }
            };
            let name_str = name.to_string();
            let mut field_pushes: Vec<TokenStream2> = Vec::new();
            for f in named {
                let fid = f.ident.as_ref().expect("named field has ident");
                let edn_key = snake_to_kebab(&fid.to_string());
                field_pushes.push(quote! {
                    __fields.push((
                        ::wat_edn::OwnedValue::Keyword(::wat_edn::Keyword::new(#edn_key)),
                        ::wat_edn::ToEdn::to_edn(&self.#fid),
                    ));
                });
            }
            return quote! {
                impl ::wat_edn::ToEdn for #name {
                    fn to_edn(&self) -> ::wat_edn::OwnedValue {
                        let mut __fields: ::std::vec::Vec<(
                            ::wat_edn::OwnedValue,
                            ::wat_edn::OwnedValue,
                        )> = ::std::vec::Vec::new();
                        #(#field_pushes)*
                        ::wat_edn::OwnedValue::Tagged(
                            ::wat_edn::Tag::ns(#namespace_tokens, #name_str),
                            ::std::boxed::Box::new(::wat_edn::OwnedValue::Map(__fields))
                        )
                    }
                }
            };
        }
        _ => {
            return syn::Error::new_spanned(
                name,
                "ToEdn derive supports enums (kind-enums) only in Strike 1",
            )
            .to_compile_error();
        }
    };

    // ── Build one match arm per variant ─────────────────────────────────────
    let mut arms: Vec<TokenStream2> = Vec::new();

    for variant in &data_enum.variants {
        let variant_ident = &variant.ident;
        let variant_name_str = variant_ident.to_string();

        // ── Parse variant-level #[to_edn(...)] attrs ──────────────────────
        let variant_attr = match parse_variant_attrs(variant) {
            Ok(a) => a,
            Err(e) => return e,
        };

        match &variant.fields {
            // ── Struct variant: { a_b: T, c: U } ─────────────────────────
            Fields::Named(named_fields) => {
                // Guard: variant-level `key` is only for tuple variants.
                if let Some(ref k) = variant_attr.key {
                    return syn::Error::new_spanned(
                        variant,
                        format!(
                            "`#[to_edn(key = {:?})]` on a struct variant is invalid; \
                             use field-level `#[to_edn(key = \"…\")]` to rename a named field; \
                             variant-level `key` is only valid on a single-field tuple variant",
                            k
                        ),
                    )
                    .to_compile_error();
                }

                // Collect all field idents and their types.
                let fields_info: Vec<(&syn::Ident, &syn::Type)> = named_fields
                    .named
                    .iter()
                    .map(|f| {
                        (
                            f.ident.as_ref().expect("named field has ident"),
                            &f.ty,
                        )
                    })
                    .collect();

                let field_idents: Vec<&syn::Ident> =
                    fields_info.iter().map(|(i, _)| *i).collect();

                // Build per-field emit tokens.
                let mut field_pushes: Vec<TokenStream2> = Vec::new();

                for (field_ident, _field_ty) in &fields_info {
                    // Find the field's syn::Field to get attrs.
                    let syn_field = named_fields
                        .named
                        .iter()
                        .find(|f| f.ident.as_ref() == Some(field_ident))
                        .expect("field must exist");

                    let field_attr = match parse_field_attrs(syn_field) {
                        Ok(a) => a,
                        Err(e) => return e,
                    };

                    let edn_key = field_attr
                        .key_override
                        .unwrap_or_else(|| snake_to_kebab(&field_ident.to_string()));

                    if field_attr.skip {
                        // Skipped field: bound in the match arm (still available as
                        // a variant-level `via` arg) but emits NO plain pair.
                        // Continue without pushing.
                    } else if let Some(via_path) = field_attr.via_fn {
                        // via-overridden field: call the helper fn.
                        field_pushes.push(quote! {
                            __fields.push((
                                ::wat_edn::OwnedValue::Keyword(
                                    ::wat_edn::Keyword::new(#edn_key)
                                ),
                                #via_path(#field_ident),
                            ));
                        });
                    } else {
                        // Normal field: call .to_edn().
                        field_pushes.push(quote! {
                            __fields.push((
                                ::wat_edn::OwnedValue::Keyword(
                                    ::wat_edn::Keyword::new(#edn_key)
                                ),
                                #field_ident.to_edn(),
                            ));
                        });
                    }
                }

                // Literal pairs to prepend.
                let literal_pushes: Vec<TokenStream2> = variant_attr
                    .literal_pairs
                    .iter()
                    .map(|(k, v)| {
                        quote! {
                            __fields.push((
                                ::wat_edn::OwnedValue::Keyword(
                                    ::wat_edn::Keyword::new(#k)
                                ),
                                ::wat_edn::OwnedValue::String(
                                    ::std::borrow::Cow::Owned(#v.to_owned())
                                ),
                            ));
                        }
                    })
                    .collect();

                // Computed via to append.
                let computed_via_push: Option<TokenStream2> =
                    variant_attr.computed_via.map(|cv| {
                        let via_key = cv.key;
                        let via_fn = cv.fn_path;
                        let via_args = cv.args;
                        quote! {
                            if let ::std::option::Option::Some(__via_val) =
                                #via_fn(#(#via_args),*)
                            {
                                __fields.push((
                                    ::wat_edn::OwnedValue::Keyword(
                                        ::wat_edn::Keyword::new(#via_key)
                                    ),
                                    ::wat_edn::ToEdn::to_edn(&__via_val),
                                ));
                            }
                        }
                    });

                arms.push(quote! {
                    Self::#variant_ident { #(#field_idents,)* } => {
                        #[allow(unused_imports)]
                        use ::wat_edn::ToEdn as _ToEdnTrait;
                        let mut __fields: ::std::vec::Vec<(
                            ::wat_edn::OwnedValue,
                            ::wat_edn::OwnedValue,
                        )> = ::std::vec::Vec::new();
                        // 1. Prepend literal synthetic pairs.
                        #(#literal_pushes)*
                        // 2. Emit field pairs (in declaration order).
                        #(#field_pushes)*
                        // 3. Append computed via field (elide on None).
                        #computed_via_push
                        ::wat_edn::OwnedValue::Tagged(
                            ::wat_edn::Tag::ns(#namespace_tokens, #variant_name_str),
                            ::std::boxed::Box::new(::wat_edn::OwnedValue::Map(__fields))
                        )
                    }
                });
            }

            // ── Unit variant: Bar ─────────────────────────────────────────
            Fields::Unit => {
                // Guard: variant-level `key` is only for tuple variants.
                if let Some(ref k) = variant_attr.key {
                    return syn::Error::new_spanned(
                        variant,
                        format!(
                            "`#[to_edn(key = {:?})]` on a unit variant is invalid; \
                             unit variants have no fields; \
                             variant-level `key` is only valid on a single-field tuple variant",
                            k
                        ),
                    )
                    .to_compile_error();
                }

                // Literal pairs to prepend.
                let literal_pushes: Vec<TokenStream2> = variant_attr
                    .literal_pairs
                    .iter()
                    .map(|(k, v)| {
                        quote! {
                            __fields.push((
                                ::wat_edn::OwnedValue::Keyword(
                                    ::wat_edn::Keyword::new(#k)
                                ),
                                ::wat_edn::OwnedValue::String(
                                    ::std::borrow::Cow::Owned(#v.to_owned())
                                ),
                            ));
                        }
                    })
                    .collect();

                // Computed via on a unit variant (no field args available —
                // args() must be empty).
                let computed_via_push: Option<TokenStream2> =
                    variant_attr.computed_via.map(|cv| {
                        let via_key = cv.key;
                        let via_fn = cv.fn_path;
                        let via_args = cv.args;
                        quote! {
                            if let ::std::option::Option::Some(__via_val) =
                                #via_fn(#(#via_args),*)
                            {
                                __fields.push((
                                    ::wat_edn::OwnedValue::Keyword(
                                        ::wat_edn::Keyword::new(#via_key)
                                    ),
                                    ::wat_edn::ToEdn::to_edn(&__via_val),
                                ));
                            }
                        }
                    });

                if literal_pushes.is_empty() && computed_via_push.is_none() {
                    // Fast path: no attrs, no mutable Vec needed.
                    arms.push(quote! {
                        Self::#variant_ident => {
                            ::wat_edn::OwnedValue::Tagged(
                                ::wat_edn::Tag::ns(#namespace_tokens, #variant_name_str),
                                ::std::boxed::Box::new(
                                    ::wat_edn::OwnedValue::Map(::std::vec::Vec::new())
                                )
                            )
                        }
                    });
                } else {
                    arms.push(quote! {
                        Self::#variant_ident => {
                            let mut __fields: ::std::vec::Vec<(
                                ::wat_edn::OwnedValue,
                                ::wat_edn::OwnedValue,
                            )> = ::std::vec::Vec::new();
                            #(#literal_pushes)*
                            #computed_via_push
                            ::wat_edn::OwnedValue::Tagged(
                                ::wat_edn::Tag::ns(#namespace_tokens, #variant_name_str),
                                ::std::boxed::Box::new(::wat_edn::OwnedValue::Map(__fields))
                            )
                        }
                    });
                }
            }

            // ── Tuple variant: Foo(T) — single-field only ─────────────────
            Fields::Unnamed(f) => {
                if f.unnamed.len() == 1 {
                    // Single-field tuple variant. Requires a variant-level
                    // `#[to_edn(key = "…")]` to name the EDN key.
                    let edn_key = match variant_attr.key {
                        Some(k) => k,
                        None => {
                            return syn::Error::new_spanned(
                                variant,
                                "single-field tuple variant requires \
                                 `#[to_edn(key = \"…\")]` to name the EDN key \
                                 (the field has no Rust ident; the key must be \
                                 declared explicitly)",
                            )
                            .to_compile_error();
                        }
                    };

                    // The single unnamed field may carry field-level attrs
                    // (e.g. `via = path`) for a custom transform.
                    let field = f.unnamed.iter().next().expect("len == 1");
                    let field_attr = match parse_field_attrs(field) {
                        Ok(a) => a,
                        Err(e) => return e,
                    };
                    if field_attr.skip {
                        return syn::Error::new_spanned(
                            variant,
                            "#[to_edn(skip)] is not valid on a tuple-variant field; \
                             the field's key is named by the variant-level \
                             `#[to_edn(key = \"…\")]`",
                        )
                        .to_compile_error();
                    }
                    // field_attr.key_override would conflict with the
                    // variant-level key; disallow it.
                    if field_attr.key_override.is_some() {
                        return syn::Error::new_spanned(
                            variant,
                            "#[to_edn(key = ...)] on the tuple field conflicts with \
                             the variant-level #[to_edn(key = ...)]; \
                             put the key annotation on the variant, not the field",
                        )
                        .to_compile_error();
                    }

                    let value_expr: TokenStream2 = if let Some(via_path) = field_attr.via_fn {
                        quote! { #via_path(__0) }
                    } else {
                        quote! { __0.to_edn() }
                    };

                    arms.push(quote! {
                        Self::#variant_ident(__0) => {
                            #[allow(unused_imports)]
                            use ::wat_edn::ToEdn as _ToEdnTrait;
                            ::wat_edn::OwnedValue::Tagged(
                                ::wat_edn::Tag::ns(#namespace_tokens, #variant_name_str),
                                ::std::boxed::Box::new(::wat_edn::OwnedValue::Map(
                                    ::std::vec![
                                        (
                                            ::wat_edn::OwnedValue::Keyword(
                                                ::wat_edn::Keyword::new(#edn_key)
                                            ),
                                            #value_expr,
                                        )
                                    ]
                                ))
                            )
                        }
                    });
                } else {
                    // Multi-field tuple: no safe key assignment → compile_error.
                    return syn::Error::new_spanned(
                        variant,
                        "ToEdn derive supports single-field tuple variants only; \
                         multi-field tuple variants have no safe key assignment \
                         (which positional field gets which EDN key?); \
                         convert to a struct variant with named fields instead",
                    )
                    .to_compile_error();
                }
            }
        }
    }

    // ── Emit the impl ────────────────────────────────────────────────────────
    //
    // Generated code uses absolute paths (`::wat_edn::ToEdn`) so the impl
    // resolves correctly in any consumer crate, not just in `wat`.
    // `use ::wat_edn::ToEdn as _ToEdnTrait` inside the match body brings the
    // trait into scope for method call resolution (`field.to_edn()`).
    quote! {
        impl ::wat_edn::ToEdn for #name {
            fn to_edn(&self) -> ::wat_edn::OwnedValue {
                #[allow(unused_imports)]
                use ::wat_edn::ToEdn as _ToEdnTrait;
                match self {
                    #(#arms,)*
                }
            }
        }
    }
}
