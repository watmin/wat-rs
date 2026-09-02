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

use crate::wat_intrinsic::{sniff_return, wrap_call_for_return, SniffedReturn};

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

/// Wrap a `role = tail` handler call per its sniffed return shape — the TAIL DOOR's own fold,
/// the INVERSE of `wat_intrinsic.rs`'s `wrap_call_for_return`. Arc 255 Stone the-tail-door,
/// STOP-4: the eval door's `wrap_call_for_return` wraps a handler's return TO `TrackedValue`
/// (what `NativeHandler` returns); a tail shim must produce a bare `Value` (what `TailHandler`
/// returns and what `eval_tail` itself returns), so the two directions cannot share one fn
/// without a direction flag answering two questions. `sniff_return` — the DETECTION of which
/// shape the annotated fn returns — is still reused verbatim from `wat_intrinsic.rs`; only the
/// wrap it feeds is a sibling, not a shared body.
fn wrap_call_for_tail_return(sniffed_return: &SniffedReturn, call: TokenStream2) -> TokenStream2 {
    match sniffed_return {
        // `eval_if_tail`/`eval_match_tail` already return `Result<Value, EvalBreak>` — pass
        // through unchanged, mirroring the arm `eval_tail`'s own `if`/`match` arms use today.
        SniffedReturn::BareValue => call,
        // `eval_let_tail` returns `Result<TrackedValue, EvalBreak>` — unwrap via
        // `.map(|tv| tv.value_owned())`, EXACTLY the adapter `eval_tail`'s own
        // `:wat::core::let` arm performs today (a move, not new logic; DESIGN's "the type needs
        // no invention" table).
        SniffedReturn::Tracked => quote! {
            #call.map(|tv| tv.value_owned())
        },
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
    // statically, and has no per-invocation call site to dispatch through. STOP-4: the
    // Value-vs-TrackedValue decision is NOT re-derived here — `sniff_return` and
    // `wrap_call_for_return` are the SAME fns `wat_intrinsic.rs`'s `emit` calls, made
    // `pub(crate)` for exactly this reuse.
    let (eval_shim_tokens, eval_handler_field) = if attr.role.to_string().as_str() == "eval" {
        let fn_ident = &item.sig.ident;
        let sniffed_return = sniff_return(item)?;
        let shim_ident = format_ident!("__wat_special_form_eval_{}", fn_ident);
        let call = wrap_call_for_return(&sniffed_return, quote! { #fn_ident(args, list_span, env, sym) });
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
            ) -> ::std::result::Result<::wat::value::TrackedValue, ::wat::value::EvalBreak> {
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
    // STOP-4: `sniff_return` is reused verbatim (the DETECTION is identical to the eval door's);
    // the WRAP is `wrap_call_for_tail_return`, above — the eval door's wrap goes TO
    // `TrackedValue`, this one goes TO bare `Value`, so the two directions are siblings, not one
    // fn with a flag.
    let (tail_shim_tokens, tail_handler_field) = if attr.role.to_string().as_str() == "tail" {
        let fn_ident = &item.sig.ident;
        let sniffed_return = sniff_return(item)?;
        let shim_ident = format_ident!("__wat_special_form_tail_{}", fn_ident);
        let call = wrap_call_for_tail_return(&sniffed_return, quote! { #fn_ident(args, list_span, env, sym) });
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
