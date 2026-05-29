use crate::ast::WatAST;
use crate::span::Span;
use crate::types::{parse_type_expr_with_span, TypeExpr};
use super::error::ArgSpecError;

/// Result of parsing a canonical `[name <- :T name <- :T ... [& rest <- :T]]` argspec.
///
/// `fixed_params` is ordered (left-to-right from the source form).
/// `rest_param` is `None` in 241.1 (rest-binder support is Stone 241.4).
/// Ret-clause (`-> :Ret`) is NOT represented here — fn-form parsers (defn, fn, etc.)
/// compose argspec + ret-clause at the form level.
#[derive(Debug, Clone)]
pub struct ArgSpec {
    /// Ordered list of `(name, type)` pairs for the fixed positional parameters.
    pub fixed_params: Vec<(String, TypeExpr)>,
    /// Rest parameter `(name, type)`, populated by Stone 241.4.
    /// Always `None` in Stone 241.1.
    // rune:purgare(future-fixture) — Stone 241.4 populates rest_param via allow_rest_binder
    //                                path; field exists in 241.1 for API stability.
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
    /// Stone 241.4 wires this field: when `true`, the rest-binder is parsed;
    /// when `false`, `ArgSpecError::RestBinderNotSupported` is returned.
    /// In Stone 241.1 the field is not consulted — `&` is always rejected
    /// unconditionally, keeping the panic-free `Result` contract honest.
    /// `defclause` callers set this `true` via Stone 241.5.
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
/// 2. Always reject `&` in Stone 241.1. Stone 241.4 wires `options.allow_rest_binder` to
///    permit rest-binder parsing; 241.1 unconditionally returns `RestBinderNotSupported`.
pub fn parse_argspec_triples(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
    _options: ParseOptions,
) -> Result<ArgSpec, ArgSpecError> {
    let mut idx = 0usize;
    let mut fixed_params: Vec<(String, TypeExpr)> = Vec::new();

    // Walk triples (name <- :T) until `&` rest-marker or end-of-slice.
    while idx < args_vec.len() {
        // Check for `&` rest-marker at this position.
        if is_bare_symbol(&args_vec[idx], "&") {
            // 241.1: always reject. Stone 241.4 wires options.allow_rest_binder to permit
            // rest-binder parsing; until then `&` is always an error regardless of the option.
            return Err(ArgSpecError::RestBinderNotSupported {
                span: args_vec[idx].span().clone(),
                head: head.to_string(),
            });
        }

        if args_vec.len().saturating_sub(idx) < 3 {
            return Err(ArgSpecError::IncompleteTriple {
                span: form_span.clone(),
                head: head.to_string(),
            });
        }

        // Slot 0: name — must be a Symbol (binding contract: arc 159/169/234).
        let name = match &args_vec[idx] {
            WatAST::Symbol(ident, _) => ident.name.clone(),
            other => {
                return Err(ArgSpecError::NameNotSymbol {
                    span: other.span().clone(),
                    head: head.to_string(),
                })
            }
        };

        // Slot 1: arrow — must be bare Symbol "<-".
        if !is_bare_symbol(&args_vec[idx + 1], "<-") {
            return Err(ArgSpecError::MissingArrow {
                span: args_vec[idx + 1].span().clone(),
                head: head.to_string(),
            });
        }

        // Slot 2: type — route through parse_keyword_type with the fixed-param error ctor.
        let ty = parse_keyword_type(&args_vec[idx + 2], head, |span, head| {
            ArgSpecError::TypeNotKeyword { span, head }
        })?;

        fixed_params.push((name, ty));
        idx += 3;
    }

    Ok(ArgSpec {
        fixed_params,
        // rune:purgare(future-fixture) — Stone 241.4 populates rest_param via
        //                                allow_rest_binder path; 241.1 always None.
        rest_param: None,
    })
}

/// Parse a type-keyword slot — the shared logic for fixed-param slot 2 (and, when
/// Stone 241.4 ships, the rest-binder type slot). If `ast` is a `Keyword`, delegates
/// to `parse_type_expr_with_span` and wraps parse failures as `MalformedTypeKeyword`.
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
