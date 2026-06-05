use crate::span::{span_prefix, Span};
use std::fmt;

/// Errors during macro registration / expansion. Pattern A (Stone
/// 243.7d): span at the outer struct level; variant data in
/// `MacroErrorKind`.
#[derive(Debug)]
pub struct MacroError {
    pub span: Span,
    pub kind: MacroErrorKind,
}

/// Variant data for [`MacroError`]. Spans live in the outer struct;
/// variants carry ONLY data unique to each failure kind.
// MacroErrorKind is pub because it's the type of MacroError's pub `kind` field (no private-in-public).
#[derive(Debug)]
pub enum MacroErrorKind {
    /// Two `(:wat::core::defmacro ...)` forms registered the same name.
    DuplicateMacro(String),
    /// A user macro declared under a reserved `:wat::...` prefix.
    ReservedPrefix(String),
    /// A `defmacro` form was malformed.
    MalformedDefmacro { reason: String },
    /// A macro call passed the wrong number of arguments (fixed-arity macro).
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
    /// A variadic macro call passed too few arguments (fewer than the
    /// required fixed-param minimum).
    ArityTooFew {
        name: String,
        minimum: usize,
        got: usize,
    },
    /// An `unquote` reference named a parameter the macro didn't declare.
    UnboundMacroParam { name: String },
    /// `unquote-splicing` was applied to a non-sequence argument.
    /// Accepts `WatAST::List` and `WatAST::Vector` (arc 200 made splice
    /// symmetric across both); fires for any other shape (Atom, Symbol,
    /// non-Vec runtime value, etc.). wat has no user-facing List runtime
    /// type — sequence here means "splice-compatible AST shape or runtime
    /// Vec value."
    SpliceNotSequence { name: String, got: &'static str },
    /// Expansion depth exceeded a sanity limit — probably an infinite
    /// recursive macro.
    ExpansionDepthExceeded { limit: usize },
    /// Other malformation in a macro invocation or template.
    MalformedTemplate { reason: String },
    /// A computed-unquote expression named a keyword head that is not on the
    /// blessed pure-combinator allow-list. Default-deny enforcement: the head
    /// is not a pure-total prim, so expand-time eval is refused.
    ///
    /// Arc 249 Stone 249.2b-i — F5 closure. Named `RefusedInMacro` (intueri
    /// cast): names the EVENT, not a cause — the refusal axis is purity AND
    /// totality, so "Impure" would assert a false cause for a non-total-but-pure
    /// head (a user-`defn` reference, `apply`, `eval-ast!`).
    RefusedInMacro { head: String },
    /// A program-body quasiquote template introduces a literal name in a
    /// binder position (`:wat::core::let` or `:wat::core::fn`), which could
    /// capture caller-site names silently. Default-deny per the hygiene bound:
    /// eval_quasiquote adds no hygiene scopes, so a literal binder could
    /// capture caller-site names. Use `~-unquote` to splice the name from a
    /// macro parameter instead. See DESIGN-STONE-249.2b.md for the
    /// let-need-reveal posture.
    ///
    /// Arc 249 Stone 249.2b-ii — gate E hygiene bound.
    ProgramBodyIntroducesName { macro_name: String, binder: String },
}

impl fmt::Display for MacroErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacroErrorKind::DuplicateMacro(n) => {
                write!(f, "duplicate macro registration: {}", n)
            }
            MacroErrorKind::ReservedPrefix(n) => write!(
                f,
                "cannot declare macro {} — reserved prefix ({}); user macros must use their own prefix",
                n,
                crate::resolve::reserved_prefix_list()
            ),
            MacroErrorKind::MalformedDefmacro { reason } => {
                write!(f, "malformed defmacro: {}", reason)
            }
            MacroErrorKind::ArityMismatch { name, expected, got } => {
                write!(
                    f,
                    "macro {} expects {} arguments; got {}",
                    name, expected, got
                )
            }
            MacroErrorKind::ArityTooFew { name, minimum, got } => {
                write!(
                    f,
                    "macro {} expects at least {} arguments; got {}",
                    name, minimum, got
                )
            }
            MacroErrorKind::UnboundMacroParam { name } => {
                write!(f, "unquote references unbound macro parameter: {}", name)
            }
            MacroErrorKind::SpliceNotSequence { name, got } => write!(
                f,
                "unquote-splicing (~@{}) requires a sequence (List/Vector AST or Vec value); got {}",
                name, got
            ),
            MacroErrorKind::ExpansionDepthExceeded { limit } => write!(
                f,
                "macro expansion exceeded depth limit {} — likely infinite recursion",
                limit
            ),
            MacroErrorKind::MalformedTemplate { reason } => {
                write!(f, "malformed template: {}", reason)
            }
            MacroErrorKind::RefusedInMacro { head } => write!(
                f,
                "keyword head `{}` refused at macro expand time — not on the pure-combinator \
                 allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only pure-total \
                 heads are permitted",
                head
            ),
            MacroErrorKind::ProgramBodyIntroducesName { macro_name, binder } => write!(
                f,
                "macro {} program body refused — quasiquote template introduces literal name \
                 `{}` in binder position; this could capture caller-site names (hygiene bound \
                 gate E, arc 249 stone 249.2b-ii); use ~-unquote to splice the name from a \
                 macro parameter",
                macro_name, binder
            ),
        }
    }
}

impl fmt::Display for MacroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = span_prefix(&self.span);
        write!(f, "{}{}", prefix, self.kind)
    }
}

impl std::error::Error for MacroError {}
