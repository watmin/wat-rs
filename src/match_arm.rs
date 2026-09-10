//! Arc 109 / 296: a match arm is a bracket clause. The head is the
//! context-free discriminator:
//!
//! ```text
//! [_ body]                         wildcard        2 elements
//! [<bare-symbol> body]             binding         2 elements
//! [<literal> body]                 literal         2 elements (int/float/bool/
//!                                                  string/rational/bigint/char)
//! [<Variant> <map-pattern> body]   variant         3 elements, namespaced head
//! [{var :field …} body]            hash-destructure of a record/map (existing
//!                                  Open-shape arm; delimiter flip only)
//! ```
//!
//! A List arm is the retired `(pattern body)` clause and is refused.
//! The variant map is KEY-FIRST (`{:field binder}`), the pattern order —
//! it is a different operation from `let`'s binder-first `{var :field}`
//! (arc 257.2). Do not route the variant map through
//! `WatAST::classify_map_destructure`.

use crate::ast::WatAST;
use crate::scope::Identifier;
use crate::span::Span;

#[derive(Debug)]
pub struct MatchArmError {
    pub span: Span,
    pub reason: String,
}

#[derive(Debug)]
pub enum MatchArm<'a> {
    Wildcard {
        body: &'a WatAST,
    },
    Binding {
        ident: &'a Identifier,
        body: &'a WatAST,
    },
    /// 2-element literal arm: `[0 false]`, `[true body]`, … — the delimiter
    /// flip of the retired `(0 false)` list arm. Same 2-element shape as
    /// wildcard/binder; the head is a literal, not a binder.
    Literal {
        pat: &'a WatAST,
        body: &'a WatAST,
    },
    /// Namespaced variant head + key-first map pattern + body.
    Variant {
        path: &'a str,
        path_span: &'a Span,
        pairs: &'a [(WatAST, WatAST)],
        body: &'a WatAST,
    },
    /// Existing Open-shape record/HashMap arm. Binder-first map, same
    /// operation as `let`. Not a variant map pattern.
    HashDestructure {
        pairs: &'a [(WatAST, WatAST)],
        body: &'a WatAST,
    },
}

impl<'a> MatchArm<'a> {
    pub fn body(&self) -> &'a WatAST {
        match self {
            MatchArm::Wildcard { body }
            | MatchArm::Binding { body, .. }
            | MatchArm::Literal { body, .. }
            | MatchArm::Variant { body, .. }
            | MatchArm::HashDestructure { body, .. } => body,
        }
    }
}

const RETIRED: &str = "retired `(pattern body)` clause; a match arm is a bracket: \
`[_ body]`, `[<binder> body]`, `[<literal> body]`, or `[<Variant> {:k v} body]`";

pub fn parse_match_arm(arm: &WatAST) -> Result<MatchArm<'_>, MatchArmError> {
    match arm {
        WatAST::List(_, span) => Err(MatchArmError {
            span: span.clone(),
            reason: RETIRED.into(),
        }),
        WatAST::Vector(items, span) => match items.as_slice() {
            [WatAST::Symbol(s, _), body] if s.as_str() == "_" => Ok(MatchArm::Wildcard { body }),
            [WatAST::Symbol(s, _), body] => Ok(MatchArm::Binding { ident: s, body }),
            [WatAST::Map(pairs, _), body] => Ok(MatchArm::HashDestructure { pairs, body }),
            [pat, body] if is_literal_ast(pat) => Ok(MatchArm::Literal { pat, body }),
            [WatAST::Keyword(k, path_span), WatAST::Map(pairs, _), body]
                if is_namespaced_variant(k) =>
            {
                if let Some(repl) = retired_bare_variant(k) {
                    return Err(MatchArmError {
                        span: path_span.clone(),
                        reason: format!(
                            "the bare variant spelling is retired; write `{repl}`"
                        ),
                    });
                }
                Ok(MatchArm::Variant {
                    path: k,
                    path_span,
                    pairs,
                    body,
                })
            }
            [WatAST::Keyword(k, path_span), WatAST::Map(_, _), _] => Err(MatchArmError {
                span: path_span.clone(),
                reason: format!(
                    "variant arm head `{k}` is not namespaced; write `<enum>::<Variant>`"
                ),
            }),
            _ => Err(MatchArmError {
                span: span.clone(),
                reason: format!(
                    "a match arm is `[_ body]`, `[<binder> body]`, `[<literal> body]`, \
                     `[<Variant> {{:k v}} body]`, or a record hash-destructure; \
                     got {} element(s)",
                    items.len()
                ),
            }),
        },
        other => Err(MatchArmError {
            span: other.span().clone(),
            reason: format!(
                "a match arm must be a vector, got {}",
                other.variant_name()
            ),
        }),
    }
}

