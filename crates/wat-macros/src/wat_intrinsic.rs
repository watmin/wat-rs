//! Codegen for `#[wat_intrinsic("<fqdn>")]` — arc 255.1b-ii / iv-b1.
//!
//! Applied to a handler fn written with a **fixed-arg signature**: the wat
//! args as individual `&WatAST` params, followed by the context tail
//! (`env: &Environment, sym: &SymbolTable, span: &Span`). The attribute:
//!
//!   1. **Sniffs args** — collects the leading `&WatAST` param idents (those
//!      BEFORE the context tail). N such params ⇒ `Exact(N)`. (This strike only
//!      needs Exact-N; a trailing `&[WatAST]` slice would be Variadic, but
//!      core::Bytes is Exact(1) twice, so a shape we can't classify is a
//!      hard compile_error! rather than a silent guess.)
//!
//!   2. **Parses the `///` block** via `wat_doc::parse`, enforcing the full
//!      doc contract at expand time (`compile_error!` on any `DocError`).
//!      Then runs `wat_doc::check_args` to verify `@arg` names match the
//!      handler's parameter idents.
//!
//!   3. **Emits a dispatch shim** with the canonical `NativeHandler`
//!      signature `fn(&[WatAST], &Span, &Environment, &SymbolTable)
//!      -> Result<Value, EvalBreak>`. The shim checks `args.len() == N`
//!      (returning the SAME `RuntimeErrorKind::ArityMismatch` shape the
//!      hand-written handlers used — `op` = the fqdn, `expected` = N,
//!      `got` = args.len(), span = the list_span), then calls the fixed-arg
//!      fn with `&args[0], …, env, sym, span`.
//!
//!   4. **Registers** the (fqdn → shim) into the `IntrinsicRegistry` via
//!      `inventory::submit!` of an `IntrinsicSubmission`, carrying the full
//!      structured doc (prose/added/args/ret/examples/deprecated/see) as
//!      `'static` literals.
//!
//! Example:
//! ```ignore
//! /// Encode a `:wat::core::Bytes` into its lowercase-hex `:String`.
//! ///
//! /// @added 1.0.0
//! /// @arg bs — the bytes to encode
//! /// @ret the lowercase hex string, two chars per byte, no separators
//! /// @example (:wat::core::Bytes::to-hex (:wat::core::Vector 255 0 16)) #=> "ff0010"
//! #[wat_intrinsic(":wat::core::Bytes::to-hex")]
//! pub(crate) fn bytes_to_hex(
//!     bs: &WatAST,
//!     env: &Environment,
//!     sym: &SymbolTable,
//!     span: &Span,
//! ) -> Result<Value, EvalBreak> { ... }
//! ```

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Error, Expr, ExprLit, FnArg, ItemFn, Lit, LitStr, Meta, Pat, Type};

/// Parse the leading `&WatAST` arg idents (the wat-side params) from a
/// fixed-arg handler signature. Returns (arg_idents, arity).
/// The context tail (`&Environment`, `&SymbolTable`, `&Span`) follows the
/// wat args; we collect only `&WatAST` params from the leading prefix.
fn sniff_args(item: &ItemFn) -> syn::Result<Vec<String>> {
    let mut wat_args: Vec<String> = Vec::new();
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
            // Extract the ident from the pattern.
            let ident = match &*pt.pat {
                Pat::Ident(pi) => pi.ident.to_string(),
                other => {
                    return Err(Error::new_spanned(
                        other,
                        "wat_intrinsic: `&WatAST` param must be a plain ident pattern",
                    ));
                }
            };
            wat_args.push(ident);
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

    Ok(wat_args)
}

