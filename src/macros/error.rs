use crate::runtime::RuntimeError;
use crate::span::Span;
use std::fmt;

/// Errors during macro registration / expansion. Pattern A (Stone
/// 243.7d): span at the outer struct level; variant data in
/// `MacroErrorKind`.
pub struct MacroError {
    pub span: Span,
    pub kind: MacroErrorKind,
}

/// Arc 296 stone I — the taxonomy conversion `resolve::register`'s `?` performs at every
/// macro-registration call site. `Rejection::verdict` is never `Insert`/`NoOp` (see its
/// doc), so those two arms are unreachable by construction.
impl From<crate::resolve::Rejection> for MacroError {
    fn from(r: crate::resolve::Rejection) -> Self {
        use crate::resolve::Registration;
        let kind = match r.verdict {
            Registration::Duplicate => MacroErrorKind::DuplicateMacro(r.name),
            Registration::Reserved => MacroErrorKind::ReservedPrefix(r.name),
            Registration::Unnamespaced => MacroErrorKind::UnnamespacedName(r.name),
            Registration::DottedName => MacroErrorKind::DottedName(r.name),
            Registration::Insert | Registration::NoOp => {
                unreachable!("resolve::register never rejects with Insert/NoOp")
            }
        };
        MacroError { span: r.span, kind }
    }
}

/// Variant data for [`MacroError`]. Spans live in the outer struct;
/// variants carry ONLY data unique to each failure kind.
///
/// Arc 298.3: `#[derive(wat_edn::ToEdn)]` generates the kind enum's
/// `impl ToEdn`. The outer `MacroError::to_edn()` wraps it with
/// `splice_span(self.kind.to_edn(), &self.span)`. Replaces the deleted
/// hand-written `macro_error_to_edn` match in `macros/error_edn.rs`.
// MacroErrorKind is pub because it's the type of MacroError's pub `kind` field (no private-in-public).
#[derive(Debug, wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::MACRO)]
pub enum MacroErrorKind {
    /// Two `(:wat::core::defmacro ...)` forms registered the same name.
    #[to_edn(key = "name")]
    DuplicateMacro(String),
    /// A user macro declared under a reserved `:wat::...` prefix.
    #[to_edn(key = "name")]
    ReservedPrefix(String),
    /// A macro name reached the registration gate with no namespace. Only fn
    /// arguments and `let` bindings may be bare — those are lexical and never
    /// reach a gate. Held against `Privilege::Stdlib` too; there is no
    /// privilege escape from the namespacing wall.
    #[to_edn(key = "name")]
    UnnamespacedName(String),
    /// Arc 296 stone H-1 — a macro name reached the registration gate with a `.` in its
    /// name segment (the part after the last `::`). Same door as `UnnamespacedName` /
    /// `ReservedPrefix` above — third taxonomy entry for `Registration::DottedName`.
    /// Reserved because a dotted NAME is the wire discriminator for a tagged-enum
    /// variant (`#ns/Enum.Variant`); a record whose name contained a dot could forge it.
    #[to_edn(key = "name")]
    DottedName(String),
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

    /// Macro program-body evaluation failed with a nested `MacroError`.
    ///
    /// Arc 296: replaces the prose-collapsed
    /// `MalformedTemplate { reason: "macro NAME — program body eval failed: …" }`.
    /// The cause carries the full typed chain; callers inspect `cause.kind`
    /// instead of parsing the reason string.
    ProgramBodyEvalFailed {
        macro_name: String,
        #[to_edn(via = crate::edn::contract::error_edn_of_boxed)]
        cause: Box<MacroError>,
    },

    /// Runtime `eval` in macro-eval context failed with a `RuntimeError`.
    ///
    /// Arc 296: replaces the prose-collapsed `MalformedTemplate` in the
    /// non-`MacroAbort` arm of `macro_eval_pre_validated`. The `MacroAbort`
    /// arm retains `MalformedTemplate { reason: message }` (clean user
    /// message, no structural cause). All other runtime failures carry the
    /// typed `RuntimeError` here.
    MacroEvalRuntimeFailed {
        #[to_edn(via = crate::edn::contract::error_edn_of_boxed)]
        cause: Box<RuntimeError>,
    },
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
            MacroErrorKind::UnnamespacedName(n) => write!(
                f,
                "top-level name '{}' is not namespaced — only fn arguments and let-bindings \
                 may be bare; give it a namespace, e.g. ':my::{}'",
                n,
                n.trim_start_matches(':')
            ),
            MacroErrorKind::DottedName(n) => write!(
                f,
                "macro name '{}' contains a '.' in its name segment — reserved: a dot in a \
                 tag's NAME half means \"this is an enum variant\" (`#ns/Enum.Variant`), so a \
                 registered name may not contain one, or it could forge that tag; rename \
                 without the dot",
                n
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
            MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause } => write!(
                f,
                "macro {} — program body eval failed: {}",
                macro_name, cause
            ),
            MacroErrorKind::MacroEvalRuntimeFailed { cause } => write!(
                f,
                "macro_eval: runtime::eval failed: {}",
                cause
            ),
        }
    }
}

impl fmt::Debug for MacroError {
    // Stone B: Debug emits EDN, not Rust struct layout.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl fmt::Display for MacroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl std::error::Error for MacroError {}
