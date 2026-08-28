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
use syn::{Error, Expr, ExprLit, FnArg, GenericArgument, ItemFn, Lit, LitStr, Meta, Pat,
    PathArguments, ReturnType, Type};

/// The parsed `#[wat_intrinsic(...)]` attribute payload (arc 255 Stone N).
///
/// The ~250 pre-existing call sites are a bare FQDN string literal —
/// `#[wat_intrinsic(":wat::hashmap::length")]` — and parse exactly as
/// before, with `value_fn: None`. A handler that ALSO has a value-level
/// implementation reachable from `:wat::core::apply`'s substrate-impl
/// fallback (`dispatch_substrate_impl`, `src/runtime.rs`) may additionally
/// name it: `#[wat_intrinsic(":wat::hashmap::length", value = eval_hashmap_length_value)]`.
/// `value_fn` must name a fn matching `fn(&[Value], &Span) -> Result<Value, EvalBreak>`
/// (arc 255 Stone Q widened `ValueHandler` with a trailing `&Span` — the call's own, not
/// binding state; the fn may ignore it) in scope at the call site — the macro does not check
/// the signature itself; a mismatch is a normal Rust type error at the `IntrinsicSubmission`
/// literal.
pub(crate) struct WatIntrinsicAttr {
    pub(crate) fqdn: LitStr,
    pub(crate) value_fn: Option<syn::Path>,
}

