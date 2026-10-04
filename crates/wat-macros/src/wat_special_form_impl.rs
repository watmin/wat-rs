//! Codegen for `#[wat_special_form_impl("<fqdn>", role = check|eval|tail|declare)]` — arc 255
//! Stone P6-a (`declare` added by Stone 1a-β-0).
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
use quote::{format_ident, quote};
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
                "wat_special_form_impl: expected `, role = check|eval|tail|declare` after the fqdn",
            )
        })?;
        let key: Ident = input.parse()?;
        if key != "role" {
            return Err(Error::new_spanned(
                &key,
                "wat_special_form_impl: expected `role = check|eval|tail|declare` as the only \
                 argument after the fqdn",
            ));
        }
        input.parse::<Token![=]>()?;
        let role: Ident = input.parse()?;
        Ok(WatSpecialFormImplAttr { fqdn, role })
    }
}

/// Map the bare `check` / `eval` / `tail` / `declare` identifier to the `SpecialFormRole`
/// variant path. Any other identifier is a `compile_error!`, not a silent default — an
/// unrecognized role or a typo must be visible at compile time, not at `registry()`-build time.
fn role_variant(role: &Ident) -> syn::Result<TokenStream2> {
    match role.to_string().as_str() {
        "check" => Ok(quote! { ::wat::intrinsic::SpecialFormRole::Check }),
        "eval" => Ok(quote! { ::wat::intrinsic::SpecialFormRole::Eval }),
        "tail" => Ok(quote! { ::wat::intrinsic::SpecialFormRole::Tail }),
        "declare" => Ok(quote! { ::wat::intrinsic::SpecialFormRole::Declare }),
        other => Err(Error::new_spanned(
            role,
            format!(
                "wat_special_form_impl: unknown role `{}`; expected one of: check, eval, tail, declare",
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

    // arc 255 Stone the-eval-door — `role = eval` ALSO emits a callable pointer, so the
    // registry's `handler` slot (not a new field, STOP into a second door) can dispatch this
    // form directly. `role = check` keeps emitting source only — a check impl runs once,
    // statically, and has no per-invocation call site to dispatch through. Excursus 003 strike
    // G item 4: the Value-vs-TrackedValue sniff/wrap this call once needed is gone — every
    // handler returns bare `Value` now, so the annotated fn's call is forwarded unwrapped.
    let (eval_shim_tokens, eval_handler_field) = if attr.role.to_string().as_str() == "eval" {
        let fn_ident = &item.sig.ident;
        let shim_ident = format_ident!("__wat_special_form_eval_{}", fn_ident);
        let call = quote! { #fn_ident(args, list_span, env, sym) };
        let shim = quote! {
            // Dispatch shim — canonical `NativeHandler` signature, same shape
            // `wat_intrinsic.rs`'s `emit` generates for an ordinary intrinsic. The annotated
            // eval fn's own params are ALREADY in this exact order (measured, DESIGN's "the
            // type needs no invention" table) — no context-tail reordering to do.
            fn #shim_ident(
                args: &[::wat::ast::WatAST],
                list_span: &::wat::span::Span,
                env: &::wat::value::Environment,
                sym: &::wat::value::SymbolTable,
            ) -> ::std::result::Result<::wat::value::Value, ::wat::value::EvalBreak> {
                #call
            }
        };
        (shim, quote! { ::std::option::Option::Some(#shim_ident) })
    } else {
        (TokenStream2::new(), quote! { ::std::option::Option::None })
    };

    // arc 255 Stone the-tail-door — `role = tail` emits a SEPARATE callable pointer
    // (`TailHandler`, not `NativeHandler`) into a SEPARATE `tail_handler` submission field
    // (STOP-3: never folded into `eval_handler`/`handler` — a tail impl called from
    // `dispatch_keyword_head_value`'s non-tail guard would run with its contract violated).
    // Excursus 003 strike G item 4: both doors now return bare `Value`, so there is no longer a
    // wrap direction to choose between — the eval door's forwarding above and this one are
    // identical in shape (kept as separate branches: the two submission fields stay distinct).
    let (tail_shim_tokens, tail_handler_field) = if attr.role.to_string().as_str() == "tail" {
        let fn_ident = &item.sig.ident;
        let shim_ident = format_ident!("__wat_special_form_tail_{}", fn_ident);
        let call = quote! { #fn_ident(args, list_span, env, sym) };
        let shim = quote! {
            // Dispatch shim — canonical `TailHandler` signature. The annotated tail fn's own
            // params are already in this exact order (DESIGN's "the type needs no invention"
            // table) — no context-tail reordering to do.
            fn #shim_ident(
                args: &[::wat::ast::WatAST],
                list_span: &::wat::span::Span,
                env: &::wat::value::Environment,
                sym: &::wat::value::SymbolTable,
            ) -> ::std::result::Result<::wat::value::Value, ::wat::value::EvalBreak> {
                #call
            }
        };
        (shim, quote! { ::std::option::Option::Some(#shim_ident) })
    } else {
        (TokenStream2::new(), quote! { ::std::option::Option::None })
    };

    let expanded = quote! {
        // The annotated implementation, passed through unchanged.
        #item

        // arc 255 Stone the-eval-door — the generated eval shim (role = eval only; empty
        // otherwise).
        #eval_shim_tokens

        // arc 255 Stone the-tail-door — the generated tail shim (role = tail only; empty
        // otherwise).
        #tail_shim_tokens

        // Auto-collect: link-time registration of this (fqdn, role) implementation. Gathered
        // by `registry()` and folded into the matching `Kind::SpecialForm` entry's `impls`.
        ::inventory::submit! {
            ::wat::intrinsic::SpecialFormImplSubmission {
                name: #fqdn,
                role: #role_token,
                source: #source_lit,
                eval_handler: #eval_handler_field,
                tail_handler: #tail_handler_field,
            }
        }
    };

    Ok(expanded)
}
