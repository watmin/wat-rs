use crate::ast::WatAST;
use crate::scope::Identifier;
use crate::span::Span;
use crate::types::{parse_type_node, TypeExpr};
use super::error::{ArgSpecError, ArgSpecErrorKind};

/// Result of parsing a canonical `[name <- :T name <- :T ... [& rest <- :T]]` argspec.
///
/// `fixed_params` is ordered (left-to-right from the source form).
/// `rest_param` is `None` unless `options.allow_rest_binder = true` AND the source
/// includes `& name <- :T`. Otherwise `None`.
/// Ret-clause (`-> :Ret`) is NOT represented here — fn-form parsers (defn, fn, etc.)
/// compose argspec + ret-clause at the form level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgSpec {
    /// Ordered list of `(name, type)` pairs for the fixed positional parameters.
    pub fixed_params: Vec<(Identifier, TypeExpr)>,
    /// Rest parameter `(name, type)`, populated when `options.allow_rest_binder = true`
    /// AND the source includes `& name <- :T`. Otherwise `None`.
    pub rest_param: Option<(Identifier, TypeExpr)>,
}

/// Per-site invariants for `parse_argspec_triples`.
///
/// Each binding site (defn, defclause) passes its own `ParseOptions` to
/// express the structural invariants that differ across sites without
/// duplicating the parser walker itself.
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// Whether a `& name <- :T` rest-binder is permitted in the arg-vector.
    ///
    /// When `true`, the canonical parser parses `& name <- :T` as a rest-binder,
    /// populating `ArgSpec.rest_param`. When `false`, encountering `&` returns
    /// `ArgSpecError::RestBinderNotSupported`. `defclause` sites set this `true`;
    /// `defn`/fn-form sites set this `false`.
    pub allow_rest_binder: bool,
}

/// Parse the canonical `[name <- :T name <- :T ... [& rest <- :T]]` argspec form.
///
/// Scope: argspec parses ONLY the canonical triple region. Ret-clause (`-> :Ret`)
/// is NOT argspec's concern — fn-form parsers (defn, fn, etc.) split the form-level
/// Vector at `->` and compose argspec + ret-clause parsing at the form level.
///
/// # Parameters
///
/// - `args_vec` — the inner items of a `WatAST::Vector` at the binding site.
///   Callers extract the items by matching `WatAST::Vector(items, _)` before
///   calling this parser; this function receives the already-extracted slice.
///   For fn-form callers: pass only the argspec prefix (items BEFORE `->` split).
/// - `head` — the surface form name for error context (e.g. `":wat::core::defn"`).
/// - `form_span` — the `Vector`'s own span; used as fallback in error variants
///   where no more-specific offending-element span is available.
/// - `options` — per-site invariants (allow_rest_binder).
///
/// # Returns
///
/// `Ok(ArgSpec)` on success; `Err(ArgSpecError)` on any structural violation.
/// Callers convert `ArgSpecError` to their site's native error class via
/// `ArgSpecError::into()` (the four `From<>` impls in `error.rs`).
///
/// # Algorithm
///
/// 1. Walk `args_vec` in triple chunks of 3 (`name <- :T`), stopping when
///    `&` (rest-marker) is encountered or the slice is exhausted.
/// 2. On `&` (rest-marker): if `options.allow_rest_binder`, parse the following
///    triple as the rest-binder and verify no trailing items; else return
///    `RestBinderNotSupported`.
/// 3. On exhaustion with no `&`: return `ArgSpec` with `rest_param: None`.
pub fn parse_argspec_triples(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
    options: ParseOptions,
) -> Result<ArgSpec, ArgSpecError> {
    let mut cursor = 0usize;
    let mut fixed_params: Vec<(Identifier, TypeExpr)> = Vec::new();

    // Walk triples (name <- :T) until `&` rest-marker or end-of-slice.
    while cursor < args_vec.len() {
        // Check for `&` rest-marker at this position.
        if args_vec[cursor].is_bare_symbol("&") {
            if !options.allow_rest_binder {
                return Err(ArgSpecError {
                    span: args_vec[cursor].span().clone(),
                    head: head.to_string(),
                    kind: ArgSpecErrorKind::RestBinderNotSupported,
                });
            }
            cursor += 1; // consume `&`
            let rest_start = cursor;
            let triple = extract_triple(args_vec, rest_start, form_span, head)?;
            let (name, ty) = parse_triple(triple, head)?;
            let trailing_start = rest_start + 3;
            if trailing_start < args_vec.len() {
                let count = args_vec.len() - trailing_start;
                return Err(ArgSpecError {
                    span: args_vec[trailing_start].span().clone(),
                    head: head.to_string(),
                    kind: ArgSpecErrorKind::TrailingItems { count },
                });
            }
            return Ok(ArgSpec {
                fixed_params,
                rest_param: Some((name, ty)),
            });
        }

        let triple = extract_triple(args_vec, cursor, form_span, head)?;
        let (name, ty) = parse_triple(triple, head)?;
        fixed_params.push((name, ty));
        cursor += 3;
    }

    Ok(ArgSpec {
        fixed_params,
        rest_param: None,
    })
}

