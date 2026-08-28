//! Codegen for `#[wat_special_form_impl("<fqdn>", role = check|eval|tail)]` — arc 255 Stone
//! P6-a.
//!
//! `#[wat_special_form]` (the sibling in `wat_special_form.rs`) annotates a doc-only unit
//! struct — a proc-macro sees only the tokens of the item it decorates, so that struct's
//! attribute can never reach across files to capture `eval_if`'s or `infer_if`'s body. This
//! macro goes on each of a special form's REAL implementations instead, exactly the way
//! `#[wat_intrinsic]` captures a handler: `quote!(#item).to_string()` into a `source` field
//! (`wat_intrinsic.rs:565`), the fn passed through completely unchanged, an
//! `inventory::submit!` recording the (fqdn, role) key.
//!
//! Three implementations submit under the SAME fqdn with different roles; `registry()` gathers
//! them into the `IntrinsicEntry::impls` Vec and `show-source` prints all three, labelled.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Error, Ident, ItemFn, LitStr, Token};

/// The parsed `#[wat_special_form_impl(<fqdn>, role = <role>)]` attribute payload.
pub(crate) struct WatSpecialFormImplAttr {
    pub(crate) fqdn: LitStr,
    pub(crate) role: Ident,
}

impl syn::parse::Parse for WatSpecialFormImplAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fqdn: LitStr = input.parse()?;
        input.parse::<Token![,]>().map_err(|_| {
            Error::new(
                input.span(),
                "wat_special_form_impl: expected `, role = check|eval|tail` after the fqdn",
            )
        })?;
        let key: Ident = input.parse()?;
        if key != "role" {
            return Err(Error::new_spanned(
                &key,
                "wat_special_form_impl: expected `role = check|eval|tail` as the only argument \
                 after the fqdn",
            ));
        }
        input.parse::<Token![=]>()?;
        let role: Ident = input.parse()?;
        Ok(WatSpecialFormImplAttr { fqdn, role })
    }
}

/// Map the bare `check` / `eval` / `tail` identifier to the `SpecialFormRole` variant path.
/// Any other identifier is a `compile_error!`, not a silent default — a fourth role or a typo
/// must be visible at compile time, not at `registry()`-build time.
fn role_variant(role: &Ident) -> syn::Result<TokenStream2> {
    match role.to_string().as_str() {
        "check" => Ok(quote! { ::wat::intrinsic::SpecialFormRole::Check }),
        "eval" => Ok(quote! { ::wat::intrinsic::SpecialFormRole::Eval }),
        "tail" => Ok(quote! { ::wat::intrinsic::SpecialFormRole::Tail }),
        other => Err(Error::new_spanned(
            role,
            format!(
                "wat_special_form_impl: unknown role `{}`; expected one of: check, eval, tail",
                other
            ),
        )),
    }
}

pub(crate) fn emit(attr: &WatSpecialFormImplAttr, item: &ItemFn) -> syn::Result<TokenStream2> {
    let fqdn = &attr.fqdn;
    let role_token = role_variant(&attr.role)?;

    // Arc 255.1b-v's mechanism, reused verbatim: capture the ANNOTATED fn's own source via
    // stable restringify. `#item` below passes it through completely unchanged — this macro
    // adds a submission, it does not reroute a call (STOP-2).
    let source_lit = quote!(#item).to_string();

    let expanded = quote! {
        // The annotated implementation, passed through unchanged.
        #item

        // Auto-collect: link-time registration of this (fqdn, role) implementation. Gathered
        // by `registry()` and folded into the matching `Kind::SpecialForm` entry's `impls`.
        ::inventory::submit! {
            ::wat::intrinsic::SpecialFormImplSubmission {
                name: #fqdn,
                role: #role_token,
                source: #source_lit,
            }
        }
    };

    Ok(expanded)
}
