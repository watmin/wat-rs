//! Arc 146 slice 1 — multimethod entity + registry + parsing.
//!
//! A multimethod is a substrate entity that dispatches over input
//! type to one of N per-Type implementations. Pass-through semantics:
//! the multimethod's surface arity equals each arm's impl arity; all
//! args at the call site flow unchanged to the matched impl.
//!
//! # Why an entity kind, not a type-system feature
//!
//! Per `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 10 + arc 144
//! REALIZATIONS § 2: when polymorphism doesn't fit ONE rank-1 scheme,
//! the right answer is a NEW ENTITY KIND (multimethod — Clojure /
//! Common Lisp generic function / Julia multiple dispatch), NOT a
//! type-system extension. Each arm's impl is a clean rank-1 scheme;
//! the multimethod is the dispatch table that routes to them.
//!
//! # Form
//!
//! ```scheme
//! (:wat::core::defmultimethod :wat::core::length
//!   ((:wat::core::Vector<T>)    :wat::core::Vector/length)
//!   ((:wat::core::HashMap<K,V>) :wat::core::HashMap/length)
//!   ((:wat::core::HashSet<T>)   :wat::core::HashSet/length))
//! ```
//!
//! Each arm is `((<type-pattern>...) <impl-keyword>)`. The type-pattern
//! arity defines the multimethod's surface arity; ALL arms must have
//! the same arity. The impl-keyword names a per-Type primitive whose
//! arity must match the surface arity (verified at first check-time
//! call — see arc 146 BRIEF Q1).
//!
//! # Concurrency
//!
//! Per `docs/ZERO-MUTEX.md`: the registry is owned via Arc; per-
//! definition data is immutable after registration. The mutable
//! HashMap inside `MultimethodRegistry` is only mutated during freeze
//! (single-threaded). Post-freeze the registry is wrapped in `Arc<...>`
//! and shared read-only — no Mutex/RwLock/CondVar.

use crate::ast::WatAST;
use crate::span::Span;
use crate::types::{parse_type_expr_with_span, TypeError, TypeExpr};
use std::collections::HashMap;
use std::fmt;

/// One arm of a multimethod's dispatch table — a per-Type pattern
/// paired with the keyword path of the impl that handles that pattern.
#[derive(Debug, Clone)]
pub struct MultimethodArm {
    /// Input-type pattern, one TypeExpr per surface argument. Arity
    /// of this Vec is the multimethod's surface arity.
    pub pattern: Vec<TypeExpr>,
    /// Full keyword path of the per-Type impl (e.g.
    /// `:wat::core::Vector/length`). The impl must exist as a callable
    /// (user define or substrate primitive) at first check-time call.
    pub impl_name: String,
    /// Source span of the arm's syntax for diagnostic prefixing.
    pub span: Span,
}

/// A registered multimethod.
#[derive(Debug, Clone)]
pub struct Multimethod {
    /// Full keyword path of the multimethod (e.g. `:wat::core::length`).
    pub name: String,
    /// Dispatch table — one arm per per-Type impl. Order is the
    /// declaration order; check-time arm matching scans in this order
    /// and picks the first arm that unifies.
    pub arms: Vec<MultimethodArm>,
    /// Source span of the `(:wat::core::defmultimethod ...)` form.
    pub span: Span,
}

/// Keyword-path → `Multimethod` registry. Mirrors
/// `crate::macros::MacroRegistry`'s shape.
#[derive(Debug, Default, Clone)]
pub struct MultimethodRegistry {
    multimethods: HashMap<String, Multimethod>,
}

impl MultimethodRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.multimethods.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&Multimethod> {
        self.multimethods.get(name)
    }

    /// Register a multimethod. Errors on duplicate or reserved prefix.
    pub fn register(&mut self, def: Multimethod) -> Result<(), MultimethodError> {
        if crate::resolve::is_reserved_prefix(&def.name) {
            return Err(MultimethodError::ReservedPrefix(def.name, def.span.clone()));
        }
        if self.multimethods.contains_key(&def.name) {
            return Err(MultimethodError::DuplicateMultimethod(
                def.name,
                def.span.clone(),
            ));
        }
        self.multimethods.insert(def.name.clone(), def);
        Ok(())
    }

    /// Stdlib-registration variant — bypasses the reserved-prefix gate
    /// (substrate-declared multimethods live under `:wat::core::*`).
    /// Still errors on duplicates.
    pub fn register_stdlib(&mut self, def: Multimethod) -> Result<(), MultimethodError> {
        if self.multimethods.contains_key(&def.name) {
            return Err(MultimethodError::DuplicateMultimethod(
                def.name,
                def.span.clone(),
            ));
        }
        self.multimethods.insert(def.name.clone(), def);
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Multimethod)> {
        self.multimethods.iter()
    }
}

