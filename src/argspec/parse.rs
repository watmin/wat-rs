use crate::ast::WatAST;
use crate::span::Span;
use crate::types::{parse_type_expr_with_span, TypeExpr};
use super::error::ArgSpecError;

/// Result of parsing a canonical `[name <- :T name <- :T ... [-> :Ret]]` argspec.
///
/// `fixed_params` is ordered (left-to-right from the source form).
/// `rest_param` is `None` in 241.1 (rest-binder support is Stone 241.4).
/// `ret_type` is `None` when `ParseOptions.include_ret_type = false`.
#[derive(Debug, Clone)]
pub struct ArgSpec {
    /// Ordered list of `(name, type)` pairs for the fixed positional parameters.
    pub fixed_params: Vec<(String, TypeExpr)>,
    /// Rest parameter `(name, type)`, populated by Stone 241.4.
    /// Always `None` in Stone 241.1.
    pub rest_param: Option<(String, TypeExpr)>,
    /// Return type, populated when `ParseOptions.include_ret_type = true`.
    /// `None` for binding sites that have no return type (e.g. defclause).
    pub ret_type: Option<TypeExpr>,
}

/// Per-site invariants for `parse_argspec_triples`.
///
/// Each binding site (defn, defclause) passes its own `ParseOptions` to
/// express the structural invariants that differ across sites without
/// duplicating the parser walker itself.
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    /// Whether a `-> :RetType` slot is expected after the fixed-param triples.
    ///
    /// `true` for fn / defn forms (A1/A2/A3 sites); `false` for defclause
    /// forms (A4 site) where no return type appears in the arg-vector.
    pub include_ret_type: bool,
    /// Whether a `& name <- :T` rest-binder is permitted in the arg-vector.
    ///
    /// Always `false` in Stone 241.1. Stone 241.4 adds rest-binder logic;
    /// `defclause` callers set this `true` via 241.5. Reject with
    /// `ArgSpecError::RestBinderNotSupported` when `false` and `&` is seen.
    pub allow_rest_binder: bool,
}

/// Parse the canonical `[name <- :T name <- :T ... [-> :Ret]]` argspec form.
///
/// # Parameters
///
/// - `args_vec` — the inner items of a `WatAST::Vector` at the binding site.
///   Callers extract the items by matching `WatAST::Vector(items, _)` before
///   calling this parser; this function receives the already-extracted slice.
/// - `head` — the surface form name for error context (e.g. `":wat::core::defn"`).
/// - `form_span` — the `Vector`'s own span; used as fallback in error variants
///   where no more-specific offending-element span is available.
/// - `options` — per-site invariants (include_ret_type, allow_rest_binder).
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
///    `->` (ret-arrow) or `&` (rest-marker) is encountered.
/// 2. After fixed-param triples, if `options.include_ret_type`: consume `->` +
///    the ret-type keyword.
/// 3. Reject trailing items beyond the expected shape.
pub fn parse_argspec_triples(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
    options: ParseOptions,
) -> Result<ArgSpec, ArgSpecError> {
    let mut idx = 0usize;
    let mut fixed_params: Vec<(String, TypeExpr)> = Vec::new();

    // Walk triples (name <- :T) until we hit -> (if include_ret_type), & (rest), or end.
    while idx < args_vec.len() {
        // Check for `&` rest-marker at this position.
        if is_bare_symbol(&args_vec[idx], "&") {
            if !options.allow_rest_binder {
                return Err(ArgSpecError::RestBinderNotSupported {
                    span: args_vec[idx].span().clone(),
                    head: head.to_string(),
                });
            }
            // Stone 241.4 implements rest-binder parsing here.
            // allow_rest_binder is always false in 241.1, so this branch is
            // unreachable at this stone. The field exists so the API is stable.
            unreachable!("allow_rest_binder is always false in Stone 241.1");
        }

        // Check for `->` ret-arrow — if so, stop fixed-param parsing.
        if is_bare_symbol(&args_vec[idx], "->") {
            break;
        }

        // Need 3 items for a complete triple; check before indexing.
        if idx + 2 >= args_vec.len() {
            return Err(ArgSpecError::IncompleteSignature {
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

        // Slot 2: type — must be a Keyword parsed by the canonical type parser.
        let ty = match &args_vec[idx + 2] {
            WatAST::Keyword(kw, kw_span) => {
                parse_type_expr_with_span(kw, kw_span).map_err(|inner| {
                    ArgSpecError::MalformedTypeKeyword {
                        span: kw_span.clone(),
                        head: head.to_string(),
                        inner: Box::new(inner),
                    }
                })?
            }
            other => {
                return Err(ArgSpecError::TypeNotKeyword {
                    span: other.span().clone(),
                    head: head.to_string(),
                })
            }
        };

        fixed_params.push((name, ty));
        idx += 3;
    }

    // Handle the ret-type slot when the binding site requires it.
    let ret_type = if options.include_ret_type {
        // Expect `->` now — either we hit it in the loop above (idx points at it)
        // or we've consumed all items without finding it.
        if idx >= args_vec.len() {
            return Err(ArgSpecError::MissingRetArrow {
                span: form_span.clone(),
                head: head.to_string(),
            });
        }
        if !is_bare_symbol(&args_vec[idx], "->") {
            return Err(ArgSpecError::MissingRetArrow {
                span: args_vec[idx].span().clone(),
                head: head.to_string(),
            });
        }
        idx += 1; // consume `->`

        // Ret-type keyword.
        if idx >= args_vec.len() {
            return Err(ArgSpecError::RetTypeNotKeyword {
                span: form_span.clone(),
                head: head.to_string(),
            });
        }
        let ret = match &args_vec[idx] {
            WatAST::Keyword(kw, kw_span) => {
                parse_type_expr_with_span(kw, kw_span).map_err(|inner| {
                    ArgSpecError::MalformedTypeKeyword {
                        span: kw_span.clone(),
                        head: head.to_string(),
                        inner: Box::new(inner),
                    }
                })?
            }
            other => {
                return Err(ArgSpecError::RetTypeNotKeyword {
                    span: other.span().clone(),
                    head: head.to_string(),
                })
            }
        };
        idx += 1;
        Some(ret)
    } else {
        None
    };

    // Reject trailing items beyond the expected shape.
    if idx < args_vec.len() {
        return Err(ArgSpecError::TrailingItems {
            span: form_span.clone(),
            head: head.to_string(),
            count: args_vec.len() - idx,
        });
    }

    Ok(ArgSpec {
        fixed_params,
        // rest_param is always None in 241.1; Stone 241.4 extends this.
        rest_param: None,
        ret_type,
    })
}

/// Returns `true` if `ast` is a bare `Symbol` whose name equals `name`.
///
/// Used to detect the structural tokens `"<-"`, `"->"`, and `"&"` without
/// allocating or cloning.
fn is_bare_symbol(ast: &WatAST, name: &str) -> bool {
    matches!(ast, WatAST::Symbol(ident, _) if ident.name == name)
}
