//! `#[derive(ToEdn)]` — structural body generator for kind-enums.
//!
//! Generates `impl crate::to_edn::ToEdn for <KindEnum>` from the Rust type,
//! so there is no hand-written match body to smuggle prose into. An embedded
//! field whose type is not `ToEdn` is a compile error — the structural wall
//! arc 296 Strike 1 promises.
//!
//! ## Supported shapes (Strike 1)
//!
//! - **Struct variant** `Foo { a_b: T, c: U }` →
//!   `#wat.kernel/Foo {:a-b <a_b.to_edn()> :c <c.to_edn()>}`.
//!   Snake-case field idents map to kebab-case EDN keys. Declaration order
//!   is preserved.
//! - **Unit variant** `Bar` →
//!   `#wat.kernel/Bar {}` (empty map body).
//! - **Tuple variants** — STOP: unsupported in Strike 1; the derive emits a
//!   `compile_error!` if any tuple variant is encountered.
//!
//! ## The helper attribute `#[to_edn(...)]`
//!
//! Declared so the compiler does not reject `#[to_edn(...)]` on fields, but
//! Strike 1 **ignores** all such annotations. Strike 2 wires them up.
//!
//! ## Namespace
//!
//! All variant tags use `wat.kernel` as the namespace, matching the existing
//! hand-written serializers.
//!
//! ## Wall
//!
//! Every field value is serialized via `field.to_edn()`. If a field type does
//! not implement `crate::to_edn::ToEdn`, rustc emits:
//!
//! ```text
//! error[E0277]: the trait bound `T: ToEdn` is not satisfied
//! ```
//!
//! This is the "floorless BODY is unrepresentable" guarantee: the mistake
//! cannot be expressed in Rust — it is a compile error, not a runtime failure.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// Convert a snake_case identifier string to kebab-case EDN key.
///
/// `setter_head` → `"setter-head"`, `field` → `"field"`, `a_b` → `"a-b"`.
fn snake_to_kebab(s: &str) -> String {
    s.replace('_', "-")
}

/// Entry point called by the `#[proc_macro_derive(ToEdn)]` shim in `lib.rs`.
///
/// Returns a `TokenStream2` that is the `impl crate::to_edn::ToEdn for
/// <Enum>` block, or a `compile_error!` when the input is not a supported
/// shape.
pub fn derive_to_edn(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    // ── Only enums are supported in Strike 1 ────────────────────────────────
    let data_enum = match &input.data {
        Data::Enum(e) => e,
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

        match &variant.fields {
            // ── Struct variant: { a_b: T, c: U } ─────────────────────────
            Fields::Named(named_fields) => {
                // Collect field idents (skip any #[to_edn(...)] attribute —
                // Strike 1 ignores helper attrs, Strike 2 wires them up).
                let field_idents: Vec<&syn::Ident> = named_fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().expect("named field has ident"))
                    .collect();

                let field_keys: Vec<String> = field_idents
                    .iter()
                    .map(|i| snake_to_kebab(&i.to_string()))
                    .collect();

                arms.push(quote! {
                    Self::#variant_ident { #(#field_idents,)* } => {
                        ::wat_edn::OwnedValue::Tagged(
                            ::wat_edn::Tag::ns("wat.kernel", #variant_name_str),
                            ::std::boxed::Box::new(::wat_edn::OwnedValue::Map(vec![
                                #(
                                    (
                                        ::wat_edn::OwnedValue::Keyword(
                                            ::wat_edn::Keyword::new(#field_keys)
                                        ),
                                        #field_idents.to_edn(),
                                    ),
                                )*
                            ]))
                        )
                    }
                });
            }

            // ── Unit variant: Bar ─────────────────────────────────────────
            Fields::Unit => {
                arms.push(quote! {
                    Self::#variant_ident => {
                        ::wat_edn::OwnedValue::Tagged(
                            ::wat_edn::Tag::ns("wat.kernel", #variant_name_str),
                            ::std::boxed::Box::new(::wat_edn::OwnedValue::Map(vec![]))
                        )
                    }
                });
            }

            // ── Tuple variant: NOT supported in Strike 1 ──────────────────
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(
                    variant,
                    "ToEdn derive does not support tuple variants in Strike 1; \
                     report this to the arc 296 author for Strike 2",
                )
                .to_compile_error();
            }
        }
    }

    // ── Emit the impl ────────────────────────────────────────────────────────
    //
    // `use crate::to_edn::ToEdn as _ToEdnTrait` brings the trait into scope
    // for method resolution so `field_ident.to_edn()` resolves correctly.
    // The impl expands inside the `wat` crate, so `crate::to_edn::ToEdn` is
    // the canonical path (no external-crate indirection needed).
    quote! {
        impl crate::to_edn::ToEdn for #name {
            fn to_edn(&self) -> ::wat_edn::OwnedValue {
                #[allow(unused_imports)]
                use crate::to_edn::ToEdn as _ToEdnTrait;
                match self {
                    #(#arms,)*
                }
            }
        }
    }
}
