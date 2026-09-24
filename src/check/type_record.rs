//! The type `infer` assigns to every node it infers — recorded, OFF by default.
//!
//! the-little-wat excursus 002 stone 2 ("the compiler knows what the language knows"). The
//! native wat compiler carries its own reconstruction of wat's types; this is the language's
//! side of the check that holds it to them. With recording on, every [`super::infer`] that
//! produces a type notes `(span, type)`; when the inference ROOT that owns the substitution
//! finishes (a function body, a top-level form, a `def`'s value, a defclause clause), each
//! note is resolved through that root's FINAL substitution and aliases are expanded
//! ([`super::reduce`]). A type recorded mid-inference can still hold a variable that a later
//! unification sharpens — `:None` infers as `Option` of a fresh `T` — which is why nothing is
//! resolved at the moment it is noted.
//!
//! **Nothing an existing caller observes changes.** Recording is per thread and off unless
//! [`start`] is called; while it is off, the only work `infer` does for it is one
//! thread-local flag read. It never changes a type, an error, or the order of anything.
//!
//! A type in which a unification variable survives the final substitution is kept apart as
//! [`Recorded::unresolved`], never presented as though it were a type.

use crate::span::Span;
use crate::types::{TypeEnv, TypeExpr};
use std::cell::{Cell, RefCell};

use super::Subst;

/// One node's type, resolved through its root's final substitution.
#[derive(Clone, Debug)]
pub struct Recorded {
    /// The node's span, as the reader stamped it (file, 1-based line, 1-based column).
    pub span: Span,
    /// The type, final substitution applied and aliases expanded.
    pub ty: TypeExpr,
    /// `ty` with every variant type, at every level, widened to the enum it belongs to
    /// (`:user::S.Some` -> `:user::S`) -- the language's own subsumption, the widening
    /// `check.rs`'s `widen_to_enclosing_enum` performs, applied structurally. Equal to `ty`
    /// when `ty` names no variant.
    pub wide: TypeExpr,
    /// `true` when a unification variable survived the final substitution.
    pub unresolved: bool,
}

/// What a recording session produced.
#[derive(Clone, Debug, Default)]
pub struct Recording {
    /// Every note, in the order its root finished.
    pub types: Vec<Recorded>,
    /// Notes made while no inference root was open — which would mean a root was missed.
    /// Counted, never guessed at.
    pub orphans: usize,
}

#[derive(Default)]
struct Recorder {
    /// One frame per open inference root; `infer` notes into the innermost.
    frames: Vec<Vec<(Span, TypeExpr)>>,
    done: Recording,
}

thread_local! {
    static ON: Cell<bool> = const { Cell::new(false) };
    static REC: RefCell<Recorder> = RefCell::new(Recorder::default());
}

/// Begin recording on this thread, discarding anything a previous session left.
pub fn start() {
    REC.with(|r| *r.borrow_mut() = Recorder::default());
    ON.with(|on| on.set(true));
}

/// Stop recording on this thread and hand back what was recorded. `None` if recording was
/// not on.
pub fn finish() -> Option<Recording> {
    if !is_on() {
        return None;
    }
    ON.with(|on| on.set(false));
    Some(REC.with(|r| std::mem::take(&mut r.borrow_mut().done)))
}

/// Is recording on for this thread?
#[inline]
pub fn is_on() -> bool {
    ON.with(|on| on.get())
}

/// Note the type `infer` just returned for the node at `span`.
pub(crate) fn note(span: &Span, ty: &TypeExpr) {
    REC.with(|r| {
        let mut r = r.borrow_mut();
        match r.frames.last_mut() {
            Some(frame) => frame.push((span.clone(), ty.clone())),
            None => r.done.orphans += 1,
        }
    });
}

/// An inference root that owns a substitution is starting.
pub(crate) fn open_root() {
    if is_on() {
        REC.with(|r| r.borrow_mut().frames.push(Vec::new()));
    }
}

/// That root is finished and `subst` is final: resolve every note made inside it.
pub(crate) fn close_root(subst: &Subst, types: &TypeEnv) {
    if !is_on() {
        return;
    }
    REC.with(|r| {
        let mut r = r.borrow_mut();
        let frame = match r.frames.pop() {
            Some(f) => f,
            None => return,
        };
        for (span, ty) in frame {
            let ty = super::reduce(&ty, subst, types);
            let unresolved = has_var(&ty);
            let wide = widen(&ty, types);
            r.done.types.push(Recorded { span, ty, wide, unresolved });
        }
    });
}

/// Every variant type in `t`, at every level, as the enum it belongs to. The head-level step
/// is exactly `check.rs`'s `widen_to_enclosing_enum`: `TypeEnv::enclosing_enum`, and a no-op
/// when the head already IS that enum.
fn widen(t: &TypeExpr, types: &TypeEnv) -> TypeExpr {
    match t {
        TypeExpr::Path(p) => match types.enclosing_enum(p) {
            Some(parent) if parent != crate::types::parametric_head_fqdn(p) => {
                TypeExpr::Path(parent.to_string())
            }
            _ => t.clone(),
        },
        TypeExpr::Parametric { head, args } => {
            let args: Vec<TypeExpr> = args.iter().map(|a| widen(a, types)).collect();
            let head = match types.enclosing_enum(head) {
                Some(parent) if parent != crate::types::parametric_head_fqdn(head) => {
                    parent.trim_start_matches(':').to_string()
                }
                _ => head.clone(),
            };
            TypeExpr::Parametric { head, args }
        }
        TypeExpr::Fn { args, ret } => TypeExpr::Fn {
            args: args.iter().map(|a| widen(a, types)).collect(),
            ret: Box::new(widen(ret, types)),
        },
        TypeExpr::Tuple(es) => TypeExpr::Tuple(es.iter().map(|e| widen(e, types)).collect()),
        TypeExpr::Var(_) => t.clone(),
    }
}

fn has_var(t: &TypeExpr) -> bool {
    match t {
        TypeExpr::Var(_) => true,
        TypeExpr::Path(_) => false,
        TypeExpr::Parametric { args, .. } => args.iter().any(has_var),
        TypeExpr::Fn { args, ret } => args.iter().any(has_var) || has_var(ret),
        TypeExpr::Tuple(es) => es.iter().any(has_var),
    }
}