/// Errors during multimethod registration / parsing / dispatch.
#[derive(Debug)]
pub enum MultimethodError {
    /// A user multimethod declared under a reserved `:wat::...` prefix.
    ReservedPrefix(String, Span),
    /// Two `(:wat::core::defmultimethod ...)` forms registered the same
    /// name.
    DuplicateMultimethod(String, Span),
    /// The `defmultimethod` form was malformed.
    MalformedDefmultimethod { reason: String, span: Span },
    /// The arm-pattern arity disagreed with another arm's pattern arity
    /// (surface arity must be uniform across all arms).
    InconsistentArmArity {
        multimethod: String,
        first_arity: usize,
        offending_arity: usize,
        span: Span,
    },
    /// At first check-time call, an arm's impl was looked up and its
    /// arity disagreed with the multimethod's surface arity. Surfaced
    /// per arc 146 slice 1 BRIEF Q1 (deferred to call-time).
    ArityMismatch {
        multimethod: String,
        surface_arity: usize,
        arm_impl: String,
        arm_arity: usize,
        span: Span,
    },
    /// A type-pattern keyword failed to parse via `parse_type_expr`.
    InvalidTypePattern {
        multimethod: String,
        raw: String,
        cause: String,
        span: Span,
    },
}

/// Prefix `"<file>:<line>:<col>: "` when span is known; mirrors
/// `crate::macros::span_prefix`.
fn span_prefix(span: &Span) -> String {
    if span.is_unknown() {
        String::new()
    } else {
        format!("{}: ", span)
    }
}

impl fmt::Display for MultimethodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultimethodError::ReservedPrefix(n, span) => write!(
                f,
                "{}cannot declare multimethod {} — reserved prefix ({}); user multimethods must use their own prefix",
                span_prefix(span),
                n,
                crate::resolve::reserved_prefix_list()
            ),
            MultimethodError::DuplicateMultimethod(n, span) => write!(
                f,
                "{}duplicate multimethod registration: {}",
                span_prefix(span),
                n
            ),
            MultimethodError::MalformedDefmultimethod { reason, span } => write!(
                f,
                "{}malformed defmultimethod: {}",
                span_prefix(span),
                reason
            ),
            MultimethodError::InconsistentArmArity {
                multimethod,
                first_arity,
                offending_arity,
                span,
            } => write!(
                f,
                "{}multimethod {} arm-pattern arity {} disagrees with first arm's arity {}; all arms must share surface arity",
                span_prefix(span),
                multimethod,
                offending_arity,
                first_arity
            ),
            MultimethodError::ArityMismatch {
                multimethod,
                surface_arity,
                arm_impl,
                arm_arity,
                span,
            } => write!(
                f,
                "{}multimethod {} surface arity {} disagrees with arm impl {}'s arity {}",
                span_prefix(span),
                multimethod,
                surface_arity,
                arm_impl,
                arm_arity
            ),
            MultimethodError::InvalidTypePattern {
                multimethod,
                raw,
                cause,
                span,
            } => write!(
                f,
                "{}multimethod {} arm-pattern type {} failed to parse: {}",
                span_prefix(span),
                multimethod,
                raw,
                cause
            ),
        }
    }
}

impl std::error::Error for MultimethodError {}

/// Walk `forms`, peel off every `(:wat::core::defmultimethod ...)`,
/// parse + register each into `registry`, and return the remaining
/// forms in order. Mirrors `crate::macros::register_defmacros`.
pub fn register_defmultimethods(
    forms: Vec<WatAST>,
    registry: &mut MultimethodRegistry,
) -> Result<Vec<WatAST>, MultimethodError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_defmultimethod_form(&form) {
            let def = parse_defmultimethod_form(form)?;
            registry.register(def)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

/// Stdlib-registration variant — bypasses the reserved-prefix gate.
pub fn register_stdlib_defmultimethods(
    forms: Vec<WatAST>,
    registry: &mut MultimethodRegistry,
) -> Result<Vec<WatAST>, MultimethodError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_defmultimethod_form(&form) {
            let def = parse_defmultimethod_form(form)?;
            registry.register_stdlib(def)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

pub fn is_defmultimethod_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items, _)
            if matches!(
                items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::defmultimethod"
            )
    )
}