/// A variant head is namespaced (`:enum::Variant`). Discriminator
/// between a variant map pattern and a non-variant keyword (refused).
pub fn is_namespaced_variant(path: &str) -> bool {
    wat_reader::identifier::decompose_variant(path).is_some()
}

/// Literal heads of a 2-element arm. Mirrors `try_match_pattern`'s
/// equality cases so parse and eval agree on what a literal is.
pub fn is_literal_ast(ast: &WatAST) -> bool {
    matches!(
        ast,
        WatAST::IntLit(_, _)
            | WatAST::FloatLit(_, _)
            | WatAST::BoolLit(_, _)
            | WatAST::StringLit(_, _)
            | WatAST::RationalLit(_, _)
            | WatAST::BigIntLit(_, _)
            | WatAST::CharLit(_, _)
    )
}

/// Built-in Option/Result variant heads. User enums compose
/// `type_path::variant_name` and do not go through this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinVariant {
    OptionSome,
    OptionNone,
    ResultOk,
    ResultErr,
}

pub fn builtin_variant(path: &str) -> Option<BuiltinVariant> {
    match path {
        ":wat::core::Option::Some" => Some(BuiltinVariant::OptionSome),
        ":wat::core::Option::None" => Some(BuiltinVariant::OptionNone),
        ":wat::core::Result::Ok" => Some(BuiltinVariant::ResultOk),
        ":wat::core::Result::Err" => Some(BuiltinVariant::ResultErr),
        _ => None,
    }
}

/// The five illegal two-segment (and legacy `:None`) spellings, with the
/// qualified Type::Variant replacement the refusal must name (STOP-3).
pub fn retired_bare_variant(path: &str) -> Option<&'static str> {
    match path {
        ":wat::core::Some" => Some(":wat::core::Option::Some"),
        ":None" | ":wat::core::None" => Some(":wat::core::Option::None"),
        ":wat::core::Ok" => Some(":wat::core::Result::Ok"),
        ":wat::core::Err" => Some(":wat::core::Result::Err"),
        _ => None,
    }
}

pub fn bare_variant_retired_reason(path: &str) -> String {
    match retired_bare_variant(path) {
        Some(repl) => format!("the bare variant spelling is retired; write `{repl}`"),
        None => format!("the bare variant spelling is retired; write a qualified Type::Variant FQDN, not `{path}`"),
    }
}

/// Key-first map pattern: each pair is `(Keyword :field, pattern)`.
/// Returns `(bare_field_name, value_pattern)` in source order.
pub fn parse_key_first_pairs<'a>(
    pairs: &'a [(WatAST, WatAST)],
    span: &Span,
) -> Result<Vec<(String, &'a WatAST)>, MatchArmError> {
    let mut out = Vec::with_capacity(pairs.len());
    for (k, v) in pairs {
        match k {
            WatAST::Keyword(kw, kspan) => {
                let name = kw.trim_start_matches(':');
                // rune:lint(one-variant-separator, namespace) — rejects a namespaced keyword used where a bare map-pattern field name is required; unrelated to enum variants.
                if name.is_empty() || name.contains("::") {
                    return Err(MatchArmError {
                        span: kspan.clone(),
                        reason: format!(
                            "map-pattern key must be a bare field keyword (`:a`), got `{kw}`"
                        ),
                    });
                }
                if out.iter().any(|(n, _)| n == name) {
                    return Err(MatchArmError {
                        span: kspan.clone(),
                        reason: format!("map-pattern key `:{name}` is repeated"),
                    });
                }
                out.push((name.to_string(), v));
            }
            WatAST::Symbol(_, sp) => {
                return Err(MatchArmError {
                    span: sp.clone(),
                    reason: "map pattern is key-first (`{:field binder}`); \
                             `{binder :field}` is let's destructure, a different operation"
                        .into(),
                });
            }
            other => {
                return Err(MatchArmError {
                    span: other.span().clone(),
                    reason: format!(
                        "map-pattern key must be a keyword, got {}",
                        other.variant_name()
                    ),
                });
            }
        }
    }
    let _ = span;
    Ok(out)
}
