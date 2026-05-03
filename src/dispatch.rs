//! Arc 146 slice 1 — dispatch entity + registry + parsing.
//!
//! A dispatch is a substrate entity that dispatches over input
//! type to one of N per-Type implementations. Pass-through semantics:
//! the dispatch's surface arity equals each arm's impl arity; all
//! args at the call site flow unchanged to the matched impl.
//!
//! # Why an entity kind, not a type-system feature
//!
//! Per `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 10 + arc 144
//! REALIZATIONS § 2: when polymorphism doesn't fit ONE rank-1 scheme,
//! the right answer is a NEW ENTITY KIND (dispatch — Clojure /
//! Common Lisp generic function / Julia multiple dispatch), NOT a
//! type-system extension. Each arm's impl is a clean rank-1 scheme;
//! the dispatch is the dispatch table that routes to them.
//!
//! # Form
//!
//! ```scheme
//! (:wat::core::define-dispatch :wat::core::length
//!   ((:wat::core::Vector<T>)    :wat::core::Vector/length)
//!   ((:wat::core::HashMap<K,V>) :wat::core::HashMap/length)
//!   ((:wat::core::HashSet<T>)   :wat::core::HashSet/length))
//! ```
//!
//! Each arm is `((<type-pattern>...) <impl-keyword>)`. The type-pattern
//! arity defines the dispatch's surface arity; ALL arms must have
//! the same arity. The impl-keyword names a per-Type primitive whose
//! arity must match the surface arity (verified at first check-time
//! call — see arc 146 BRIEF Q1).
//!
//! # Concurrency
//!
//! Per `docs/ZERO-MUTEX.md`: the registry is owned via Arc; per-
//! definition data is immutable after registration. The mutable
//! HashMap inside `DispatchRegistry` is only mutated during freeze
//! (single-threaded). Post-freeze the registry is wrapped in `Arc<...>`
//! and shared read-only — no Mutex/RwLock/CondVar.

use crate::ast::WatAST;
use crate::span::Span;
use crate::types::{parse_type_expr_with_span, TypeError, TypeExpr};
use std::collections::HashMap;
use std::fmt;

/// One arm of a dispatch's dispatch table — a per-Type pattern
/// paired with the keyword path of the impl that handles that pattern.
#[derive(Debug, Clone)]
pub struct DispatchArm {
    /// Input-type pattern, one TypeExpr per surface argument. Arity
    /// of this Vec is the dispatch's surface arity.
    pub pattern: Vec<TypeExpr>,
    /// Full keyword path of the per-Type impl (e.g.
    /// `:wat::core::Vector/length`). The impl must exist as a callable
    /// (user define or substrate primitive) at first check-time call.
    pub impl_name: String,
    /// Source span of the arm's syntax for diagnostic prefixing.
    pub span: Span,
}

/// A registered dispatch.
#[derive(Debug, Clone)]
pub struct Dispatch {
    /// Full keyword path of the dispatch (e.g. `:wat::core::length`).
    pub name: String,
    /// Dispatch table — one arm per per-Type impl. Order is the
    /// declaration order; check-time arm matching scans in this order
    /// and picks the first arm that unifies.
    pub arms: Vec<DispatchArm>,
    /// Source span of the `(:wat::core::define-dispatch ...)` form.
    pub span: Span,
}

/// Keyword-path → `Dispatch` registry. Mirrors
/// `crate::macros::MacroRegistry`'s shape.
#[derive(Debug, Default, Clone)]
pub struct DispatchRegistry {
    dispatchs: HashMap<String, Dispatch>,
}