/// Parse `(:wat::core::defmultimethod :name ((<type-pattern>...) <impl>) ...)`.
///
/// The first child after the head must be a Keyword (the multimethod
/// name); each remaining child must be a 2-element List whose first
/// child is a List of type-pattern keywords (the arm pattern) and
/// whose second child is a Keyword (the impl path).
///
/// Surface arity is the arity of arms[0].pattern; later arms must
/// match. Per BRIEF Q1, impl-arity validation is deferred to
/// first check-time call.
pub fn parse_defmultimethod_form(form: WatAST) -> Result<Multimethod, MultimethodError> {
    let (items, list_span) = match form {
        WatAST::List(items, span) => (items, span),
        _ => {
            return Err(MultimethodError::MalformedDefmultimethod {
                reason: "expected list form".into(),
                span: Span::unknown(),
            });
        }
    };
    if items.len() < 3 {
        return Err(MultimethodError::MalformedDefmultimethod {
            reason: format!(
                "expected (:wat::core::defmultimethod :name <arm>+); got {} elements",
                items.len()
            ),
            span: list_span,
        });
    }
    let mut iter = items.into_iter();
    let _defmultimethod_kw = iter.next();
    let name = match iter.next() {
        Some(WatAST::Keyword(k, _)) => k,
        Some(other) => {
            return Err(MultimethodError::MalformedDefmultimethod {
                reason: "multimethod name must be a keyword-path".into(),
                span: other.span().clone(),
            });
        }
        None => {
            return Err(MultimethodError::MalformedDefmultimethod {
                reason: "missing multimethod name".into(),
                span: list_span,
            });
        }
    };

    let mut arms: Vec<MultimethodArm> = Vec::new();
    let mut surface_arity: Option<usize> = None;
    for item in iter {
        let arm = parse_arm(&name, item)?;
        if let Some(a) = surface_arity {
            if arm.pattern.len() != a {
                return Err(MultimethodError::InconsistentArmArity {
                    multimethod: name.clone(),
                    first_arity: a,
                    offending_arity: arm.pattern.len(),
                    span: arm.span.clone(),
                });
            }
        } else {
            surface_arity = Some(arm.pattern.len());
        }
        arms.push(arm);
    }
    if arms.is_empty() {
        return Err(MultimethodError::MalformedDefmultimethod {
            reason: "multimethod must declare at least one arm".into(),
            span: list_span,
        });
    }

    Ok(Multimethod {
        name,
        arms,
        span: list_span,
    })
}

fn parse_arm(mm_name: &str, arm: WatAST) -> Result<MultimethodArm, MultimethodError> {
    let (children, arm_span) = match arm {
        WatAST::List(children, span) => (children, span),
        other => {
            return Err(MultimethodError::MalformedDefmultimethod {
                reason: "each arm must be a list ((<type-pattern>...) <impl-keyword>)".into(),
                span: other.span().clone(),
            });
        }
    };
    if children.len() != 2 {
        return Err(MultimethodError::MalformedDefmultimethod {
            reason: format!(
                "each arm must have exactly 2 elements (pattern + impl); got {}",
                children.len()
            ),
            span: arm_span,
        });
    }
    let mut iter = children.into_iter();
    let pattern_form = iter.next().expect("length checked");
    let impl_form = iter.next().expect("length checked");

    let pattern = match pattern_form {
        WatAST::List(pattern_items, _) => {
            let mut out = Vec::with_capacity(pattern_items.len());
            for item in pattern_items {
                let (raw, item_span) = match item {
                    WatAST::Keyword(k, span) => (k, span),
                    other => {
                        return Err(MultimethodError::MalformedDefmultimethod {
                            reason: "arm-pattern element must be a type keyword".into(),
                            span: other.span().clone(),
                        });
                    }
                };
                let ty = parse_type_expr_with_span(&raw, &item_span).map_err(|e| {
                    MultimethodError::InvalidTypePattern {
                        multimethod: mm_name.to_string(),
                        raw: raw.clone(),
                        cause: format_type_error(&e),
                        span: item_span,
                    }
                })?;
                out.push(ty);
            }
            out
        }
        other => {
            return Err(MultimethodError::MalformedDefmultimethod {
                reason: "arm pattern must be a list of type keywords".into(),
                span: other.span().clone(),
            });
        }
    };

    let impl_name = match impl_form {
        WatAST::Keyword(k, _) => k,
        other => {
            return Err(MultimethodError::MalformedDefmultimethod {
                reason: "arm impl must be a keyword-path".into(),
                span: other.span().clone(),
            });
        }
    };

    Ok(MultimethodArm {
        pattern,
        impl_name,
        span: arm_span,
    })
}

fn format_type_error(e: &TypeError) -> String {
    format!("{}", e)
}
