//! Codegen for `#[wat_intrinsic("<fqdn>")]` — arc 255.1b-ii / iv-b1.
//!
//! Applied to a handler fn written with either a **fixed-arg signature** (each
//! wat arg as a `&WatAST` param) or a **variadic signature** (single `&[WatAST]`
//! slice param). The context tail (`env: &Environment, sym: &SymbolTable,
//! span: &Span`) follows in both cases.  The attribute:
//!
//!   1. **Sniffs args** — collects the leading `&WatAST` param idents (those
//!      BEFORE the context tail). N such params ⇒ `Exact(N)`. A single
//!      `&[WatAST]` leading param ⇒ `Variadic` (the slice is passed through
//!      directly; no arity check in the shim).
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

/// Result of sniffing the handler signature's arg structure.
enum SniffedArgs {
    /// N leading `&WatAST` params — fixed arity.
    Exact(Vec<String>),
    /// A single leading `&[WatAST]` param — variadic (any number of args).
    Variadic(String),
}

/// Parse the leading wat-side params from a handler signature.
/// Returns `SniffedArgs::Exact(names)` for fixed-arity handlers
/// (`&WatAST` params leading) or `SniffedArgs::Variadic(name)` for
/// a single `&[WatAST]` param.
/// The context tail (`&Environment`, `&SymbolTable`, `&Span`) follows.
fn sniff_args(item: &ItemFn) -> syn::Result<SniffedArgs> {
    let mut wat_args: Vec<String> = Vec::new();
    let mut seen_context = false;
    let mut variadic_param: Option<String> = None;

    for input in item.sig.inputs.iter() {
        let FnArg::Typed(pt) = input else {
            return Err(Error::new_spanned(
                input,
                "wat_intrinsic: handler fns take no `self` receiver",
            ));
        };
        if is_ref_watast_slice(&pt.ty) {
            // Variadic shape: a single `&[WatAST]` param.
            if seen_context || !wat_args.is_empty() || variadic_param.is_some() {
                return Err(Error::new_spanned(
                    &pt.ty,
                    "wat_intrinsic: `&[WatAST]` variadic param must be the SOLE \
                     leading param (before context tail; no mixing with `&WatAST` params)",
                ));
            }
            let ident = match &*pt.pat {
                Pat::Ident(pi) => pi.ident.to_string(),
                other => {
                    return Err(Error::new_spanned(
                        other,
                        "wat_intrinsic: `&[WatAST]` variadic param must be a plain ident pattern",
                    ));
                }
            };
            variadic_param = Some(ident);
        } else if is_ref_watast(&pt.ty) {
            if seen_context || variadic_param.is_some() {
                // A `&WatAST` after a context param or variadic param — violated.
                return Err(Error::new_spanned(
                    &pt.ty,
                    "wat_intrinsic: all `&WatAST` arg params must precede the \
                     context tail (env/sym/span) and cannot mix with a variadic param",
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
        } else {
            // First non-`&WatAST`/`&[WatAST]` param marks the start of the context tail.
            seen_context = true;
        }
    }

    if let Some(name) = variadic_param {
        Ok(SniffedArgs::Variadic(name))
    } else {
        Ok(SniffedArgs::Exact(wat_args))
    }
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
            format!("unknown doc directive `{}`; recognized: @added @arg @ret @example @example-norun @deprecated @see @pure @deterministic @category", tag)
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
        wat_doc::DocError::MissingPure => {
            "doc comment is missing a required `@Purity <Variant>` directive".into()
        }
        wat_doc::DocError::MissingDeterministic => {
            "doc comment is missing a required `@Determinism <Variant>` directive".into()
        }
        wat_doc::DocError::MissingCategory => {
            format!("doc comment is missing a required `@Category <Variant>` directive (known: {})",
                wat_doc::Category::variants().join(", "))
        }
        wat_doc::DocError::MissingSyntax => {
            "doc comment is missing a required `@syntax (...)` directive (special forms only)".into()
        }
        wat_doc::DocError::MissingShape => {
            "doc comment has neither `@arg` nor `@syntax`; at least one must express the form's \
             shape (`@arg` for positional forms — grammar derived; `@syntax` for structural forms)".into()
        }
        wat_doc::DocError::MissingPurity => {
            "doc comment is missing a required `@Purity <Variant>` directive (known: Pure, Effectful, Preserving)".into()
        }
        wat_doc::DocError::MissingDeterminism => {
            "doc comment is missing a required `@Determinism <Variant>` directive (known: Deterministic, Nondeterministic, Preserving)".into()
        }
        wat_doc::DocError::InvalidPurityVariant { got } => {
            format!("unknown @Purity variant `{}`; known: Pure, Effectful, Preserving", got)
        }
        wat_doc::DocError::InvalidDeterminismVariant { got } => {
            format!("unknown @Determinism variant `{}`; known: Deterministic, Nondeterministic, Preserving", got)
        }
        wat_doc::DocError::InvalidCategoryVariant { got } => {
            format!("unknown @Category variant `{}`; known: {}", got, wat_doc::Category::variants().join(", "))
        }
    }
}

pub(crate) fn emit(fqdn: &LitStr, item: &ItemFn) -> syn::Result<TokenStream2> {
    let sniffed = sniff_args(item)?;

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

    // Build the param-name list for check_args and the shim.
    // For Variadic, pass the single rest-param name (matches the one `@arg xs…` doc entry).
    let (arg_names, is_variadic): (Vec<String>, bool) = match &sniffed {
        SniffedArgs::Exact(names) => (names.clone(), false),
        SniffedArgs::Variadic(name) => (vec![name.clone()], true),
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
    let source_lit = quote!(#item).to_string();

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
            let is_rest = a.is_rest;
            quote! { (#name, #ty, #desc, #is_rest) }
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

    let purity_token = match doc.purity {
        wat_doc::Purity::Pure => quote! { ::wat_doc::Purity::Pure },
        wat_doc::Purity::Effectful => quote! { ::wat_doc::Purity::Effectful },
        wat_doc::Purity::Preserving => quote! { ::wat_doc::Purity::Preserving },
    };
    let determinism_token = match doc.determinism {
        wat_doc::Determinism::Deterministic => quote! { ::wat_doc::Determinism::Deterministic },
        wat_doc::Determinism::Nondeterministic => quote! { ::wat_doc::Determinism::Nondeterministic },
        wat_doc::Determinism::Preserving => quote! { ::wat_doc::Determinism::Preserving },
    };
    let category_token = match doc.category {
        wat_doc::Category::Transform => quote! { ::wat_doc::Category::Transform },
        wat_doc::Category::Reflection => quote! { ::wat_doc::Category::Reflection },
        wat_doc::Category::ControlFlow => quote! { ::wat_doc::Category::ControlFlow },
        wat_doc::Category::Binding => quote! { ::wat_doc::Category::Binding },
        wat_doc::Category::Entropic => quote! { ::wat_doc::Category::Entropic },
        wat_doc::Category::Arithmetic => quote! { ::wat_doc::Category::Arithmetic },
        wat_doc::Category::Io => quote! { ::wat_doc::Category::Io },
        wat_doc::Category::Probe => quote! { ::wat_doc::Category::Probe },
        wat_doc::Category::Combine => quote! { ::wat_doc::Category::Combine },
        wat_doc::Category::Declaration => quote! { ::wat_doc::Category::Declaration },
        wat_doc::Category::Resource => quote! { ::wat_doc::Category::Resource },
        wat_doc::Category::Message => quote! { ::wat_doc::Category::Message },
        wat_doc::Category::Ambient => quote! { ::wat_doc::Category::Ambient },
        wat_doc::Category::Project => quote! { ::wat_doc::Category::Project },
        wat_doc::Category::CheckGate => quote! { ::wat_doc::Category::CheckGate },
    };

    let yields_type_lit = match &doc.yields {
        Some(y) => {
            let ty = &y.ty;
            quote! { ::std::option::Option::Some(#ty) }
        }
        None => quote! { ::std::option::Option::None },
    };

    // Emit the arity value: `Arity::Exact(N)` or `Arity::Variadic`.
    let arity_lit = if is_variadic {
        quote! { ::wat::intrinsic::Arity::Variadic }
    } else {
        let n = arg_names.len();
        quote! { ::wat::intrinsic::Arity::Exact(#n) }
    };

    // Build the shim body. For exact-arity: check len == N, then forward individual refs.
    // For variadic: pass the whole slice directly (no arity check — 0+ args all valid).
    let shim_body = if is_variadic {
        // Variadic: pass the whole slice to the handler.
        quote! {
            #fn_name(args, env, sym, list_span)
        }
    } else {
        let n = arg_names.len();
        let arg_forwards: Vec<TokenStream2> = (0..n).map(|i| quote! { &args[#i] }).collect();
        quote! {
            if args.len() != #n {
                return ::std::result::Result::Err(
                    ::wat::value::RuntimeError::new(list_span.clone(), ::wat::value::RuntimeErrorKind::ArityMismatch {
                            op: #fqdn.into(),
                            expected: #n,
                            got: args.len(),
                        })
                    .into(),
                );
            }
            #fn_name(#(#arg_forwards,)* env, sym, list_span)
        }
    };

    let expanded = quote! {
        // The annotated handler, passed through unchanged.
        #item

        // Dispatch shim — canonical NativeHandler signature.
        fn #shim_ident(
            args: &[::wat::ast::WatAST],
            list_span: &::wat::span::Span,
            env: &::wat::value::Environment,
            sym: &::wat::value::SymbolTable,
        ) -> ::std::result::Result<::wat::value::Value, ::wat::value::EvalBreak> {
            #shim_body
        }

        // Auto-collect: link-time registration of (fqdn → shim) into the
        // IntrinsicRegistry. `registry()` iterates these submissions.
        ::inventory::submit! {
            ::wat::intrinsic::IntrinsicSubmission {
                name: #fqdn,
                handler: #shim_ident,
                arity: #arity_lit,
                prose: #prose_lit,
                added: #added_lit,
                args: &[#(#args_lit),*],
                ret_type: #ret_type_lit,
                ret: #ret_lit,
                examples: &[#(#examples_lit),*],
                deprecated: #deprecated_lit,
                see: &[#(#see_lit),*],
                source: #source_lit,
                purity: #purity_token,
                determinism: #determinism_token,
                category: #category_token,
                yields_type: #yields_type_lit,
            }
        }
    };

    Ok(expanded)
}