impl DispatchRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.dispatchs.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&Dispatch> {
        self.dispatchs.get(name)
    }

    /// Register a dispatch. Errors on duplicate or reserved prefix.
    pub fn register(&mut self, def: Dispatch) -> Result<(), DispatchError> {
        if crate::resolve::is_reserved_prefix(&def.name) {
            return Err(DispatchError::ReservedPrefix(def.name, def.span.clone()));
        }
        if self.dispatchs.contains_key(&def.name) {
            return Err(DispatchError::DuplicateDispatch(
                def.name,
                def.span.clone(),
            ));
        }
        self.dispatchs.insert(def.name.clone(), def);
        Ok(())
    }

    /// Stdlib-registration variant — bypasses the reserved-prefix gate
    /// (substrate-declared dispatchs live under `:wat::core::*`).
    /// Still errors on duplicates.
    pub fn register_stdlib(&mut self, def: Dispatch) -> Result<(), DispatchError> {
        if self.dispatchs.contains_key(&def.name) {
            return Err(DispatchError::DuplicateDispatch(
                def.name,
                def.span.clone(),
            ));
        }
        self.dispatchs.insert(def.name.clone(), def);
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Dispatch)> {
        self.dispatchs.iter()
    }
}

/// Errors during dispatch registration / parsing / dispatch.
#[derive(Debug)]
pub enum DispatchError {
    /// A user dispatch declared under a reserved `:wat::...` prefix.
    ReservedPrefix(String, Span),
    /// Two `(:wat::core::define-dispatch ...)` forms registered the same
    /// name.
    DuplicateDispatch(String, Span),
    /// The `define_dispatch` form was malformed.
    MalformedDefdispatch { reason: String, span: Span },
    /// The arm-pattern arity disagreed with another arm's pattern arity
    /// (surface arity must be uniform across all arms).
    InconsistentArmArity {
        dispatch: String,
        first_arity: usize,
        offending_arity: usize,
        span: Span,
    },
    /// At first check-time call, an arm's impl was looked up and its
    /// arity disagreed with the dispatch's surface arity. Surfaced
    /// per arc 146 slice 1 BRIEF Q1 (deferred to call-time).
    ArityMismatch {
        dispatch: String,
        surface_arity: usize,
        arm_impl: String,
        arm_arity: usize,
        span: Span,
    },
    /// A type-pattern keyword failed to parse via `parse_type_expr`.
    InvalidTypePattern {
        dispatch: String,
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

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DispatchError::ReservedPrefix(n, span) => write!(
                f,
                "{}cannot declare dispatch {} — reserved prefix ({}); user dispatchs must use their own prefix",
                span_prefix(span),
                n,
                crate::resolve::reserved_prefix_list()
            ),
            DispatchError::DuplicateDispatch(n, span) => write!(
                f,
                "{}duplicate dispatch registration: {}",
                span_prefix(span),
                n
            ),
            DispatchError::MalformedDefdispatch { reason, span } => write!(
                f,
                "{}malformed define_dispatch: {}",
                span_prefix(span),
                reason
            ),
            DispatchError::InconsistentArmArity {
                dispatch,
                first_arity,
                offending_arity,
                span,
            } => write!(
                f,
                "{}dispatch {} arm-pattern arity {} disagrees with first arm's arity {}; all arms must share surface arity",
                span_prefix(span),
                dispatch,
                offending_arity,
                first_arity
            ),
            DispatchError::ArityMismatch {
                dispatch,
                surface_arity,
                arm_impl,
                arm_arity,
                span,
            } => write!(
                f,
                "{}dispatch {} surface arity {} disagrees with arm impl {}'s arity {}",
                span_prefix(span),
                dispatch,
                surface_arity,
                arm_impl,
                arm_arity
            ),
            DispatchError::InvalidTypePattern {
                dispatch,
                raw,
                cause,
                span,
            } => write!(
                f,
                "{}dispatch {} arm-pattern type {} failed to parse: {}",
                span_prefix(span),
                dispatch,
                raw,
                cause
            ),
        }
    }
}

impl std::error::Error for DispatchError {}

/// Walk `forms`, peel off every `(:wat::core::define-dispatch ...)`,
/// parse + register each into `registry`, and return the remaining
/// forms in order. Mirrors `crate::macros::register_defmacros`.
pub fn register_define_dispatches(
    forms: Vec<WatAST>,
    registry: &mut DispatchRegistry,
) -> Result<Vec<WatAST>, DispatchError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_define_dispatch_form(&form) {
            let def = parse_define_dispatch_form(form)?;
            registry.register(def)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

/// Stdlib-registration variant — bypasses the reserved-prefix gate.
pub fn register_stdlib_define_dispatches(
    forms: Vec<WatAST>,
    registry: &mut DispatchRegistry,
) -> Result<Vec<WatAST>, DispatchError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_define_dispatch_form(&form) {
            let def = parse_define_dispatch_form(form)?;
            registry.register_stdlib(def)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

pub fn is_define_dispatch_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items, _)
            if matches!(
                items.first(),
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::define-dispatch"
            )
    )
}

