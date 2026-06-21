//! Codegen for `#[wat_intrinsic("<fqdn>")]` — arc 255.1b-ii.
//!
//! Applied to a handler fn written with a **fixed-arg signature**: the wat
//! args as individual `&WatAST` params, followed by the context tail
//! (`env: &Environment, sym: &SymbolTable, span: &Span`). The attribute:
//!
//!   1. **Sniffs arity** — counts the leading `&WatAST` params (those BEFORE
//!      the context tail). N such params ⇒ `Exact(N)`. (This strike only
//!      needs Exact-N; a trailing `&[WatAST]` slice would be Variadic, but
//!      core::Bytes is Exact(1) twice, so a shape we can't classify is a
//!      hard compile_error! rather than a silent guess.)
//!
//!   2. **Emits a dispatch shim** with the canonical `NativeHandler`
//!      signature `fn(&[WatAST], &Span, &Environment, &SymbolTable)
//!      -> Result<Value, EvalBreak>`. The shim checks `args.len() == N`
//!      (returning the SAME `RuntimeErrorKind::ArityMismatch` shape the
//!      hand-written handlers used — `op` = the fqdn, `expected` = N,
//!      `got` = args.len(), span = the list_span), then calls the fixed-arg
//!      fn with `&args[0], …, env, sym, span`.
//!
//!   3. **Registers** the (fqdn → shim) into the `IntrinsicRegistry` via
//!      `inventory::submit!` of an `IntrinsicSubmission`. `registry()` builds
//!      itself by iterating `inventory::iter::<IntrinsicSubmission>`.
//!
//! Example:
//! ```ignore
//! #[wat_intrinsic(":wat::core::Bytes::to-hex")]
//! pub(crate) fn bytes_to_hex(
//!     s: &WatAST,
//!     env: &Environment,
//!     sym: &SymbolTable,
//!     span: &Span,
//! ) -> Result<Value, EvalBreak> { ... }
//! ```

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Error, FnArg, ItemFn, LitStr, Type};

/// Parse the leading `&WatAST` arg count (the wat-side arity) from a
/// fixed-arg handler signature. The context tail (`&Environment`,
/// `&SymbolTable`, `&Span`) follows the wat args; we count `&WatAST`
/// params and require they be a contiguous leading prefix.
fn sniff_arity(item: &ItemFn) -> syn::Result<usize> {
    let mut wat_arg_count = 0usize;
    let mut seen_context = false;

    for input in item.sig.inputs.iter() {
        let FnArg::Typed(pt) = input else {
            return Err(Error::new_spanned(
                input,
                "wat_intrinsic: handler fns take no `self` receiver",
            ));
        };
        if is_ref_watast(&pt.ty) {
            if seen_context {
                // A `&WatAST` after a context param — the leading-args
                // contract is violated. STOP rather than guess.
                return Err(Error::new_spanned(
                    &pt.ty,
                    "wat_intrinsic: all `&WatAST` arg params must precede the \
                     context tail (env/sym/span); cannot classify this shape",
                ));
            }
            wat_arg_count += 1;
        } else if is_ref_watast_slice(&pt.ty) {
            // Variadic shape: `&[WatAST]`. Not handled this strike.
            return Err(Error::new_spanned(
                &pt.ty,
                "wat_intrinsic: variadic `&[WatAST]` handlers are not supported \
                 by this strike (255.1b-ii covers Exact-N only); STOP-1",
            ));
        } else {
            // First non-`&WatAST` param marks the start of the context tail.
            seen_context = true;
        }
    }

    Ok(wat_arg_count)
}

/// Is the type `&WatAST` (with optional path qualification)?
fn is_ref_watast(ty: &Type) -> bool {
    if let Type::Reference(r) = ty {
        return type_path_ends_with(&r.elem, "WatAST");
    }
    false
}

/// Is the type `&[WatAST]`?
fn is_ref_watast_slice(ty: &Type) -> bool {
    if let Type::Reference(r) = ty {
        if let Type::Slice(s) = &*r.elem {
            return type_path_ends_with(&s.elem, "WatAST");
        }
    }
    false
}

/// Does the type's final path segment equal `name`? (Tolerates
/// `WatAST`, `ast::WatAST`, `crate::ast::WatAST`, etc.)
fn type_path_ends_with(ty: &Type, name: &str) -> bool {
    if let Type::Path(p) = ty {
        if let Some(last) = p.path.segments.last() {
            return last.ident == name;
        }
    }
    false
}

pub(crate) fn emit(fqdn: &LitStr, item: &ItemFn) -> syn::Result<TokenStream2> {
    let arity = sniff_arity(item)?;

    let fn_name = &item.sig.ident;
    let shim_ident = format_ident!("__wat_intrinsic_shim_{}", fn_name);

    // The shim forwards `&args[0], &args[1], …, env, sym, span` to the
    // fixed-arg handler. Indices 0..arity feed the wat-arg params; the
    // context tail is `env, sym, span` in that order.
    let arg_forwards: Vec<TokenStream2> = (0..arity)
        .map(|i| quote! { &args[#i] })
        .collect();

    let expanded = quote! {
        // The annotated handler, passed through unchanged.
        #item

        // Dispatch shim — canonical NativeHandler signature. Bridges the
        // registry's slice-based ABI to the fixed-arg handler, enforcing
        // arity with the SAME ArityMismatch shape the hand-written
        // handlers produced (op = fqdn, expected = N, got = len, span =
        // the call's list_span).
        fn #shim_ident(
            args: &[::wat::ast::WatAST],
            list_span: &::wat::span::Span,
            env: &::wat::value::Environment,
            sym: &::wat::value::SymbolTable,
        ) -> ::std::result::Result<::wat::value::Value, ::wat::value::EvalBreak> {
            if args.len() != #arity {
                return ::std::result::Result::Err(
                    ::wat::value::RuntimeError {
                        span: list_span.clone(),
                        kind: ::wat::value::RuntimeErrorKind::ArityMismatch {
                            op: #fqdn.into(),
                            expected: #arity,
                            got: args.len(),
                        },
                    }
                    .into(),
                );
            }
            #fn_name(#(#arg_forwards,)* env, sym, list_span)
        }

        // Auto-collect: link-time registration of (fqdn → shim) into the
        // IntrinsicRegistry. `registry()` iterates these submissions.
        ::inventory::submit! {
            ::wat::intrinsic::IntrinsicSubmission {
                name: #fqdn,
                handler: #shim_ident,
            }
        }
    };

    Ok(expanded)
}