impl syn::parse::Parse for WatIntrinsicAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fqdn: LitStr = input.parse()?;
        let mut value_fn = None;
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            let key: syn::Ident = input.parse()?;
            if key != "value" {
                return Err(Error::new_spanned(
                    &key,
                    "wat_intrinsic: expected `value = <path>` as the only optional argument",
                ));
            }
            input.parse::<syn::Token![=]>()?;
            value_fn = Some(input.parse()?);
        }
        Ok(WatIntrinsicAttr { fqdn, value_fn })
    }
}

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
        } else if is_ref_value(&pt.ty) || is_ref_value_slice(&pt.ty) {
            // arc 255 Stone O-iii, STOP-3 (the `&WatAST`-leading direction): a BINDING
            // handler's leading params are `&WatAST`/`&[WatAST]` only. A `&Value`/`&[Value]`
            // param here is the OTHER kind's shape wandering into this one's signature — not a
            // context param to wave through silently.
            return Err(Error::new_spanned(
                &pt.ty,
                "wat_intrinsic: cannot mix `&WatAST`/`&[WatAST]` params with `&Value`/`&[Value]` \
                 params in one signature — a BINDING handler's leading params are \
                 `&WatAST`/`&[WatAST]` only (an ALGEBRA handler's are `&Value`/`&[Value]` only)",
            ));
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

/// The handler's KIND (arc 255 Stone O-iii) — the third sniff on the same mechanism as
/// `sniff_args` (the argument shape) and `sniff_return` (the return shape), applied to which
/// TYPE the leading params are. Decided by the FIRST param: `&WatAST`/`&[WatAST]` ⇒ BINDING
/// (today's shape, parsed by `sniff_args`, untouched); `&Value`/`&[Value]` ⇒ ALGEBRA (the macro
/// generates BOTH the value door — the fn itself — and the AST door, behind one arity check).
enum IntrinsicKind {
    /// Leading `&WatAST`/`&[WatAST]` params. AST door only; unchanged in every respect.
    Binding(SniffedArgs),
    /// Leading `&Value`/`&[Value]` params, then NOTHING or a single trailing `&Span`
    /// (arc 255 Stone Q). Both doors generated. The `bool` is `true` when the handler
    /// itself declared the trailing `&Span` and wants it forwarded.
    Algebra(SniffedArgs, bool),
}

/// Sniff the handler's kind, then (for ALGEBRA) parse its leading `&Value`/`&[Value]` params —
/// same shape `sniff_args` parses for `&WatAST`/`&[WatAST]`, different predicate — and enforce
/// STOP-1: an ALGEBRA handler's leading run must be followed by NOTHING or a single trailing
/// `&Span` (arc 255 Stone Q — `env`/`sym` remain forbidden as binding state a splatted handler
/// genuinely cannot use; a span is not binding state, it is a location) and STOP-3 in the
/// ALGEBRA-leading direction (a later `&WatAST`/`&[WatAST]` param is the other kind's shape,
/// rejected).
fn sniff_kind(item: &ItemFn) -> syn::Result<IntrinsicKind> {
    let is_algebra = matches!(
        item.sig.inputs.iter().next(),
        Some(FnArg::Typed(pt)) if is_ref_value(&pt.ty) || is_ref_value_slice(&pt.ty)
    );
    if !is_algebra {
        return Ok(IntrinsicKind::Binding(sniff_args(item)?));
    }

    let mut wat_args: Vec<String> = Vec::new();
    let mut variadic_param: Option<String> = None;
    // Arc 255 Stone Q — an ALGEBRA handler may take AT MOST one trailing `&Span`, and it must
    // be the LAST param (nothing after it — same "context tail ends the signature" shape
    // `sniff_args` enforces for BINDING's `&WatAST` run).
    let mut span_seen = false;

    for input in item.sig.inputs.iter() {
        let FnArg::Typed(pt) = input else {
            return Err(Error::new_spanned(
                input,
                "wat_intrinsic: handler fns take no `self` receiver",
            ));
        };
        if span_seen {
            // Stone Q: nothing may follow the trailing `&Span` — including another `&Span`.
            return Err(Error::new_spanned(
                &pt.ty,
                "wat_intrinsic: an ALGEBRA handler's trailing `&Span` (arc 255 Stone Q) must be \
                 the LAST param — nothing may follow it",
            ));
        }
        if is_ref_value_slice(&pt.ty) {
            if !wat_args.is_empty() || variadic_param.is_some() {
                return Err(Error::new_spanned(
                    &pt.ty,
                    "wat_intrinsic: `&[Value]` variadic param must be the SOLE leading param \
                     (no mixing with `&Value` params)",
                ));
            }
            variadic_param = Some(pat_ident(&pt.pat)?);
        } else if is_ref_value(&pt.ty) {
            if variadic_param.is_some() {
                return Err(Error::new_spanned(
                    &pt.ty,
                    "wat_intrinsic: `&Value` param cannot follow a `&[Value]` variadic param",
                ));
            }
            wat_args.push(pat_ident(&pt.pat)?);
        } else if is_ref_span(&pt.ty) {
            // Arc 255 Stone Q — the ONE thing allowed after the leading `&Value`/`&[Value]`
            // run. Not binding state (STOP-1 still forbids `env`/`sym`): a span is a location,
            // `apply` already holds one, and this is the ONLY place a value-door handler can
            // receive it.
            span_seen = true;
        } else if is_ref_watast(&pt.ty) || is_ref_watast_slice(&pt.ty) {
            return Err(Error::new_spanned(
                &pt.ty,
                "wat_intrinsic: cannot mix `&Value`/`&[Value]` params with `&WatAST`/`&[WatAST]` \
                 params in one signature — an ALGEBRA handler's leading params are \
                 `&Value`/`&[Value]` only",
            ));
        } else {
            // arc 255 Stone O-iii, STOP-1: a `&Value`-leading fn that also takes `env` or `sym`
            // is a contradiction, not a shape to accommodate — algebra by definition needs
            // neither. (A trailing `&Span` IS accommodated — see the `is_ref_span` arm above;
            // arc 255 Stone Q — a span is not binding state.) If an existing `:wat::vector::`
            // verb turns out to need env/sym, it is BINDING, not ALGEBRA: give it `&WatAST`
            // leading params instead.
            return Err(Error::new_spanned(
                &pt.ty,
                "wat_intrinsic: an ALGEBRA handler (leading `&Value`/`&[Value]` params) cannot \
                 also take `env` or `sym` — algebra needs neither; a single trailing `&Span` IS \
                 allowed (arc 255 Stone Q). If this handler genuinely needs the environment, it \
                 is BINDING: give it `&WatAST` leading params instead",
            ));
        }
    }

    let sniffed = if let Some(name) = variadic_param {
        SniffedArgs::Variadic(name)
    } else {
        SniffedArgs::Exact(wat_args)
    };
    Ok(IntrinsicKind::Algebra(sniffed, span_seen))
}

/// Extract the plain ident from a param pattern, or reject it. Shared by `sniff_args`'s
/// (unchanged) inline extraction and `sniff_kind`'s ALGEBRA-side parse.
fn pat_ident(pat: &Pat) -> syn::Result<String> {
    match pat {
        Pat::Ident(pi) => Ok(pi.ident.to_string()),
        other => Err(Error::new_spanned(
            other,
            "wat_intrinsic: param must be a plain ident pattern",
        )),
    }
}

/// Result of sniffing the handler's RETURN type — the same shape as `SniffedArgs`, applied to
/// the return side instead of the argument side (arc 255 Stone G).
enum SniffedReturn {
    /// `-> Result<Value, EvalBreak>` — the ~250 pre-existing handlers. The shim wraps the
    /// returned bare `Value` as `TrackedValue::new(v, Provenance::Unknown)` — unchanged default.
    BareValue,
    /// `-> Result<TrackedValue, EvalBreak>` — a handler that WANTS to stamp its own
    /// provenance (e.g. `Provenance::RuntimeBuilt`). The shim passes the returned
    /// `TrackedValue` through un-rewrapped.
    Tracked,
}

/// Sniff the handler fn's return type: `Result<Value, EvalBreak>` or
/// `Result<TrackedValue, EvalBreak>`. Any other return type is rejected with a `compile_error!`
/// naming the two accepted shapes — never silently guessed.
fn sniff_return(item: &ItemFn) -> syn::Result<SniffedReturn> {
    let ReturnType::Type(_, ty) = &item.sig.output else {
        return Err(Error::new_spanned(
            &item.sig,
            "wat_intrinsic: handler must return `Result<Value, EvalBreak>` or \
             `Result<TrackedValue, EvalBreak>`",
        ));
    };
    if is_result_of(ty, "TrackedValue") {
        Ok(SniffedReturn::Tracked)
    } else if is_result_of(ty, "Value") {
        Ok(SniffedReturn::BareValue)
    } else {
        Err(Error::new_spanned(
            ty,
            "wat_intrinsic: handler must return `Result<Value, EvalBreak>` or \
             `Result<TrackedValue, EvalBreak>` — got a different Ok type",
        ))
    }
}

/// Is `ty` shaped `Result<Ok = name, _>`? (Tolerates a preceding module path on `Result`
/// itself, e.g. `std::result::Result`; the Ok type is matched by its final path segment,
/// same tolerance as `type_path_ends_with`.)
fn is_result_of(ty: &Type, name: &str) -> bool {
    let Type::Path(p) = ty else { return false };
    let Some(seg) = p.path.segments.last() else { return false };
    if seg.ident != "Result" {
        return false;
    }
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return false;
    };
    match args.args.first() {
        Some(GenericArgument::Type(t)) => type_path_ends_with(t, name),
        _ => false,
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

/// Is the type `&Value`? Mirrors `is_ref_watast` — arc 255 Stone O-iii's third sniff, on the
/// same mechanism, checking for the ALGEBRA shape instead of the BINDING one.
fn is_ref_value(ty: &Type) -> bool {
    if let Type::Reference(r) = ty {
        return type_path_ends_with(&r.elem, "Value");
    }
    false
}

/// Is the type `&[Value]`? Mirrors `is_ref_watast_slice`.
fn is_ref_value_slice(ty: &Type) -> bool {
    if let Type::Reference(r) = ty {
        if let Type::Slice(s) = &*r.elem {
            return type_path_ends_with(&s.elem, "Value");
        }
    }
    false
}

/// Is the type `&Span`? Arc 255 Stone Q — the ONE thing an ALGEBRA handler may take after its
/// leading `&Value`/`&[Value]` run (STOP-1 still forbids `env`/`sym`; a span is not binding
/// state). Mirrors `is_ref_value`.
fn is_ref_span(ty: &Type) -> bool {
    if let Type::Reference(r) = ty {
        return type_path_ends_with(&r.elem, "Span");
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

pub(crate) fn emit(
    fqdn: &LitStr,
    value_fn: Option<&syn::Path>,
    item: &ItemFn,
) -> syn::Result<TokenStream2> {
    let kind = sniff_kind(item)?;
    let sniffed_return = sniff_return(item)?;

    // arc 255 Stone O-iii, STOP-2 — an ALGEBRA handler cannot stamp provenance: `ValueHandler`
    // returns a bare `Value`, so a `TrackedValue` return could not survive the value door. Ruled
    // out by the design's first affirmative cut, not silently dropped or half-supported.
    if let (IntrinsicKind::Algebra(_, _), SniffedReturn::Tracked) = (&kind, &sniffed_return) {
        return Err(Error::new_spanned(
            &item.sig,
            "wat_intrinsic: an ALGEBRA handler (leading `&Value`/`&[Value]` params) cannot \
             return `Result<TrackedValue, EvalBreak>` — `ValueHandler` returns a bare `Value`, \
             so a provenance stamp could not survive the value door. This handler is a \
             provenance-stamping handler and is BINDING by construction: give it `&WatAST` \
             leading params instead.",
        ));
    }

    // arc 255 Stone O-iii — an ALGEBRA handler cannot ALSO name `value = <path>`: the handler
    // itself becomes the value door (the macro generates it), so a hand-named one would leave
    // two candidates and no rule for which wins.
    if matches!(kind, IntrinsicKind::Algebra(_, _)) && value_fn.is_some() {
        return Err(Error::new_spanned(
            item,
            "wat_intrinsic: an ALGEBRA handler (leading `&Value`/`&[Value]` params) cannot also \
             name `value = <path>` — the handler itself IS the value door; the macro generates \
             the AST door from it. Drop `value = <path>`.",
        ));
    }

    let sniffed: &SniffedArgs = match &kind {
        IntrinsicKind::Binding(s) | IntrinsicKind::Algebra(s, _) => s,
    };

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
    let (arg_names, is_variadic): (Vec<String>, bool) = match sniffed {
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
        wat_doc::Category::Projection => quote! { ::wat_doc::Category::Projection },
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

    // Wrap the raw handler call per the sniffed return shape (arc 255 Stone G): a bare-`Value`
    // handler's `Ok` is lifted to `TrackedValue::new(v, Provenance::Unknown)` — today's
    // behaviour, unchanged; a `TrackedValue`-returning handler's `Ok` passes through
    // un-rewrapped, carrying whatever `Provenance` it stamped (e.g. `RuntimeBuilt`).
    let wrap_call = |call: TokenStream2| -> TokenStream2 {
        match sniffed_return {
            SniffedReturn::BareValue => quote! {
                #call.map(::wat::value::TrackedValue::from)
            },
            SniffedReturn::Tracked => call,
        }
    };

    // Build the shim body. For exact-arity: check len == N, then forward individual refs.
    // For variadic: pass the whole slice directly (no arity check — 0+ args all valid).
    // arc 255 Stone O-iii — BINDING builds today's shim exactly as before (this whole branch is
    // byte-for-byte what `emit` always did); ALGEBRA additionally generates the value door
    // (`value_door_tokens`) and points `value_handler_field` at it instead of at `value_fn`.
    let value_door_ident = format_ident!("__wat_intrinsic_value_{}", fn_name);
    let (shim_body, value_door_tokens, value_handler_field) = match &kind {
        IntrinsicKind::Binding(_) => {
            let body = if is_variadic {
                // Variadic: pass the whole slice to the handler.
                wrap_call(quote! {
                    #fn_name(args, env, sym, list_span)
                })
            } else {
                let n = arg_names.len();
                let arg_forwards: Vec<TokenStream2> = (0..n).map(|i| quote! { &args[#i] }).collect();
                let call = wrap_call(quote! {
                    #fn_name(#(#arg_forwards,)* env, sym, list_span)
                });
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
                    #call
                }
            };
            // Arc 255 Stone N — `value_handler` slot. `None` for every call site that doesn't
            // name a `value = <path>` (the ~250 pre-existing handlers, untouched — STOP-1);
            // `Some(<path>)` for the ones that do. The macro does not inspect `<path>`'s
            // signature — a mismatch against `fn(&[Value]) -> Result<Value, EvalBreak>` surfaces
            // as an ordinary Rust type error at the `IntrinsicSubmission` struct literal below.
            let vh_field = match value_fn {
                Some(path) => quote! { ::std::option::Option::Some(#path) },
                None => quote! { ::std::option::Option::None },
            };
            (body, TokenStream2::new(), vh_field)
        }
        IntrinsicKind::Algebra(_, wants_span) => {
            // Arc 255 Stone Q — the door's OWN param is always named `_span`: `ValueHandler`
            // fixes the fn-pointer shape to `fn(&[Value], &Span) -> …`, so every door takes
            // one, but only a handler that itself declared a trailing `&Span` (`wants_span`)
            // gets it forwarded. The underscore keeps an unused-param warning off the 38
            // already-migrated (span-free) verbs' generated doors without an `#[allow]`.
            let span_forward: Option<TokenStream2> = if *wants_span {
                Some(quote! { , _span })
            } else {
                None
            };

            // The value door: what `apply` reaches through `dispatch_substrate_impl`. Arity is
            // guarded HERE too (not only on the AST door below) — STOP triggers 3 and 5: the
            // adapter must be correct standing alone, raising the exact same `ArityMismatch`
            // shape (same op string, same expected/got) as the AST door and as
            // `dispatch_substrate_impl`'s own central guard (arc 255 Stone O-i).
            let value_door_body = if is_variadic {
                quote! { #fn_name(vals #span_forward) }
            } else {
                let n = arg_names.len();
                // Build the full comma-joined arg list (value forwards, then `_span` if
                // wanted) as ONE list — not `#(#val_forwards),* #span_forward`, which emits a
                // dangling leading comma when `n == 0` (a nullary ALGEBRA fn taking only a
                // span is a legal, if unusual, shape).
                let mut call_args: Vec<TokenStream2> =
                    (0..n).map(|i| quote! { &vals[#i] }).collect();
                if *wants_span {
                    call_args.push(quote! { _span });
                }
                quote! {
                    if vals.len() != #n {
                        return ::std::result::Result::Err(
                            ::wat::value::RuntimeError::new(::wat::rust_caller_span!(), ::wat::value::RuntimeErrorKind::ArityMismatch {
                                    op: #fqdn.into(),
                                    expected: #n,
                                    got: vals.len(),
                                })
                            .into(),
                        );
                    }
                    #fn_name(#(#call_args),*)
                }
            };
            let door_tokens = quote! {
                // arc 255 Stone O-iii/Q — the value door, generated from the ALGEBRA declaration
                // itself. What `:wat::core::apply` reaches through `dispatch_substrate_impl`. The
                // trailing `&Span` is the call's own (Stone Q); forwarded to `#fn_name` only when
                // the handler declared it.
                fn #value_door_ident(
                    vals: &[::wat::value::Value],
                    _span: &::wat::span::Span,
                ) -> ::std::result::Result<::wat::value::Value, ::wat::value::EvalBreak> {
                    #value_door_body
                }
            };

            // The AST door: eval each arg to an owned `Value`, then reuse the value door —
            // one implementation, not two. Arity is checked HERE too, before any arg is
            // evaluated (so a wrong-arity call fails fast, exactly like BINDING's shim), and
            // AGAIN in the value door (STOP-3/5) — same `ArityMismatch` shape both times.
            // Arc 255 Stone Q — passes `list_span`, the shim's own real call span, not a
            // synthesized one (both doors now see the SAME span for the SAME call).
            let ast_body = if is_variadic {
                quote! {
                    let vals: ::std::vec::Vec<::wat::value::Value> = args.iter()
                        .map(|a| ::wat::runtime::eval_inner(a, env, sym).map(::wat::value::TrackedValue::value_owned))
                        .collect::<::std::result::Result<_, _>>()?;
                    #value_door_ident(&vals, list_span).map(::wat::value::TrackedValue::from)
                }
            } else {
                let n = arg_names.len();
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
                    let vals: ::std::vec::Vec<::wat::value::Value> = args.iter()
                        .map(|a| ::wat::runtime::eval_inner(a, env, sym).map(::wat::value::TrackedValue::value_owned))
                        .collect::<::std::result::Result<_, _>>()?;
                    #value_door_ident(&vals, list_span).map(::wat::value::TrackedValue::from)
                }
            };
            let vh_field = quote! { ::std::option::Option::Some(#value_door_ident) };
            (ast_body, door_tokens, vh_field)
        }
    };

    let expanded = quote! {
        // The annotated handler, passed through unchanged.
        #item

        // arc 255 Stone O-iii — the generated value door (ALGEBRA only; empty for BINDING).
        #value_door_tokens

        // Dispatch shim — canonical NativeHandler signature. Returns `TrackedValue`
        // (arc 255 Stone G): a bare-`Value` handler is wrapped as `Provenance::Unknown`
        // by `wrap_call` above; a `TrackedValue`-returning handler's own provenance survives.
        fn #shim_ident(
            args: &[::wat::ast::WatAST],
            list_span: &::wat::span::Span,
            env: &::wat::value::Environment,
            sym: &::wat::value::SymbolTable,
        ) -> ::std::result::Result<::wat::value::TrackedValue, ::wat::value::EvalBreak> {
            #shim_body
        }

        // Auto-collect: link-time registration of (fqdn → shim) into the
        // IntrinsicRegistry. `registry()` iterates these submissions.
        ::inventory::submit! {
            ::wat::intrinsic::IntrinsicSubmission {
                name: #fqdn,
                handler: #shim_ident,
                value_handler: #value_handler_field,
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
