//! `#[wat_value]` — structural seal for the Value enum.
//!
//! Forbids wrapping-style variants (single `Box<Self>` / `Arc<Self>` /
//! `Rc<Self>` / `Self` field) at compile time. Future authors who try to
//! re-introduce the trap-door class encounter a `compile_error!` with a
//! teaching diagnostic.
//!
//! ## Detection rule (Decision 1 — syntactic scan)
//!
//! **Forbidden field types on any variant (unless opt-in):**
//! - `Self` or the enum's own name directly (e.g., `Wrap(Value)`)
//! - `Box<Self>`, `Arc<Self>`, `Rc<Self>` (single smart-pointer-of-Self)
//! - Nested: `Box<Box<Self>>`, `Arc<Box<Self>>`, etc.
//!
//! **Allowed field types:**
//! - Primitive: `i64`, `bool`, `String`, `Arc<String>`, etc.
//! - Collection: `Vec<Value>`, `Arc<Vec<Value>>`, `HashMap<K,Value>`, etc.
//! - Sum-type containers: `Option<Value>`, `Arc<Option<Value>>`,
//!   `Result<Value,Value>`, `Arc<Result<Value,Value>>`
//! - Anything whose outermost type constructor is NOT a smart-pointer-of-Self
//!
//! ## Opt-in escape hatch (Decision 2 — per-variant only)
//!
//! ```rust,ignore
//! #[wat_value(allow_wrapping = "your reason")]
//! MyVariant { inner: Box<MyEnum> },
//! ```
//!
//! The reason string is mandatory and non-empty. It becomes part of the
//! source record — reviewers see WHY the structural exception was allowed.
//!
//! ## Type-alias limitation (Decision 1)
//!
//! The detection is purely syntactic. If you alias a forbidden type
//! (`type BoxedValue = Box<Value>`) and use the alias in a field, the
//! macro will NOT reject it. Use `#[wat_value(allow_wrapping = "...")]`
//! explicitly if you intentionally need that shape; or document that the
//! alias exists and why it's safe. This is a known limitation per
//! sub-DESIGN Decision 1; semantic resolution would require `rustc`
//! internals out of scope here.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Attribute, Error, Ident, ItemEnum, LitStr, Type};