/// Parse `(:wat::core::define-dispatch :name ((<type-pattern>...) <impl>) ...)`.
///
/// The first child after the head must be a Keyword (the dispatch
/// name); each remaining child must be a 2-element List whose first
/// child is a List of type-pattern keywords (the arm pattern) and
/// whose second child is a Keyword (the impl path).
///
/// Surface arity is the arity of arms[0].pattern; later arms must
/// match. Per BRIEF Q1, impl-arity validation is deferred to
/// first check-time call.
pub fn parse_define_dispatch_form(form: WatAST) -> Result<Dispatch, DispatchError> {
    let (items, list_span) = match form {
        WatAST::List(items, span) => (items, span),
        _ => {
            return Err(DispatchError::MalformedDefdispatch {
                reason: "expected list form".into(),
                span: Span::unknown(),
            });
        }
    };
    if items.len() < 3 {
        return Err(DispatchError::MalformedDefdispatch {
            reason: format!(
                "expected (:wat::core::define-dispatch :name <arm>+); got {} elements",
                items.len()
            ),
            span: list_span,
        });
    }
    let mut iter = items.into_iter();
    let _define_dispatch_kw = iter.next();
    let name = match iter.next() {
        Some(WatAST::Keyword(k, _)) => k,
        Some(other) => {
            return Err(DispatchError::MalformedDefdispatch {
                reason: "dispatch name must be a keyword-path".into(),
                span: other.span().clone(),
            });
        }
        None => {
            return Err(DispatchError::MalformedDefdispatch {
                reason: "missing dispatch name".into(),
                span: list_span,
            });
        }
    };

    let mut arms: Vec<DispatchArm> = Vec::new();
    let mut surface_arity: Option<usize> = None;
    for item in iter {
        let arm = parse_arm(&name, item)?;
        if let Some(a) = surface_arity {
            if arm.pattern.len() != a {
                return Err(DispatchError::InconsistentArmArity {
                    dispatch: name.clone(),
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
        return Err(DispatchError::MalformedDefdispatch {
            reason: "dispatch must declare at least one arm".into(),
            span: list_span,
        });
    }

    Ok(Dispatch {
        name,
        arms,
        span: list_span,
    })
}

fn parse_arm(mm_name: &str, arm: WatAST) -> Result<DispatchArm, DispatchError> {
    let (children, arm_span) = match arm {
        WatAST::List(children, span) => (children, span),
        other => {
            return Err(DispatchError::MalformedDefdispatch {
                reason: "each arm must be a list ((<type-pattern>...) <impl-keyword>)".into(),
                span: other.span().clone(),
            });
        }
    };
    if children.len() != 2 {
        return Err(DispatchError::MalformedDefdispatch {
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
                        return Err(DispatchError::MalformedDefdispatch {
                            reason: "arm-pattern element must be a type keyword".into(),
                            span: other.span().clone(),
                        });
                    }
                };
                let ty = parse_type_expr_with_span(&raw, &item_span).map_err(|e| {
                    DispatchError::InvalidTypePattern {
                        dispatch: mm_name.to_string(),
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
            return Err(DispatchError::MalformedDefdispatch {
                reason: "arm pattern must be a list of type keywords".into(),
                span: other.span().clone(),
            });
        }
    };

    let impl_name = match impl_form {
        WatAST::Keyword(k, _) => k,
        other => {
            return Err(DispatchError::MalformedDefdispatch {
                reason: "arm impl must be a keyword-path".into(),
                span: other.span().clone(),
            });
        }
    };

    Ok(DispatchArm {
        pattern,
        impl_name,
        span: arm_span,
    })
}

fn format_type_error(e: &TypeError) -> String {
    format!("{}", e)
}