/// Extract a `&[WatAST; 3]` slice starting at `start` in `args_vec`.
///
/// Returns `Err(IncompleteTriple)` when fewer than 3 items remain. The span
/// attributed to the error is `args_vec[start]`'s span when an element is
/// present, otherwise `fallback_span` (the enclosing form's span).
/// The `.expect()` invariant is guaranteed by the `< 3` guard.
fn extract_triple<'a>(
    args_vec: &'a [WatAST],
    start: usize,
    fallback_span: &Span,
    head: &str,
) -> Result<&'a [WatAST; 3], ArgSpecError> {
    if args_vec.len().saturating_sub(start) < 3 {
        let span = if start < args_vec.len() {
            args_vec[start].span().clone()
        } else {
            fallback_span.clone()
        };
        return Err(ArgSpecError {
            span,
            head: head.to_string(),
            kind: ArgSpecErrorKind::IncompleteTriple,
        });
    }
    Ok(args_vec[start..start + 3].try_into().expect("len gated by the `< 3` check above"))
}

/// Parse a single `name <- :T` triple. The `&[WatAST; 3]` type makes the
/// length precondition structural — `extract_triple` performs the `try_into`
/// before handing the fixed-size reference here.
/// Returns `(name, ty)` on success; the relevant `ArgSpecError` variant on
/// per-slot failures (NameNotSymbol, MissingArrow, TypeNotKeyword,
/// MalformedTypeKeyword via parse_keyword_type).
fn parse_triple(
    triple: &[WatAST; 3],
    head: &str,
) -> Result<(Identifier, TypeExpr), ArgSpecError> {
    let name = match &triple[0] {
        WatAST::Symbol(ident, _) => ident.clone(),
        other => return Err(ArgSpecError {
            span: other.span().clone(),
            head: head.to_string(),
            kind: ArgSpecErrorKind::NameNotSymbol,
        }),
    };
    // Arc 251.4a — accept the `:-` annotation keyword (core.typed parity) as a
    // dual-read alias for the legacy `<-` binder arrow. The `<-` arrow HARD-CUTs at
    // 251.5 (the corpus `<-`→`:-` sweep rides the unified 251.5 sweep).
    let is_annotation_arrow =
        triple[1].is_bare_symbol("<-") || crate::types::is_binder_marker(&triple[1]);
    if !is_annotation_arrow {
        return Err(ArgSpecError {
            span: triple[1].span().clone(),
            head: head.to_string(),
            kind: ArgSpecErrorKind::MissingArrow,
        });
    }
    let ty = parse_keyword_type(&triple[2], head)?;
    Ok((name, ty))
}

/// Parse a type-annotation slot — the shared logic for fixed-param slot 2 and the
/// rest-binder type slot.
///
/// Arc 251.3a — accepts three node shapes:
/// - `WatAST::Keyword` — the existing surface; delegates to `parse_type_expr_with_span`.
/// - `WatAST::Symbol` — pre-normalize `wat.type/X` atom; delegates to `parse_type_node`.
/// - `WatAST::List` — parametric-type FORM `(wat.type/Vector wat.type/i64)`; delegates to
///   `parse_type_node` → `parse_type_form`.
///
/// Any other form returns `TypeNotKeyword`.
fn parse_keyword_type(
    ast: &WatAST,
    head: &str,
) -> Result<TypeExpr, ArgSpecError> {
    match ast {
        WatAST::Keyword(_, _) | WatAST::Symbol(_, _) | WatAST::List(_, _) | WatAST::Vector(_, _) => {
            parse_type_node(ast).map_err(|inner| {
                ArgSpecError {
                    span: ast.span().clone(),
                    head: head.to_string(),
                    kind: ArgSpecErrorKind::MalformedTypeKeyword { inner: Box::new(inner.into_kind()) },
                }
            })
        }
        other => Err(ArgSpecError {
            span: other.span().clone(),
            head: head.to_string(),
            kind: ArgSpecErrorKind::TypeNotKeyword,
        }),
    }
}