/// `#[wat_value]` — structural seal for enum definitions.
///
/// Rejects variants with wrapping shape (single `Box<Self>` / `Arc<Self>` /
/// `Rc<Self>` / `Self` field) at compile time, unless the variant is
/// explicitly opted-in via `#[wat_value(allow_wrapping = "reason")]`.
///
/// See module docs for detection rule, escape hatch, and known limitations.
pub fn wat_value(args: TokenStream, input: TokenStream) -> TokenStream {
    // The macro-level args are reserved for future use. Currently only
    // per-variant #[wat_value(allow_wrapping = "...")] is supported.
    // Reject any macro-level args to prevent confusion.
    if !args.is_empty() {
        let ts2 = TokenStream2::from(args);
        return Error::new_spanned(
            ts2,
            "#[wat_value] takes no arguments at the enum level. \
             Per-variant opt-in uses #[wat_value(allow_wrapping = \"reason\")] \
             on the specific variant. \
             Enum-level escape hatch is intentionally forbidden — it would defeat \
             the structural seal. See DESIGN-STONE-233.2.l.md Decision 2.",
        )
        .to_compile_error()
        .into();
    }

    let mut item_enum = parse_macro_input!(input as ItemEnum);
    let enum_name = item_enum.ident.clone();

    let mut errors: Vec<TokenStream2> = Vec::new();

    for variant in &mut item_enum.variants {
        // Check for per-variant #[wat_value(allow_wrapping = "...")] opt-in.
        // If present with a non-empty reason, skip the field-type check for
        // this variant. Strip the attribute from the output (it's documentation
        // in source, not a Rust derive/proc-macro that downstream crates need).
        if let Some(reason) = extract_allow_wrapping_reason(&variant.attrs) {
            // Validate: reason must be non-empty.
            if reason.trim().is_empty() {
                errors.push(
                    Error::new_spanned(
                        &variant.ident,
                        format!(
                            "#[wat_value(allow_wrapping = \"...\")] on variant `{}`: \
                             the reason string must be non-empty. \
                             The reason is your ceremonial documentation of WHY this \
                             structural exception is justified. \
                             See DESIGN-STONE-233.2.l.md Decision 2.",
                            variant.ident
                        ),
                    )
                    .to_compile_error(),
                );
            }
            // Strip the #[wat_value(...)] attribute regardless (even if reason
            // is empty — we emitted the error above; stripping avoids cascading
            // "unknown attribute" errors from the compiler downstream).
            variant.attrs.retain(|attr| !is_wat_value_attr(attr));
            // Opt-in accepted (or error already emitted) — skip field check.
            continue;
        }

        // No opt-in: check every field for the forbidden pattern.
        // Collect the variant ident as a string before any potential move.
        let variant_name = variant.ident.to_string();
        let forbidden_field = variant
            .fields
            .iter()
            .find(|f| is_forbidden_field_type(&f.ty, &enum_name));
        if forbidden_field.is_some() {
            errors.push(
                Error::new_spanned(
                    &*variant,
                    format!(
                        "#[wat_value]: variant `{}` has wrapping shape \
                         (single Box<Self> / Arc<Self> / Rc<Self> / Self field)\n\
                         \n\
                         Wrapping variants are forbidden because they silently mis-dispatch \
                         pattern-match on Value::X(...): the inner Value::X gets shadowed. \
                         This is the trap-door class arc 233 eliminated (see Stone 233.2.f \
                         apply fix; Stone 233.2.j cascade; Stone 233.2.k variant retirement).\n\
                         \n\
                         If your use case GENUINELY requires wrapping, add\n    \
                             #[wat_value(allow_wrapping = \"your reason\")]\n\
                         to this variant. The reason string is mandatory and non-empty; \
                         it documents WHY the structural exception is justified.\n\
                         \n\
                         More often the right fix is a SIBLING TYPE outside the enum \
                         (e.g., wat::runtime::TrackedValue per Stone 233.2.h). \
                         See docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.l.md \
                         for the doctrine.",
                        variant_name
                    ),
                )
                .to_compile_error(),
            );
        }
    }

    if !errors.is_empty() {
        // Emit all errors. The item_enum is also emitted (with #[wat_value]
        // attrs stripped) so the compiler can continue parsing and surface
        // secondary errors. This gives multi-variant feedback in one pass.
        let item_ts: TokenStream2 = quote! { #item_enum };
        let all_errors: TokenStream2 = errors.into_iter().collect();
        return quote! {
            #all_errors
            #item_ts
        }
        .into();
    }

    // All variants passed — emit the item unchanged (with #[wat_value] attrs
    // already stripped above for opt-in variants).
    quote! { #item_enum }.into()
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Returns `Some(reason)` if the variant has a
/// `#[wat_value(allow_wrapping = "reason")]` attribute, `None` otherwise.
fn extract_allow_wrapping_reason(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !is_wat_value_attr(attr) {
            continue;
        }
        // Parse the attribute's argument list. We expect exactly one
        // `allow_wrapping = "..."` key-value pair.
        let result: syn::Result<AllowWrappingArg> = attr.parse_args();
        match result {
            Ok(arg) => return Some(arg.reason),
            Err(_) => {
                // The attribute is `#[wat_value(...)]` but doesn't match the
                // expected form. We'll let the outer logic emit a better error
                // or let Rust surface a parse failure. Return None so the
                // field-type check still runs (which may emit its own error).
                return None;
            }
        }
    }
    None
}

/// Checks whether `attr` is the `#[wat_value(...)]` attribute.
fn is_wat_value_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("wat_value")
}

/// Parsed `allow_wrapping = "reason"` inside `#[wat_value(...)]`.
struct AllowWrappingArg {
    reason: String,
}

impl syn::parse::Parse for AllowWrappingArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if key != "allow_wrapping" {
            return Err(Error::new_spanned(
                key,
                "expected `allow_wrapping = \"...\"`; \
                 this is the only supported per-variant argument for #[wat_value(...)]",
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let reason_lit: LitStr = input.parse()?;
        Ok(AllowWrappingArg {
            reason: reason_lit.value(),
        })
    }
}

/// Returns `true` if `ty` is a forbidden wrapping type:
/// - `Self` or `EnumName` directly
/// - `Box<T>`, `Arc<T>`, `Rc<T>` where T is itself forbidden (recursive)
///
/// **Allowed:** container types (`Vec<Self>`, `Option<Self>`, `Result<Self,_>`,
/// `HashMap<K,Self>`, etc.) — their outermost constructor is NOT a smart-pointer.
fn is_forbidden_field_type(ty: &Type, enum_name: &Ident) -> bool {
    match ty {
        Type::Path(type_path) => {
            // Ignore any leading `self::` / `crate::` / etc. qualifiers —
            // we only care about the last-path-segment shape.
            let segments = &type_path.path.segments;
            if segments.len() != 1 {
                // Multi-segment path (e.g., `std::boxed::Box<...>`) — not the
                // short form we detect. Known limitation per Decision 1: the
                // syntactic scan only catches the single-segment form.
                return false;
            }
            let seg = &segments[0];
            let seg_name = seg.ident.to_string();

            // Direct Self or enum-own-name reference.
            if seg.ident == "Self" || seg.ident == *enum_name {
                return true;
            }

            // Smart-pointer: Box, Arc, Rc — check the inner type recursively.
            if matches!(seg_name.as_str(), "Box" | "Arc" | "Rc") {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            if is_forbidden_field_type(inner_ty, enum_name) {
                                return true;
                            }
                        }
                    }
                }
                // Box/Arc/Rc with no args or with non-Self inner — allowed.
                // (e.g., `Arc<String>` is fine; `Arc<Vec<Value>>` is fine because
                // Vec's outer constructor is checked next round and returns false.)
                return false;
            }

            false
        }
        // Reference types `&T` / `&mut T` — follow the referee.
        Type::Reference(type_ref) => is_forbidden_field_type(&type_ref.elem, enum_name),
        // All other type forms (tuples, slices, bare fn, etc.) are allowed.
        _ => false,
    }
}
