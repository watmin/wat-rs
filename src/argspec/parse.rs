use crate::ast::WatAST;
use crate::span::Span;
use crate::types::{parse_type_expr_with_span, TypeExpr};
use super::error::ArgSpecError;

/// Result of parsing a canonical `[name <- :T name <- :T ... [& rest <- :T]]` argspec.
///
/// `fixed_params` is ordered (left-to-right from the source form).
/// `rest_param` is `None` unless `options.allow_rest_binder = true` AND the source
/// includes `& name <- :T`. Otherwise `None`.
/// Ret-clause (`-> :Ret`) is NOT represented here — fn-form parsers (defn, fn, etc.)
/// compose argspec + ret-clause at the form level.
#[derive(Debug, Clone)]
pub struct ArgSpec {
    /// Ordered list of `(name, type)` pairs for the fixed positional parameters.
    pub fixed_params: Vec<(String, TypeExpr)>,
    /// Rest parameter `(name, type)`, populated when `options.allow_rest_binder = true`
    /// AND the source includes `& name <- :T`. Otherwise `None`.
    pub rest_param: Option<(String, TypeExpr)>,
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
/// `ArgSpecError::into()` (the three `From<>` impls in `error.rs`).
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
    let mut idx = 0usize;
    let mut fixed_params: Vec<(String, TypeExpr)> = Vec::new();

    // Walk triples (name <- :T) until `&` rest-marker or end-of-slice.
    while idx < args_vec.len() {
        // Check for `&` rest-marker at this position.
        if is_bare_symbol(&args_vec[idx], "&") {
            if !options.allow_rest_binder {
                return Err(ArgSpecError::RestBinderNotSupported {
                    span: args_vec[idx].span().clone(),
                    head: head.to_string(),
                });
            }
            idx += 1; // consume `&`
            let rest_start = idx;
            if args_vec.len().saturating_sub(rest_start) < 3 {
                return Err(ArgSpecError::IncompleteTriple {
                    span: form_span.clone(),
                    head: head.to_string(),
                });
            }
            let triple: &[WatAST; 3] = args_vec[rest_start..rest_start + 3]
                .try_into()
                .expect("len gated by upstream `< 3` check");
            let (name, ty) = parse_triple(triple, head)?;
            let post_rest = rest_start + 3;
            if post_rest < args_vec.len() {
                return Err(ArgSpecError::TrailingItems {
                    span: form_span.clone(),
                    head: head.to_string(),
                    count: args_vec.len() - post_rest,
                });
            }
            return Ok(ArgSpec {
                fixed_params,
                rest_param: Some((name, ty)),
            });
        }

        if args_vec.len().saturating_sub(idx) < 3 {
            return Err(ArgSpecError::IncompleteTriple {
                span: form_span.clone(),
                head: head.to_string(),
            });
        }

        let triple: &[WatAST; 3] = args_vec[idx..idx + 3]
            .try_into()
            .expect("len gated by upstream `< 3` check");
        let (name, ty) = parse_triple(triple, head)?;
        fixed_params.push((name, ty));
        idx += 3;
    }

    Ok(ArgSpec {
        fixed_params,
        rest_param: None,
    })
}

/// Parse a single `name <- :T` triple. The `&[WatAST; 3]` type enforces the
/// length precondition at the call site — callers convert via `try_into()`.
/// Returns `(name, ty)` on success; the relevant `ArgSpecError` variant on
/// per-slot failures (NameNotSymbol, MissingArrow, TypeNotKeyword,
/// MalformedTypeKeyword via parse_keyword_type).
fn parse_triple(
    triple: &[WatAST; 3],
    head: &str,
) -> Result<(String, TypeExpr), ArgSpecError> {
    let name = match &triple[0] {
        WatAST::Symbol(ident, _) => ident.name.clone(),
        other => return Err(ArgSpecError::NameNotSymbol {
            span: other.span().clone(),
            head: head.to_string(),
        }),
    };
    if !is_bare_symbol(&triple[1], "<-") {
        return Err(ArgSpecError::MissingArrow {
            span: triple[1].span().clone(),
            head: head.to_string(),
        });
    }
    let ty = parse_keyword_type(&triple[2], head, |span, head| {
        ArgSpecError::TypeNotKeyword { span, head }
    })?;
    Ok((name, ty))
}

/// Parse a type-keyword slot — the shared logic for fixed-param slot 2 and the
/// rest-binder type slot. If `ast` is a `Keyword`, delegates to
/// `parse_type_expr_with_span` and wraps parse failures as `MalformedTypeKeyword`.
/// If `ast` is any other form, calls `non_keyword_err` to produce the caller's
/// non-keyword variant.
fn parse_keyword_type<F>(
    ast: &WatAST,
    head: &str,
    non_keyword_err: F,
) -> Result<TypeExpr, ArgSpecError>
where
    F: FnOnce(Span, String) -> ArgSpecError,
{
    match ast {
        WatAST::Keyword(kw, kw_span) => {
            parse_type_expr_with_span(kw, kw_span).map_err(|inner| {
                ArgSpecError::MalformedTypeKeyword {
                    span: kw_span.clone(),
                    head: head.to_string(),
                    inner: Box::new(inner),
                }
            })
        }
        other => Err(non_keyword_err(other.span().clone(), head.to_string())),
    }
}

/// Returns `true` if `ast` is a bare `Symbol` whose name equals `name`.
///
/// Used to detect the structural tokens `"<-"`, `"->"`, and `"&"` without
/// allocating or cloning.
fn is_bare_symbol(ast: &WatAST, name: &str) -> bool {
    matches!(ast, WatAST::Symbol(ident, _) if ident.name == name)
}