/// Sniff the handler fn's docstring — the Clojure-style whole string. `///`
/// lines desugar to `#[doc = "…"]` attrs (one per line); we collect every
/// such `doc` string literal, strip the single leading space syn leaves on
/// each `///` line, and join with `\n` — VERBATIM, no curation/splitting.
/// Returns `None` when there are no `#[doc]` attrs (doc absent).
fn sniff_doc(item: &ItemFn) -> Option<String> {
    let lines: Vec<String> = item
        .attrs
        .iter()
        .filter_map(|attr| {
            if let Meta::NameValue(nv) = &attr.meta {
                if nv.path.is_ident("doc") {
                    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = &nv.value {
                        // `///` desugars to `#[doc = " text"]` (one leading
                        // space). Strip that single space; keep the rest verbatim.
                        let raw = s.value();
                        return Some(raw.strip_prefix(' ').map(str::to_owned).unwrap_or(raw));
                    }
                }
            }
            None
        })
        .collect();

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

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

/// Render a `wat_doc::DocError` into a precise human message for `compile_error!`.
fn render_doc_error(e: &wat_doc::DocError) -> String {
    match e {
        wat_doc::DocError::MissingProse => {
            "doc comment has no prose (text before the first @-directive is required)".into()
        }
        wat_doc::DocError::MissingAdded => {
            "doc comment is missing a required `@added <version>` directive".into()
        }
        wat_doc::DocError::MissingRet => {
            "doc comment is missing a required `@ret <desc>` directive".into()
        }
        wat_doc::DocError::MissingExample => {
            "doc comment must have at least one `@example` or `@example-norun` directive".into()
        }
        wat_doc::DocError::MalformedDirective { tag, why } => {
            format!("malformed `{}` directive: {}", tag, why)
        }
        wat_doc::DocError::UnknownDirective { tag } => {
            format!("unknown doc directive `{}`; recognized: @added @arg @ret @example @example-norun @deprecated @see", tag)
        }
        wat_doc::DocError::ExampleMissingMarker { expr } => {
            format!(
                "`@example` must carry a `#=>` expected-value marker; \
                 use `@example-norun` if no expected value — got: `{}`",
                expr
            )
        }
        wat_doc::DocError::DuplicateSingleton { tag } => {
            format!("duplicate singleton directive `{}`; may appear at most once", tag)
        }
        wat_doc::DocError::ArgCountMismatch { documented, signature } => {
            format!(
                "@arg count ({}) does not match the handler's `&WatAST` parameter count ({})",
                documented, signature
            )
        }
        wat_doc::DocError::ArgNameMismatch { position, documented, signature } => {
            format!(
                "@arg at position {} names `{}` but the handler parameter is `{}`",
                position, documented, signature
            )
        }
    }
}

pub(crate) fn emit(fqdn: &LitStr, item: &ItemFn) -> syn::Result<TokenStream2> {
    let arg_names: Vec<String> = sniff_args(item)?;
    let arity = arg_names.len();

    // Require a doc comment; parse it through wat_doc.
    let raw_doc = match sniff_doc(item) {
        Some(d) => d,
        None => {
            return Err(Error::new_spanned(
                item,
                format!(
                    "#[wat_intrinsic] {}: missing doc comment (/// is required; \
                     must include @added, @ret, and at least one @example)",
                    fqdn.value()
                ),
            ));
        }
    };

    let doc = match wat_doc::parse(&raw_doc) {
        Ok(d) => d,
        Err(e) => {
            return Err(Error::new_spanned(
                item,
                format!("#[wat_intrinsic] {}: {}", fqdn.value(), render_doc_error(&e)),
            ));
        }
    };

    // Check @arg names against signature param idents.
    let arg_name_refs: Vec<&str> = arg_names.iter().map(String::as_str).collect();
    if let Err(e) = wat_doc::check_args(&doc, &arg_name_refs) {
        return Err(Error::new_spanned(
            item,
            format!("#[wat_intrinsic] {}: {}", fqdn.value(), render_doc_error(&e)),
        ));
    }

    let fn_name = &item.sig.ident;
    let shim_ident = format_ident!("__wat_intrinsic_shim_{}", fn_name);

    // Arc 255.1b-v — capture the handler source via stable restringify.
    // `quote!(#item).to_string()` re-serializes the ItemFn's token stream.
    // Comments are NOT preserved (token-level, not source-level), but the
    // structural source — signature + body — is faithful-if-reformatted.
    // `proc_macro::Span::source_text` would be exact but is nightly-only;
    // the contract names this stable fallback. STOP-2: if `#item` is not in
    // scope at this point (the ItemFn before we expand it), the quote! will
    // fail to compile — but ItemFn IS in scope here (we have it as `item`).
    let source_lit = quote!(#item).to_string();

    // The shim forwards `&args[0], &args[1], …, env, sym, span` to the
    // fixed-arg handler. Indices 0..arity feed the wat-arg params; the
    // context tail is `env, sym, span` in that order.
    let arg_forwards: Vec<TokenStream2> = (0..arity)
        .map(|i| quote! { &args[#i] })
        .collect();

    // Emit 'static literals for the structured doc fields.
    let prose_lit = &doc.prose;
    let added_lit = &doc.added;
    let ret_type_lit = &doc.ret_type;
    let ret_lit = &doc.ret;

    let args_lit: Vec<TokenStream2> = doc
        .args
        .iter()
        .map(|a| {
            let name = &a.name;
            let ty = &a.ty;
            let desc = &a.desc;
            quote! { (#name, #ty, #desc) }
        })
        .collect();

    let examples_lit: Vec<TokenStream2> = doc
        .examples
        .iter()
        .map(|ex| {
            let expr = &ex.expr;
            let run = ex.run;
            let expected = match &ex.expected {
                Some(s) => quote! { ::std::option::Option::Some(#s) },
                None => quote! { ::std::option::Option::None },
            };
            quote! {
                ::wat::intrinsic::ExampleSubmission {
                    expr: #expr,
                    expected: #expected,
                    run: #run,
                }
            }
        })
        .collect();

    let deprecated_lit = match &doc.deprecated {
        Some(d) => {
            let since = &d.since;
            let use_instead = &d.use_instead;
            quote! { ::std::option::Option::Some((#since, #use_instead)) }
        }
        None => quote! { ::std::option::Option::None },
    };

    let see_lit: Vec<&str> = doc.see.iter().map(String::as_str).collect();

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
                arity: #arity,
                prose: #prose_lit,
                added: #added_lit,
                args: &[#(#args_lit),*],
                ret_type: #ret_type_lit,
                ret: #ret_lit,
                examples: &[#(#examples_lit),*],
                deprecated: #deprecated_lit,
                see: &[#(#see_lit),*],
                source: #source_lit,
            }
        }
    };

    Ok(expanded)
}
