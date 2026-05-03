//! Type-check pass — rank-1 Hindley-Milner.
//!
//! This slice implements real parametric polymorphism per 058-030:
//!
//! - [`TypeScheme`] carries `type_params` — the list of names that are
//!   universally quantified (e.g., `["T"]` for `∀T. T -> :wat::holon::HolonAST`).
//! - Every call site **instantiates** the scheme with fresh unification
//!   variables ([`TypeExpr::Var`]), accumulates a substitution by
//!   unifying each argument type with its (instantiated) parameter
//!   type, and produces a monomorphic return type after applying the
//!   final substitution.
//! - Within a user define's body, declared type parameters are
//!   **rigid** — they unify only with themselves, not with concrete
//!   types. The body must type-check for any choice of T.
//! - Built-in schemes use real polymorphism: `list` is `∀T. T* ->
//!   List<T>`; `= < > <= >=` are `∀T. T -> T -> :bool`; `Atom` is
//!   `∀T. T -> :wat::holon::HolonAST`.
//! - `:Any` is banned everywhere — the type universe is closed
//!   ([058-030](https://…/058-030-types/PROPOSAL.md), §591). User
//!   source containing `:Any` halts at parse (see
//!   [`crate::types::parse_type_expr`]).
//!
//! # What this catches today
//!
//! - Arity mismatches in user-function and built-in calls at startup.
//! - Type mismatches: `(:wat::core::i64::+ "hello" 3)`, `(:wat::core::< 1 "x")`
//!   — `<` requires matching operand types.
//! - Polymorphic failures: `(:wat::core::vec 1 "two" 3)` — list
//!   elements must unify to a common element type.
//! - User-define body vs signature mismatches. Rigid type params
//!   mean a body of `:i64` in a `∀T. T -> T` signature is rejected.
//!
//! # What this does NOT catch (explicitly deferred)
//!
//! - **Lambda-value call-site typing.** Lambda values don't carry
//!   structured signatures through [`crate::runtime::Function`] yet,
//!   so calling a lambda stays Unknown at the check layer.
//! - **`:Union<T,U,V>` coproduct discipline.** `:Union` is a
//!   first-class type form in the grammar; full subtype / variant
//!   checks land when stdlib needs demand them.
//! - **Typed-macro parameter checks (058-032).** Macros expand before
//!   check; macro-definition-time checks (`:AST<T>` against body
//!   positions) are future work.
//! - **Numeric promotion.** `:i64` does not promote to `:f64` statically;
//!   mixing numeric types in arithmetic is rejected.

use crate::ast::WatAST;
use crate::identifier::Identifier;
use crate::runtime::{Function, SymbolTable};
use crate::span::Span;
use crate::types::{TypeEnv, TypeExpr};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// A function's declared signature: universally-quantified type
/// parameters, then parameter types and return type.
///
/// `type_params` names the variables that are `∀`-bound over the
/// scheme. At every use site, [`instantiate`] freshens them to unique
/// [`TypeExpr::Var`]s so multiple independent call sites don't alias.
#[derive(Debug, Clone)]
pub struct TypeScheme {
    pub type_params: Vec<String>,
    pub params: Vec<TypeExpr>,
    pub ret: TypeExpr,
}

/// Type-checking errors. Multiple errors accumulate in a single pass
/// so users get one batch of findings.
#[derive(Debug, Clone)]
pub enum CheckError {
    ArityMismatch {
        callee: String,
        expected: usize,
        got: usize,
        /// Arc 138 slice 1 — source coordinates of the offending
        /// call form so users (and agents) can navigate to the
        /// site without grepping. Use [`Span::unknown`] for
        /// synthetic check rules that have no originating node;
        /// Display skips the prefix when the span is unknown.
        span: Span,
    },
    TypeMismatch {
        callee: String,
        param: String,
        expected: String,
        got: String,
        /// Arc 138 slice 1 — see `ArityMismatch::span`.
        span: Span,
    },
    ReturnTypeMismatch {
        function: String,
        expected: String,
        got: String,
        /// Arc 138 slice 1 — see `ArityMismatch::span`.
        span: Span,
    },
    UnknownCallee {
        callee: String,
        /// Arc 138 slice 1 — see `ArityMismatch::span`.
        span: Span,
    },
    /// A built-in form (e.g., `:wat::core::match`) is structurally
    /// malformed in a way the syntax-level grammar doesn't catch —
    /// e.g., a match arm that isn't `(pattern body)`, or a match
    /// whose pattern coverage is non-exhaustive.
    MalformedForm {
        head: String,
        reason: String,
        /// Arc 138 slice 1 — see `ArityMismatch::span`.
        span: Span,
    },
    /// Arc 110 — `:wat::kernel::send` / `:wat::kernel::recv` appeared
    /// somewhere other than the discriminant of `:wat::core::match`
    /// or the value-position of `:wat::core::option::expect`.
    /// Silent-disconnect bugs (proof_004) are the class this rule
    /// makes structurally impossible.
    CommCallOutOfPosition {
        callee: String,
        /// Arc 138 slice 1 — see `ArityMismatch::span`.
        span: Span,
    },
    /// Arc 117 — a `let*` binding-block contains BOTH a Thread/Process
    /// binding (a value one calls `Thread/join-result` / `Process/
    /// join-result` on) AND a sibling binding whose alias-resolved
    /// type contains a Sender-bearing parametric (`Sender`,
    /// `QueueSender`, `QueuePair`, `HandlePool`), AND that let*'s
    /// extent contains a join-result call on the Thread. The
    /// Sender-bearing sibling holds a clone alive past the join site;
    /// the worker can't see EOF; the join blocks forever. The fix is
    /// structural: nest the Sender-bearing bindings in an inner
    /// `let*` whose body returns the Thread.
    ScopeDeadlock {
        /// Name of the Thread (or Process) binding being joined.
        thread_binding: String,
        /// Name of the sibling Sender-bearing binding whose live
        /// reference outlives the join site.
        offending_binding: String,
        /// Kind of the offending value — "Sender", "Channel", or
        /// "HandlePool". Names which substrate abstraction holds
        /// the live Sender clone.
        offending_kind: &'static str,
        /// Source location of the Thread binding (the let* site
        /// where the structural deadlock can be addressed by
        /// nesting).
        span: Span,
    },
    /// Arc 126 — a function call passes two arguments that trace
    /// back to the same `:wat::kernel::make-bounded-channel` /
    /// `make-unbounded-channel` pair-anchor. One argument is a
    /// `Sender<T>`; the other is a `Receiver<T>`; both are halves
    /// of one channel. Holding both ends in one role deadlocks any
    /// recv on the Receiver — the caller's Sender clone keeps the
    /// channel alive even if the receiving thread dies.
    ///
    /// Sibling rule to `ScopeDeadlock`: same trace machinery applied
    /// at call sites instead of spawn-thread closure bodies. Catches
    /// the arc 119 "Pattern B Put-ack helper-verb cycle" deadlock
    /// before runtime.
    ChannelPairDeadlock {
        /// Name of the function being called (callee head).
        callee: String,
        /// Name of the `Sender<T>`-typed argument.
        sender_arg: String,
        /// Name of the `Receiver<T>`-typed argument.
        receiver_arg: String,
        /// Name of the let* binding that held the pair-anchor
        /// (the `(:wat::kernel::make-bounded-channel ...)` RHS).
        pair_anchor: String,
        /// Source location of the function-call site.
        span: Span,
    },
    /// Arc 109 slice 1c — a bare primitive type (`:i64`, `:f64`,
    /// `:bool`, `:String`, `:u8`) appears in user code. Bare forms
    /// retire; the canonical FQDN form (`:wat::core::i64`, etc.) is
    /// the substitute. Detected at every type-token position
    /// (outer annotation, parametric inner, tuple member, fn arg/
    /// return).
    ///
    /// Same SUBSTRATE-AS-TEACHER pattern 3 as `CommCallOutOfPosition`
    /// (arc 110), `InnerColonInCompoundArg` (arc 115), `ScopeDeadlock`
    /// (arc 117) — dedicated variant per migration class; Display IS
    /// the migration brief; no `collect_hints` involvement.
    BareLegacyPrimitive {
        /// User-source spelling — `":i64"` (outer) or `"i64"` (bare
        /// inside parametric). Reproduces the offending token shape.
        primitive: String,
        /// Canonical FQDN form — `":wat::core::i64"` (outer) or
        /// `"wat::core::i64"` (matching the offending token's
        /// position).
        fqdn: String,
        /// Source location of the keyword carrying the bare token.
        /// Multiple bare occurrences in one keyword (e.g.
        /// `:Vec<i64,String>`) all carry the same span — the keyword
        /// token.
        span: Span,
    },
    /// Arc 109 slice 1d — the bare unit type annotation (`:()` or
    /// `()` inside a parametric/tuple/fn) appears in user code.
    /// `:wat::core::unit` is the canonical FQDN form. The
    /// empty-tuple LITERAL VALUE `()` is a list literal and is not
    /// affected; only the type-position spelling retires.
    ///
    /// Distinct from `BareLegacyPrimitive` because the unit type
    /// parses to `TypeExpr::Tuple(vec![])`, not `TypeExpr::Path` —
    /// the walker arm is a separate Tuple-empty guard. Same
    /// Pattern 3 mechanism otherwise.
    BareLegacyUnitType {
        /// Source location of the keyword carrying the bare unit
        /// token.
        span: Span,
    },
    /// Arc 109 slice 1e — a bare substrate-named parametric type
    /// head (`Option`, `Result`, `HashMap`, `HashSet`) appears in
    /// user code. The four containers move under `:wat::core::*`;
    /// the bare-source spelling retires.
    ///
    /// Detects against `TypeExpr::Parametric.head` — the third
    /// TypeExpr shape the walker template covers (slice 1c
    /// detected `Path`, slice 1d detected `Tuple`). The mechanism
    /// generalizes across all TypeExpr shapes via per-arm guards.
    ///
    /// Vec<T> is NOT in this slice — slice 1f territory because
    /// the rename to Vector couples with § D's verb companion.
    BareLegacyContainerHead {
        /// User-source spelling — `"Option"` / `"Result"` /
        /// `"HashMap"` / `"HashSet"`.
        head: String,
        /// Canonical FQDN form — `"wat::core::Option"` etc. (no
        /// leading colon; matches the head-position spelling at
        /// the offending site).
        fqdn: String,
        /// Source location of the keyword carrying the bare head.
        span: Span,
    },
    /// Arc 109 slice 9d — a keyword carrying the legacy
    /// `:wat::std::stream::` prefix appears in user code. The
    /// stream stdlib graduated to `:wat::stream::*` per § G's
    /// three-tier substrate organization (every substrate concern
    /// earns its own top-level tier; `:wat::std::*` empties out).
    /// File path mirrors: `wat/std/stream.wat` → `wat/stream.wat`.
    ///
    /// Walker fires on every keyword starting with the legacy
    /// prefix, regardless of position (callable head, type
    /// annotation, value position) — uniform Pattern 3 detection.
    /// Same shape as slices 1c/1d/1e but at the keyword-prefix
    /// level rather than the parsed-TypeExpr level.
    BareLegacyStreamPath {
        /// User-source keyword — `":wat::std::stream::map"`,
        /// `":wat::std::stream::Stream"`, etc.
        old: String,
        /// Canonical replacement — `":wat::stream::map"`, etc.
        /// Computed by stripping the `std::` segment.
        new: String,
        /// Source location of the keyword carrying the legacy
        /// prefix.
        span: Span,
    },
    /// Arc 109 slice K.telemetry — a keyword carrying the legacy
    /// `:wat::telemetry::Service::` (typealias path) or
    /// `:wat::telemetry::Service/` (verb path) prefix appears in
    /// user code. The Service grouping noun retires per § K's
    /// "/ requires a real Type" doctrine — Service has no struct,
    /// no value, no kind; verbs and typealiases live at the
    /// namespace level. Real types Stats and MetricsCadence keep
    /// their PascalCase + /methods because they ARE structs (just
    /// one less namespace segment deep).
    ///
    /// Walker fires on every keyword whose path starts with one
    /// of the two retired prefixes; canonical replacement strips
    /// the `Service::` or `Service/` segment.
    BareLegacyTelemetryServicePath {
        /// User-source keyword — `":wat::telemetry::Service/spawn"`,
        /// `":wat::telemetry::Service::Stats"`, etc.
        old: String,
        /// Canonical replacement — `":wat::telemetry::spawn"`,
        /// `":wat::telemetry::Stats"`, etc.
        new: String,
        /// Source location of the keyword carrying the legacy
        /// prefix.
        span: Span,
    },
    /// Arc 109 slice K.console — a keyword carrying the legacy
    /// `:wat::std::service::Console::` (typealias path) or
    /// `:wat::std::service::Console/` (verb path) prefix appears
    /// in user code. The Console grouping noun retires per § K's
    /// "/ requires a real Type" doctrine; verbs and typealiases
    /// live at the namespace level (`:wat::console::*`).
    ///
    /// Pattern A canonicalization rides this slice: `Tx` and `Rx`
    /// rename to `ReqTx` / `ReqRx` (per the channel-naming-
    /// patterns subsection of § K — Console mirrors Telemetry's
    /// canonical Pattern A shape post-K.console).
    BareLegacyConsolePath {
        /// User-source keyword — `":wat::std::service::Console/spawn"`,
        /// `":wat::std::service::Console::Tx"`, etc.
        old: String,
        /// Canonical replacement — `":wat::console::spawn"`,
        /// `":wat::console::ReqTx"` (Tx → ReqTx is the channel
        /// canonicalization), `":wat::console::Message"`, etc.
        new: String,
        /// Source location of the keyword carrying the legacy
        /// prefix.
        span: Span,
    },
    /// Arc 109 slice K.lru — a keyword carrying the legacy
    /// `:wat::lru::CacheService::` (typealias path) or
    /// `:wat::lru::CacheService/` (verb path) prefix appears in
    /// user code. The CacheService grouping noun retires per § K;
    /// verbs and typealiases live at `:wat::lru::*`. Real types
    /// Stats / MetricsCadence / State / Report keep their
    /// PascalCase + /methods (just one less namespace segment).
    ///
    /// Pattern B canonicalization rides this slice: `ReqPair`
    /// renames to `ReqChannel` (gaze 2026-05-01 — in-crate
    /// ReqPair/ReplyChannel mumble; both are (Tx, Rx) tuples but
    /// the suffix divergence forces lookup); plus NEW
    /// `ReplyRx<V>` + `ReplyChannel<V>` typealiases minted to
    /// complete the Pattern B reference.
    BareLegacyLruCacheServicePath {
        /// User-source keyword — `":wat::lru::CacheService/get"`,
        /// `":wat::lru::CacheService::ReqPair"`, etc.
        old: String,
        /// Canonical replacement — `":wat::lru::get"`,
        /// `":wat::lru::ReqChannel"` (ReqPair → ReqChannel),
        /// `":wat::lru::Stats"`, etc.
        new: String,
        /// Source location of the keyword carrying the legacy
        /// prefix.
        span: Span,
    },
    /// Arc 109 slice K.kernel-channel — a keyword carrying one of
    /// the retired kernel `Queue*` family names. The kernel's
    /// channel-primitive vocabulary moves from Queue* (which
    /// leaked crossbeam's data-structure name) to the canonical
    /// Channel / Sender / Receiver family.
    ///
    /// Retired prefixes:
    /// - `:wat::kernel::QueueSender` → `:wat::kernel::Sender`
    /// - `:wat::kernel::QueueReceiver` → `:wat::kernel::Receiver`
    /// - `:wat::kernel::QueuePair` → `:wat::kernel::Channel`
    /// - `:wat::kernel::make-bounded-queue` → `:wat::kernel::make-bounded-channel`
    /// - `:wat::kernel::make-unbounded-queue` → `:wat::kernel::make-unbounded-channel`
    BareLegacyKernelQueuePath {
        /// User-source keyword — `":wat::kernel::QueueSender"`,
        /// `":wat::kernel::make-bounded-queue"`, etc.
        old: String,
        /// Canonical replacement — `":wat::kernel::Sender"`,
        /// `":wat::kernel::make-bounded-channel"`, etc.
        new: String,
        /// Source location of the keyword carrying the retired
        /// name.
        span: Span,
    },
    /// Arc 140 — a deftest body (or any sandboxed sub-program's
    /// forms-block) invokes a name that exists in the OUTER scope
    /// but NOT in the sub-program's own forms (prelude + auto-
    /// generated `:user::main`). Sandboxes do NOT capture outer
    /// scope by design (per `wat/test.wat`'s deftest macro and
    /// `wat/std/sandbox.wat`'s `run-sandboxed-ast`); the user
    /// either typed a name they thought would be visible or
    /// forgot to put the helper into the deftest's prelude.
    ///
    /// Same teaching shape as arc 117 `ScopeDeadlock` / arc 126
    /// `ChannelPairDeadlock`: substrate-as-teacher pattern 3 —
    /// dedicated variant per failure class; Display IS the brief.
    /// Two spans land in the diagnostic so users (and agents)
    /// navigate to BOTH the offending invocation AND the helper
    /// they meant to reference.
    SandboxScopeLeak {
        /// The keyword name invoked at the call site.
        offending_name: String,
        /// Source location of the offending invocation inside the
        /// sandboxed body.
        call_span: Span,
        /// Source location of the outer-scope define. Best-effort:
        /// uses the function's body span when the substrate doesn't
        /// track the outer define-form span directly. May be
        /// `Span::unknown()` if the outer scope is a built-in.
        outer_define_span: Span,
    },
}

/// Arc 138 slice 1 — render the file:line:col prefix for an error,
/// or empty when the span is unknown (synthetic check rule with no
/// originating node). The prefix shape mirrors `ScopeDeadlock` /
/// `ChannelPairDeadlock` / `BareLegacyPrimitive` — `<file>:<line>:<col>: `.
fn span_prefix(span: &Span) -> String {
    if span.is_unknown() {
        String::new()
    } else {
        format!("{}: ", span)
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckError::ArityMismatch { callee, expected, got, span } => write!(
                f,
                "{}{}: expected {} argument(s); got {}",
                span_prefix(span), callee, expected, got
            ),
            CheckError::TypeMismatch {
                callee,
                param,
                expected,
                got,
                span,
            } => {
                write!(
                    f,
                    "{}{}: parameter {} expects {}; got {}",
                    span_prefix(span), callee, param, expected, got
                )?;
                if let Some(hint) = collect_hints(callee, expected, got) {
                    write!(f, "\n  hint: {}", hint)?;
                }
                Ok(())
            }
            CheckError::ReturnTypeMismatch {
                function,
                expected,
                got,
                span,
            } => {
                write!(
                    f,
                    "{}{}: body produces {}; signature declares {}",
                    span_prefix(span), function, got, expected
                )?;
                if let Some(hint) = collect_hints(function, expected, got) {
                    write!(f, "\n  hint: {}", hint)?;
                }
                Ok(())
            }
            CheckError::UnknownCallee { callee, span } => {
                write!(f, "{}unknown callee: {}", span_prefix(span), callee)
            }
            CheckError::MalformedForm { head, reason, span } => {
                write!(f, "{}malformed {} form: {}", span_prefix(span), head, reason)
            }
            CheckError::CommCallOutOfPosition { callee, span } => write!(
                f,
                "{}{} may appear only as the scrutinee of `:wat::core::match`, the value-position of `:wat::core::Result/expect`, or the value-position of `:wat::core::Option/expect`; silent disconnect must be handled at every comm call",
                span_prefix(span), callee
            ),
            CheckError::ScopeDeadlock {
                thread_binding,
                offending_binding,
                offending_kind,
                span,
            } => write!(
                f,
                "scope-deadlock at {}: Thread/join-result on '{}' would block. Sibling binding '{}' (a {}) holds a Sender clone that outlives the worker; the worker's recv never sees EOF. Fix: nest the {} binding (and any other Sender clones) in an inner let* whose body returns '{}' — outer scope holds only the Thread. SERVICE-PROGRAMS.md § \"The lockstep\".",
                span, thread_binding, offending_binding, offending_kind, offending_kind, thread_binding
            ),
            CheckError::ChannelPairDeadlock {
                callee,
                sender_arg,
                receiver_arg,
                pair_anchor,
                span,
            } => write!(
                f,
                "channel-pair-deadlock at {}: function call '{}' receives two halves of the same channel pair. Argument '{}' is a Sender<T> and argument '{}' is a Receiver<T>; both trace back to the make-bounded-channel allocation at '{}' (let* binding above). Holding both ends of one channel in one role deadlocks any recv — the caller's writer keeps the channel alive even when the receiving thread dies. Fix options (per ZERO-MUTEX.md § \"Routing acks\"): 1. Pair-by-index via HandlePool — each producer pops one Handle holding ONE end of EACH of two distinct channels. 2. Embedded reply-tx in payload — caller does not bind the reply-tx; project the Sender directly into the Request.",
                span, callee, sender_arg, receiver_arg, pair_anchor
            ),
            CheckError::BareLegacyPrimitive { primitive, fqdn, span } => write!(
                f,
                "bare primitive type '{}' at {} is retired (arc 109 slice 1c); canonical FQDN form is '{}'. Substrate-provided primitives live under :wat::core::* (see arc 109 § A). Rename '{}' → '{}' at the offending site.",
                primitive, span, fqdn, primitive, fqdn
            ),
            CheckError::BareLegacyUnitType { span } => write!(
                f,
                "bare unit type '()' at {} is retired (arc 109 slice 1d); canonical FQDN form is ':wat::core::unit'. Substrate-provided primitives live under :wat::core::* (see arc 109 § A). The empty-tuple LITERAL VALUE `()` is unaffected; only the type-position spelling renames. Rename ':()' → ':wat::core::unit' (or '()' → 'wat::core::unit' inside parametrics) at the offending site.",
                span
            ),
            CheckError::BareLegacyContainerHead { head, fqdn, span } => write!(
                f,
                "bare container type '{}' at {} is retired (arc 109 slice 1e); canonical FQDN form is '{}'. Substrate-provided container types live under :wat::core::* (see arc 109 § B). Rename '{}' → '{}' at the offending site (works in both outer position like ':{}' → ':{}' and inner position like 'Vec<{}>' → 'Vec<{}>').",
                head, span, fqdn, head, fqdn, head, fqdn, head, fqdn
            ),
            CheckError::BareLegacyStreamPath { old, new, span } => write!(
                f,
                "legacy stream path '{}' at {} is retired (arc 109 slice 9d); canonical form is '{}'. The stream stdlib graduated to :wat::stream::* per § G's three-tier substrate organization (every substrate concern earns its own top-level tier; :wat::std::* empties out). File path mirrors: wat/std/stream.wat → wat/stream.wat. Rename '{}' → '{}' at the offending site.",
                old, span, new, old, new
            ),
            CheckError::BareLegacyTelemetryServicePath { old, new, span } => write!(
                f,
                "legacy telemetry-service path '{}' at {} is retired (arc 109 slice K.telemetry); canonical form is '{}'. The :wat::telemetry::Service grouping noun retired per § K's '/ requires a real Type' doctrine — Service has no struct, no value, no kind. Verbs and typealiases live at the namespace level. Real types Stats and MetricsCadence keep their PascalCase + /methods because they ARE structs (just one less namespace segment deep). Rename '{}' → '{}' at the offending site.",
                old, span, new, old, new
            ),
            CheckError::BareLegacyConsolePath { old, new, span } => write!(
                f,
                "legacy console path '{}' at {} is retired (arc 109 slice K.console); canonical form is '{}'. The :wat::std::service::Console grouping noun retired per § K's '/ requires a real Type' doctrine. Plus Pattern A canonicalization: Tx/Rx renamed to ReqTx/ReqRx (mirrors Telemetry's Pattern A reference shape). File moved: wat/std/service/Console.wat → wat/console.wat. Rename '{}' → '{}' at the offending site.",
                old, span, new, old, new
            ),
            CheckError::BareLegacyLruCacheServicePath { old, new, span } => write!(
                f,
                "legacy lru-cache-service path '{}' at {} is retired (arc 109 slice K.lru); canonical form is '{}'. The :wat::lru::CacheService grouping noun retired per § K's '/ requires a real Type' doctrine. Real types Stats / MetricsCadence / State / Report keep PascalCase + /methods (just one less namespace segment). Plus Pattern B canonicalization: ReqPair renamed to ReqChannel (in-crate ReqPair/ReplyChannel mumble); ReplyRx<V> + ReplyChannel<V> typealiases minted to complete the Pattern B reference. Rename '{}' → '{}' at the offending site.",
                old, span, new, old, new
            ),
            CheckError::BareLegacyKernelQueuePath { old, new, span } => write!(
                f,
                "legacy kernel queue path '{}' at {} is retired (arc 109 slice K.kernel-channel); canonical form is '{}'. The :wat::kernel::Queue* family renamed to Channel / Sender / Receiver (Queue leaked crossbeam's data-structure name; the canonical vocabulary is the substrate's honest naming). File moved: wat/kernel/queue.wat → wat/kernel/channel.wat. Rename '{}' → '{}' at the offending site.",
                old, span, new, old, new
            ),
            CheckError::SandboxScopeLeak { offending_name, call_span, outer_define_span } => {
                let define_loc = if outer_define_span.is_unknown() {
                    "an outer scope".to_string()
                } else {
                    format!("{}", outer_define_span)
                };
                write!(
                    f,
                    "{}sandbox-scope leak: '{}' invoked here is defined at {} but deftest sandboxes do NOT capture outer-scope. Move (:wat::core::define {} ...) into this deftest's prelude (the second argument of `(:wat::test::deftest <name> <prelude> <body>)`), or load it into the prelude via `(:wat::core::load! \"path/to/file.wat\")`. The sandbox isolation is intentional — see wat/test.wat's deftest macro.",
                    span_prefix(call_span), offending_name, define_loc, offending_name
                )
            }
        }
    }
}

impl std::error::Error for CheckError {}

// Arc 111 / 112 / 113 migration-hint helpers retired 2026-04-30.
// Each shipped with an `arc_N_migration_hint(callee, expected, got)
// -> Option<String>` function that `collect_hints` invoked on every
// `TypeMismatch` + `ReturnTypeMismatch`. Each hint self-identified
// via its leading `"arc N — "` prefix and described the one-token
// annotation fix the substrate-as-teacher pattern relied on for
// sonnet fixture sweeps.
//
// Once each arc's consumer wave had been swept, the helper retired
// — the hint had served its job.
//
// 2026-04-30 (later): arc 114 reintroduces the pattern for
// `:wat::kernel::spawn` retirement. See `arc_114_migration_hint`
// below.

/// Arc 114 — fires on bare-spawn callees and on the
/// `:ProgramHandle<R>` ↔ `:Thread<I,O>` shape pair. Tells the reader
/// the migration path to `:wat::kernel::spawn-thread` +
/// `:wat::kernel::Thread<I,O>` + `:wat::kernel::Thread/join-result`.
///
/// The hint frames mini-TCP (`docs/ZERO-MUTEX.md` § "Mini-TCP via
/// paired channels") as the primary worker shape — most existing
/// `:wat::kernel::spawn` callers close over caller-allocated
/// `make-bounded-queue` pairs and don't need substrate-allocated
/// channels. Workers that don't fit that mold get a manual flag
/// (`;; ARC 114 MANUAL`) — substrate-author judgment calls don't
/// auto-sweep.
fn arc_114_migration_hint(callee: &str, expected: &str, got: &str) -> Option<String> {
    let bare_spawn_callee = matches!(
        callee,
        ":wat::kernel::spawn"
            | ":wat::kernel::join"
            | ":wat::kernel::join-result"
    );
    // Annotation-leftover smell: a binding declared :ProgramHandle<R>
    // but the value is :Thread<I,O>, or vice versa. Detects partial
    // sweeps where sonnet flipped the verb but left the binding type.
    let proghandle = "wat::kernel::ProgramHandle<";
    let thread = "wat::kernel::Thread<";
    let shape_pair_mismatch = (expected.contains(proghandle) && got.contains(thread))
        || (got.contains(proghandle) && expected.contains(thread));
    if !bare_spawn_callee && !shape_pair_mismatch {
        return None;
    }
    Some(
        "arc 114 — :wat::kernel::spawn / :wat::kernel::join / \
         :wat::kernel::join-result retire. Programs deliver values only \
         via their output channel; R-via-join is gone. \
         Migrate: (:wat::kernel::spawn :worker args...) → \
         (:wat::kernel::spawn-thread (:wat::core::lambda \
         ((_in :rust::crossbeam_channel::Receiver<()>) \
         (_out :rust::crossbeam_channel::Sender<()>)) (:worker args...))) \
         returning :wat::kernel::Thread<(),()>. \
         Replace (:wat::kernel::join h) and (:wat::kernel::join-result h) \
         with (:wat::kernel::Thread/join-result thr) returning \
         :Result<:(),:Vec<wat::kernel::ThreadDiedError>>; match arms \
         ((Ok _) ...) ((Err chain) ...). \
         Mini-TCP workers (docs/ZERO-MUTEX.md) close over caller-held \
         channels; substrate-allocated `_in` / `_out` stay unused. \
         Workers not fitting :Fn(:Receiver<I>, :Sender<O>) -> :() — \
         non-channel sig, non-unit return, R-via-join ferrying — get a \
         `;; ARC 114 MANUAL — needs type-design review` comment and skip; \
         judgment calls don't auto-sweep."
            .into(),
    )
}

/// Aggregated errors — `check_program` returns all findings together.
#[derive(Debug)]
pub struct CheckErrors(pub Vec<CheckError>);

impl fmt::Display for CheckErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} type-check error(s):", self.0.len())?;
        for e in &self.0 {
            writeln!(f, "  - {}", e)?;
        }
        Ok(())
    }
}

impl std::error::Error for CheckErrors {}

impl CheckError {
    /// Arc 115 slice 1 — produce a structured [`Diagnostic`] for this
    /// error variant. Renderers (text via Display, EDN, JSON) consume
    /// the same data without parsing string forms.
    ///
    /// Each variant maps to one `Diagnostic` with `kind` = the variant
    /// name and field-name → field-value pairs that mirror the Rust
    /// struct fields. **One `hint` field, optional**: present when the
    /// substrate has migration guidance for the error; absent otherwise.
    /// Multiple applicable hints (e.g., arc 111 + arc 112 both firing
    /// on the same TypeMismatch) join with a blank line — each hint
    /// already self-identifies via its leading `"arc N — "` prefix.
    pub fn diagnostic(&self) -> crate::diagnostic::Diagnostic {
        use crate::diagnostic::Diagnostic;
        match self {
            CheckError::ArityMismatch { callee, expected, got, span } => {
                let mut diag = Diagnostic::new("ArityMismatch")
                    .field("callee", callee.as_str())
                    .field("expected", *expected)
                    .field("got", *got);
                if !span.is_unknown() {
                    diag = diag.field("span", span.to_string());
                }
                diag
            }
            CheckError::TypeMismatch { callee, param, expected, got, span } => {
                let mut diag = Diagnostic::new("TypeMismatch")
                    .field("callee", callee.as_str())
                    .field("param", param.as_str())
                    .field("expected", expected.as_str())
                    .field("got", got.as_str());
                if !span.is_unknown() {
                    diag = diag.field("span", span.to_string());
                }
                if let Some(hint) = collect_hints(callee, expected, got) {
                    diag = diag.field("hint", hint);
                }
                diag
            }
            CheckError::ReturnTypeMismatch { function, expected, got, span } => {
                let mut diag = Diagnostic::new("ReturnTypeMismatch")
                    .field("function", function.as_str())
                    .field("expected", expected.as_str())
                    .field("got", got.as_str());
                if !span.is_unknown() {
                    diag = diag.field("span", span.to_string());
                }
                if let Some(hint) = collect_hints(function, expected, got) {
                    diag = diag.field("hint", hint);
                }
                diag
            }
            CheckError::UnknownCallee { callee, span } => {
                let mut diag = Diagnostic::new("UnknownCallee").field("callee", callee.as_str());
                if !span.is_unknown() {
                    diag = diag.field("span", span.to_string());
                }
                diag
            }
            CheckError::MalformedForm { head, reason, span } => {
                let mut diag = Diagnostic::new("MalformedForm")
                    .field("head", head.as_str())
                    .field("reason", reason.as_str());
                if !span.is_unknown() {
                    diag = diag.field("span", span.to_string());
                }
                diag
            }
            CheckError::CommCallOutOfPosition { callee, span } => {
                let mut diag = Diagnostic::new("CommCallOutOfPosition")
                    .field("callee", callee.as_str());
                if !span.is_unknown() {
                    diag = diag.field("span", span.to_string());
                }
                diag
            }
            CheckError::ScopeDeadlock {
                thread_binding,
                offending_binding,
                offending_kind,
                span,
            } => Diagnostic::new("ScopeDeadlock")
                .field("thread_binding", thread_binding.as_str())
                .field("offending_binding", offending_binding.as_str())
                .field("offending_kind", *offending_kind)
                .field("location", format!("{}", span)),
            CheckError::ChannelPairDeadlock {
                callee,
                sender_arg,
                receiver_arg,
                pair_anchor,
                span,
            } => Diagnostic::new("ChannelPairDeadlock")
                .field("callee", callee.as_str())
                .field("sender_arg", sender_arg.as_str())
                .field("receiver_arg", receiver_arg.as_str())
                .field("pair_anchor", pair_anchor.as_str())
                .field("location", format!("{}", span)),
            CheckError::BareLegacyPrimitive { primitive, fqdn, span } => {
                Diagnostic::new("BareLegacyPrimitive")
                    .field("primitive", primitive.as_str())
                    .field("fqdn", fqdn.as_str())
                    .field("location", format!("{}", span))
            }
            CheckError::BareLegacyUnitType { span } => {
                Diagnostic::new("BareLegacyUnitType")
                    .field("primitive", ":()")
                    .field("fqdn", ":wat::core::unit")
                    .field("location", format!("{}", span))
            }
            CheckError::BareLegacyContainerHead { head, fqdn, span } => {
                Diagnostic::new("BareLegacyContainerHead")
                    .field("head", head.as_str())
                    .field("fqdn", fqdn.as_str())
                    .field("location", format!("{}", span))
            }
            CheckError::BareLegacyStreamPath { old, new, span } => {
                Diagnostic::new("BareLegacyStreamPath")
                    .field("old", old.as_str())
                    .field("new", new.as_str())
                    .field("location", format!("{}", span))
            }
            CheckError::BareLegacyTelemetryServicePath { old, new, span } => {
                Diagnostic::new("BareLegacyTelemetryServicePath")
                    .field("old", old.as_str())
                    .field("new", new.as_str())
                    .field("location", format!("{}", span))
            }
            CheckError::BareLegacyConsolePath { old, new, span } => {
                Diagnostic::new("BareLegacyConsolePath")
                    .field("old", old.as_str())
                    .field("new", new.as_str())
                    .field("location", format!("{}", span))
            }
            CheckError::BareLegacyLruCacheServicePath { old, new, span } => {
                Diagnostic::new("BareLegacyLruCacheServicePath")
                    .field("old", old.as_str())
                    .field("new", new.as_str())
                    .field("location", format!("{}", span))
            }
            CheckError::BareLegacyKernelQueuePath { old, new, span } => {
                Diagnostic::new("BareLegacyKernelQueuePath")
                    .field("old", old.as_str())
                    .field("new", new.as_str())
                    .field("location", format!("{}", span))
            }
            CheckError::SandboxScopeLeak { offending_name, call_span, outer_define_span } => {
                let mut diag = Diagnostic::new("SandboxScopeLeak")
                    .field("offending_name", offending_name.as_str())
                    .field("call_span", format!("{}", call_span));
                if !outer_define_span.is_unknown() {
                    diag = diag.field("outer_define_span", format!("{}", outer_define_span));
                }
                diag
            }
        }
    }
}

/// Collect all migration hints that fire for this (callee, expected,
/// got) triple into a single string. Each hint already self-identifies
/// via its leading `"arc N — "` prefix; we just concatenate.
///
/// Returns `None` when no hint applies — currently the steady state
/// (arcs 111 / 112 / 113 retired their helpers 2026-04-30 once the
/// respective consumer waves swept clean). The function stays as
/// scaffold for future arcs: add `arc_NNN_migration_hint(callee,
/// expected, got)` to the array below; the rest of the substrate
/// (Display impl + `CheckError::diagnostic`'s hint field) picks it
/// up automatically. See the retirement note above for the helper
/// shape.
/// Arc 109 slice 1f — fires when the dispatcher has poisoned the
/// retired `:wat::core::vec` head. The hint names the canonical
/// replacement (`:wat::core::Vector`, verb-equals-type per
/// INVENTORY § D) and the literal swap.
fn arc_109_vec_verb_migration_hint(callee: &str, _expected: &str, _got: &str) -> Option<String> {
    if callee != ":wat::core::vec" {
        return None;
    }
    Some(
        "arc 109 slice 1f — `:wat::core::vec` is retired. Canonical \
         constructor is `:wat::core::Vector` (verb-equals-type per \
         INVENTORY § D — `(:wat::core::Vector :T x y z)` reads as \
         `construct a Vector of T from these elements`). Rename \
         `:wat::core::vec` → `:wat::core::Vector` at the offending \
         site. The substrate produces the same `Vec<T>` value; only \
         the spelling changes."
            .into(),
    )
}

/// Arc 109 slice 1g — fires when the dispatcher has poisoned the
/// retired `:wat::core::list` head. `list` was always a duplicate
/// of `vec` (both produced `Vec<T>`); post-slice-1f the canonical
/// constructor is `:wat::core::Vector`. The redundancy retires
/// in this slice.
fn arc_109_list_verb_migration_hint(callee: &str, _expected: &str, _got: &str) -> Option<String> {
    if callee != ":wat::core::list" {
        return None;
    }
    Some(
        "arc 109 slice 1g — `:wat::core::list` is retired. It was \
         always a duplicate of `:wat::core::vec` (now \
         `:wat::core::Vector` post-slice-1f); both produced \
         `Vec<T>`. The redundancy goes; rename `:wat::core::list` \
         → `:wat::core::Vector` at the offending site. The \
         substrate produces the same `Vec<T>` value; only the \
         spelling changes."
            .into(),
    )
}

/// Arc 109 slice 1g — fires when the dispatcher has poisoned the
/// retired `:wat::core::tuple` head. The canonical constructor is
/// `:wat::core::Tuple` (verb-equals-type per slice 1f's
/// vec→Vector playbook).
fn arc_109_tuple_verb_migration_hint(callee: &str, _expected: &str, _got: &str) -> Option<String> {
    if callee != ":wat::core::tuple" {
        return None;
    }
    Some(
        "arc 109 slice 1g — `:wat::core::tuple` is retired. \
         Canonical constructor is `:wat::core::Tuple` \
         (verb-equals-type per slice 1f's vec→Vector playbook — \
         `(:wat::core::Tuple x y z)` reads as `construct a Tuple \
         of these elements`). Rename `:wat::core::tuple` → \
         `:wat::core::Tuple` at the offending site. The substrate \
         produces the same tuple value; only the spelling changes. \
         The TYPE spelling `:(T,U,V)` is parsed separately and is \
         unaffected."
            .into(),
    )
}

/// Arc 109 slice 1h — fires when the dispatcher has poisoned the
/// bare-Symbol `Some` head (a retired grammar exception). The
/// canonical FQDN form is `:wat::core::Some`.
fn arc_109_some_variant_migration_hint(callee: &str, _expected: &str, _got: &str) -> Option<String> {
    if callee != "Some" {
        return None;
    }
    Some(
        "arc 109 slice 1h — bare `Some` is a retiring grammar \
         exception (wat's general rule: callable heads must be \
         FQDN keywords). Canonical form is `:wat::core::Some`. \
         Rename `(Some x)` → `(:wat::core::Some x)` at \
         constructor sites; rename `((Some v) ...)` → \
         `((:wat::core::Some v) ...)` at match-pattern sites. \
         The substrate produces the same `Option<T>` value; only \
         the spelling changes."
            .into(),
    )
}

/// Arc 109 slice 1h — fires when the dispatcher has poisoned the
/// bare keyword `:None` (a retired grammar exception). The
/// canonical FQDN form is `:wat::core::None`.
fn arc_109_none_variant_migration_hint(callee: &str, _expected: &str, _got: &str) -> Option<String> {
    if callee != ":None" {
        return None;
    }
    Some(
        "arc 109 slice 1h — bare `:None` is a retiring grammar \
         exception (substrate-provided keywords live under \
         `:wat::core::*`). Canonical form is `:wat::core::None`. \
         Rename `:None` → `:wat::core::None` at value-position \
         sites; rename `(:None ...)` → `(:wat::core::None ...)` \
         at match-pattern sites. The substrate produces the same \
         `Option<T>` (None) value; only the spelling changes."
            .into(),
    )
}

/// Arc 109 slice 1i — fires when the dispatcher has poisoned the
/// bare-Symbol `Ok` head. Mirrors slice 1h's Some hint.
fn arc_109_ok_variant_migration_hint(callee: &str, _expected: &str, _got: &str) -> Option<String> {
    if callee != "Ok" {
        return None;
    }
    Some(
        "arc 109 slice 1i — bare `Ok` is a retiring grammar \
         exception (wat's general rule: callable heads must be \
         FQDN keywords). Canonical form is `:wat::core::Ok`. \
         Rename `(Ok x)` → `(:wat::core::Ok x)` at constructor \
         sites; rename `((Ok v) ...)` → `((:wat::core::Ok v) ...)` \
         at match-pattern sites. The substrate produces the same \
         `Result<T,E>` value; only the spelling changes."
            .into(),
    )
}

/// Arc 109 slice 1i — fires when the dispatcher has poisoned the
/// bare-Symbol `Err` head. Mirrors slice 1h's Some hint.
fn arc_109_err_variant_migration_hint(callee: &str, _expected: &str, _got: &str) -> Option<String> {
    if callee != "Err" {
        return None;
    }
    Some(
        "arc 109 slice 1i — bare `Err` is a retiring grammar \
         exception (wat's general rule: callable heads must be \
         FQDN keywords). Canonical form is `:wat::core::Err`. \
         Rename `(Err e)` → `(:wat::core::Err e)` at constructor \
         sites; rename `((Err _e) ...)` → `((:wat::core::Err _e) ...)` \
         at match-pattern sites. The substrate produces the same \
         `Result<T,E>` value; only the spelling changes."
            .into(),
    )
}

/// Arc 109 slice 1j — fires when the dispatcher has poisoned the
/// retired `:wat::core::try` head. Canonical form is
/// `:wat::core::Result/try` (per the `Type/verb` shape minted in
/// slice 1j; see `INVENTORY.md` § D').
fn arc_109_try_verb_migration_hint(callee: &str, _expected: &str, _got: &str) -> Option<String> {
    if callee != ":wat::core::try" {
        return None;
    }
    Some(
        "arc 109 slice 1j — `:wat::core::try` is retired. Canonical \
         form is `:wat::core::Result/try` (the Result-side method-form \
         per § D' — sibling to the brand-new `:wat::core::Option/try` \
         minted in the same slice). Rename `:wat::core::try` → \
         `:wat::core::Result/try` at the offending site. The substrate \
         produces the same value; only the spelling changes."
            .into(),
    )
}

/// Arc 109 slice 1j — fires when the dispatcher has poisoned the
/// retired `:wat::core::option::expect` head. Canonical form is
/// `:wat::core::Option/expect` (PascalCase Type + slash-verb).
fn arc_109_option_expect_migration_hint(
    callee: &str,
    _expected: &str,
    _got: &str,
) -> Option<String> {
    if callee != ":wat::core::option::expect" {
        return None;
    }
    Some(
        "arc 109 slice 1j — `:wat::core::option::expect` is retired. \
         Canonical form is `:wat::core::Option/expect` (PascalCase \
         Type + slash-verb per § D' — matches the `Stats/new` / \
         `HandlePool/pop` family). Rename \
         `:wat::core::option::expect` → `:wat::core::Option/expect` \
         at the offending site. The shape (-> :T <opt> <msg>) and \
         semantics (panic on :None with msg) are unchanged."
            .into(),
    )
}

/// Arc 109 slice 1j — fires when the dispatcher has poisoned the
/// retired `:wat::core::result::expect` head. Canonical form is
/// `:wat::core::Result/expect`.
fn arc_109_result_expect_migration_hint(
    callee: &str,
    _expected: &str,
    _got: &str,
) -> Option<String> {
    if callee != ":wat::core::result::expect" {
        return None;
    }
    Some(
        "arc 109 slice 1j — `:wat::core::result::expect` is retired. \
         Canonical form is `:wat::core::Result/expect` (PascalCase \
         Type + slash-verb per § D'). Rename \
         `:wat::core::result::expect` → `:wat::core::Result/expect` \
         at the offending site. The shape (-> :T <res> <msg>) and \
         semantics (panic on Err with msg, carrying any \
         `Vec<*DiedError>` chain) are unchanged."
            .into(),
    )
}

fn collect_hints(callee: &str, expected: &str, got: &str) -> Option<String> {
    let hints: Vec<String> = [
        arc_114_migration_hint(callee, expected, got),
        arc_109_vec_verb_migration_hint(callee, expected, got),
        arc_109_list_verb_migration_hint(callee, expected, got),
        arc_109_tuple_verb_migration_hint(callee, expected, got),
        arc_109_some_variant_migration_hint(callee, expected, got),
        arc_109_none_variant_migration_hint(callee, expected, got),
        arc_109_ok_variant_migration_hint(callee, expected, got),
        arc_109_err_variant_migration_hint(callee, expected, got),
        arc_109_try_verb_migration_hint(callee, expected, got),
        arc_109_option_expect_migration_hint(callee, expected, got),
        arc_109_result_expect_migration_hint(callee, expected, got),
    ]
    .into_iter()
    .flatten()
    .collect();
    if hints.is_empty() {
        None
    } else {
        Some(hints.join("\n\n"))
    }
}

impl CheckErrors {
    /// Arc 115 slice 1 — produce one [`Diagnostic`] per CheckError.
    pub fn diagnostics(&self) -> Vec<crate::diagnostic::Diagnostic> {
        self.0.iter().map(|e| e.diagnostic()).collect()
    }
}

/// Cross-cutting context threaded through every `infer_*` helper.
/// Owns two concerns that need global scope during a single
/// `check_program` run:
///
/// 1. **Fresh type-variable ids.** A monotonic counter that hands out
///    unique `TypeExpr::Var(n)` ids so distinct unification variables
///    never collide across call sites or function bodies.
/// 2. **Enclosing return-type stack.** Pushed on entry to every
///    function body and lambda body, popped on exit, consulted by
///    `infer_try` to unify the propagated `E` with the enclosing
///    function/lambda's own `Err` variant. LIFO so the innermost
///    enclosing scope wins — matches Rust's `?`-operator scoping.
///
/// The parameter name in most call sites is still `fresh` by
/// convention — the ctx's primary role was originally just fresh-var
/// generation, and the shorter name reads naturally for that case.
/// New concerns added here (scoped flags, effect rows, whatever) land
/// as additional fields without further renames.
#[derive(Debug, Default)]
struct InferCtx {
    next: u64,
    enclosing_rets: Vec<TypeExpr>,
}

impl InferCtx {
    fn fresh(&mut self) -> TypeExpr {
        let v = TypeExpr::Var(self.next);
        self.next += 1;
        v
    }

    /// Push the declared return type of a function/lambda we are about
    /// to check. Paired with [`pop_enclosing_ret`].
    fn push_enclosing_ret(&mut self, ret: TypeExpr) {
        self.enclosing_rets.push(ret);
    }

    /// Pop the most recently pushed return type. Caller is responsible
    /// for pairing pushes and pops at scope boundaries.
    fn pop_enclosing_ret(&mut self) {
        self.enclosing_rets.pop();
    }

    /// Innermost enclosing return type, if any. `None` outside any
    /// function/lambda body (top-level `check_form` invocations).
    fn enclosing_ret(&self) -> Option<&TypeExpr> {
        self.enclosing_rets.last()
    }
}

/// Substitution map: unification variable id → its concrete type.
/// Updated as unification succeeds; applied via [`apply_subst`] to
/// resolve a type to its canonical form.
type Subst = HashMap<u64, TypeExpr>;

/// The type-check environment: built-in + user function schemes plus
/// a shared handle to the [`TypeEnv`] (user type declarations).
/// Unification consults the type-env to expand typealiases to their
/// structural definitions before the structural match.
#[derive(Debug)]
pub struct CheckEnv {
    schemes: HashMap<String, TypeScheme>,
    /// Arc 048 — keyword paths for user-enum unit variants mapped to
    /// the enum's type. When `infer` sees one of these as a value-
    /// position keyword (e.g. `:trading::types::PhaseLabel::Valley`),
    /// it returns the enum's type instead of the generic
    /// `:wat::core::keyword`. Mirrors the runtime's
    /// `SymbolTable.unit_variants`. Populated at construction by
    /// walking every `:wat::core::enum` declaration in `types`.
    unit_variant_types: HashMap<String, TypeExpr>,
    types: Arc<TypeEnv>,
}

impl CheckEnv {
    pub fn new() -> Self {
        Self::with_types(Arc::new(TypeEnv::with_builtins()))
    }

    /// Build an env with built-in schemes for `:wat::core::*` and
    /// `:wat::holon::*` forms, then overlay user-define signatures
    /// from `sym`. `types` carries the registered user type
    /// declarations (struct/enum/newtype/typealias) — unification uses
    /// it to expand aliases.
    pub fn from_symbols(sym: &SymbolTable, types: Arc<TypeEnv>) -> Self {
        let mut env = Self::with_builtins_and_types(types);
        for (path, func) in &sym.functions {
            if let Some(scheme) = derive_scheme_from_function(func) {
                env.register(path.clone(), scheme);
            }
        }
        env
    }

    pub fn with_builtins() -> Self {
        Self::with_builtins_and_types(Arc::new(TypeEnv::with_builtins()))
    }

    pub fn with_builtins_and_types(types: Arc<TypeEnv>) -> Self {
        let mut env = Self::with_types(types);
        register_builtins(&mut env);
        env
    }

    fn with_types(types: Arc<TypeEnv>) -> Self {
        // Arc 048 — pre-populate unit-variant keyword types from the
        // declared enums. Walks every TypeDef::Enum and registers each
        // unit variant's full keyword path (`:enum::Variant`) → enum
        // type, so `infer` can return the enum type when the bare
        // keyword appears in expression position.
        let mut unit_variant_types = HashMap::new();
        for (name, def) in types.iter() {
            if let crate::types::TypeDef::Enum(e) = def {
                for variant in &e.variants {
                    if let crate::types::EnumVariant::Unit(variant_name) = variant {
                        let key = format!("{}::{}", name, variant_name);
                        unit_variant_types.insert(key, TypeExpr::Path(name.clone()));
                    }
                }
            }
        }
        CheckEnv {
            schemes: HashMap::new(),
            unit_variant_types,
            types,
        }
    }

    /// Arc 048 — look up the enum type for a unit-variant keyword
    /// path. Returns `None` for non-variant keywords.
    pub fn unit_variant_type(&self, key: &str) -> Option<&TypeExpr> {
        self.unit_variant_types.get(key)
    }

    pub fn register(&mut self, name: String, scheme: TypeScheme) {
        self.schemes.insert(name, scheme);
    }

    pub fn get(&self, name: &str) -> Option<&TypeScheme> {
        self.schemes.get(name)
    }

    /// Handle to the user/builtin type declarations. Used by `unify`
    /// to expand typealiases to their structural form before the
    /// structural match.
    pub fn types(&self) -> &TypeEnv {
        &self.types
    }
}

impl Default for CheckEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Check every user define's body against its declared return type;
/// verify every call-position form in the `forms` list has correct
/// arity and argument types.
///
/// Reports all errors found in a single pass.
pub fn check_program(
    forms: &[WatAST],
    sym: &SymbolTable,
    types: &TypeEnv,
) -> Result<(), CheckErrors> {
    let env = CheckEnv::from_symbols(sym, Arc::new(types.clone()));
    let mut errors = Vec::new();
    let mut fresh = InferCtx::default();

    // Arc 110 — every kernel-comm call must land in match-scrutinee
    // or option::expect-value position. Run the walk before inference
    // so a misplaced send/recv reports as the structural problem it
    // is, not as a downstream type mismatch.
    for func in sym.functions.values() {
        validate_comm_positions(&func.body, CommCtx::Forbidden, &mut errors);
    }
    for form in forms {
        validate_comm_positions(form, CommCtx::Forbidden, &mut errors);
    }

    // Arc 117 / Arc 133 — scope-deadlock prevention. The structural
    // pre-inference walker (`validate_scope_deadlock`) was retired by
    // arc 133: the rule is now enforced in-place inside
    // `infer_let_star` after all bindings are processed. That path
    // uses inferred TypeExprs from the `extended` scope map, covering
    // BOTH typed-name bindings (the original arc 117 shape) AND
    // untyped tuple-destructure bindings (the arc 133 gap).
    // Duplicate-firing is impossible: inference drives both coverage
    // and classification; the structural walker was redundant once
    // the inference path fired for the same shapes.
    //
    // If you see this comment and are wondering why ScopeDeadlock
    // fires at type-check time: look for `check_let_star_for_scope_deadlock_inferred`
    // in `infer_let_star` (arc 133 slice 1).

    // Arc 126 — refuse to compile a function-call site that passes
    // BOTH halves of one `make-bounded-channel` / `make-unbounded-channel`
    // pair. Sibling rule to ScopeDeadlock — same trace machinery
    // applied at call sites instead of spawn-thread closure bodies.
    // Catches the arc 119 "Pattern B Put-ack helper-verb cycle"
    // deadlock at type-check time: when `tx` and `rx` both project
    // off the same pair-anchor (`(first pair)` / `(second pair)`
    // with `pair = (make-bounded-channel ...)`), passing them to
    // one helper guarantees the helper's recv never sees EOF —
    // the caller's Sender clone keeps the channel alive even when
    // the receiver should disconnect.
    for func in sym.functions.values() {
        validate_channel_pair_deadlock(&func.body, types, &mut errors);
    }
    for form in forms {
        validate_channel_pair_deadlock(form, types, &mut errors);
    }

    // Arc 140 — sandbox-scope leak prevention. Walk every form for
    // sandbox-primitive call sites. For each, build the inner-scope
    // name set from the forms-block; walk inner-form bodies; fire
    // `SandboxScopeLeak` when a call head resolves in the OUTER
    // SymbolTable but NOT in the inner scope. The diagnostic carries
    // both spans (offending invocation + outer-scope define) so users
    // and agents navigate without grepping. See arc 140 DESIGN.md.
    for func in sym.functions.values() {
        validate_sandbox_scope_leak(&func.body, sym, &mut errors);
    }
    for form in forms {
        validate_sandbox_scope_leak(form, sym, &mut errors);
    }

    // Arc 109 slice 1c — refuse bare primitive types (`:i64`, `:f64`,
    // `:bool`, `:String`, `:u8`) anywhere in the program. The
    // canonical FQDN form (`:wat::core::i64`, etc.) is the
    // substitute. Walks every keyword token in the AST, scans for
    // bare-primitive substrings at type-token boundaries, emits one
    // BareLegacyPrimitive per occurrence.
    for func in sym.functions.values() {
        validate_bare_legacy_primitives(&func.body, &mut errors);
    }
    for form in forms {
        validate_bare_legacy_primitives(form, &mut errors);
    }

    // Arc 109 slice 9d — refuse the legacy `:wat::std::stream::*`
    // namespace prefix anywhere in the program. The stream stdlib
    // graduated to `:wat::stream::*` per § G's three-tier substrate
    // organization. Walks every keyword token in the AST and emits
    // one BareLegacyStreamPath per occurrence.
    for func in sym.functions.values() {
        validate_legacy_stream_path(&func.body, &mut errors);
    }
    for form in forms {
        validate_legacy_stream_path(form, &mut errors);
    }

    // Arc 109 slice K.telemetry — refuse the legacy
    // `:wat::telemetry::Service::*` (typealias) and
    // `:wat::telemetry::Service/*` (verb) prefixes. The Service
    // grouping noun retires per § K's "/ requires a real Type"
    // doctrine; verbs and typealiases live at the namespace level.
    // Real types Stats and MetricsCadence keep their /methods.
    for func in sym.functions.values() {
        validate_legacy_telemetry_service_path(&func.body, &mut errors);
    }
    for form in forms {
        validate_legacy_telemetry_service_path(form, &mut errors);
    }

    // Arc 109 slice K.console — refuse the legacy
    // `:wat::std::service::Console::*` (typealias) and
    // `:wat::std::service::Console/*` (verb) prefixes. The Console
    // grouping noun retires per § K; verbs and typealiases live at
    // `:wat::console::*`. Plus Pattern A channel canonicalization:
    // Tx/Rx rename to ReqTx/ReqRx mirroring Telemetry's reference.
    for func in sym.functions.values() {
        validate_legacy_console_path(&func.body, &mut errors);
    }
    for form in forms {
        validate_legacy_console_path(form, &mut errors);
    }

    // Arc 109 slice K.lru — refuse the legacy
    // `:wat::lru::CacheService::*` (typealias) and
    // `:wat::lru::CacheService/*` (verb) prefixes. CacheService
    // grouping noun retires per § K; verbs + typealiases live at
    // `:wat::lru::*`. Plus Pattern B canonicalization: ReqPair →
    // ReqChannel (gaze 2026-05-01); ReplyRx + ReplyChannel
    // typealiases minted to complete Pattern B.
    for func in sym.functions.values() {
        validate_legacy_lru_cache_service_path(&func.body, &mut errors);
    }
    for form in forms {
        validate_legacy_lru_cache_service_path(form, &mut errors);
    }

    // Arc 109 slice K.kernel-channel — refuse the legacy
    // `:wat::kernel::Queue*` family. Kernel channel-primitive
    // vocabulary moved from Queue* (which leaked crossbeam's
    // data-structure name) to the canonical Channel / Sender /
    // Receiver family.
    for func in sym.functions.values() {
        validate_legacy_kernel_queue_path(&func.body, &mut errors);
    }
    for form in forms {
        validate_legacy_kernel_queue_path(form, &mut errors);
    }

    // Check each user define's body against its declared return type.
    for (path, func) in &sym.functions {
        if let Some(scheme) = env.get(path) {
            check_function_body(path, func, scheme, &env, &mut fresh, &mut errors);
        }
    }

    // Check every call in the program body (the post-define residue).
    for form in forms {
        check_form(form, &env, &mut fresh, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(CheckErrors(errors))
    }
}

/// Arc 110 — parent-context tag for the `validate_comm_positions`
/// walk. Every sub-expression descends with one of these; comm calls
/// are legal only under the three non-Forbidden tags.
#[derive(Clone, Copy)]
enum CommCtx {
    /// Top-level form, function/lambda body, struct field, call
    /// argument, let RHS — anywhere a `:None` would silently slip
    /// past. The default for every recursive descent.
    Forbidden,
    /// Discriminant slot of `:wat::core::match`. The match form
    /// requires every arm of the comm result's three states
    /// (Ok(Some _), Ok(:None), Err _) to be handled per arc 111.
    MatchScrutinee,
    /// Value-position slot of `:wat::core::result::expect`. Since
    /// arc 111, send/recv return Result<Option<T>, ThreadDiedError>;
    /// result::expect unwraps the Result (panic on Err), leaving
    /// the inner Option<T> for the caller to handle.
    ResultExpectValue,
    /// Value-position slot of `:wat::core::option::expect`. Reserved
    /// for callers that have already unwrapped a Result somewhere
    /// upstream and want to panic on the inner :None. (Direct
    /// kernel::send/recv calls won't fit here under arc 111 — their
    /// type is Result<Option<_>, _>, not Option<_>; they fit
    /// `ResultExpectValue` instead.)
    OptionExpectValue,
}

/// Arc 110 — refuse to compile programs that ignore a kernel-comm
/// terminal `:None`. Walks the AST tracking the parent's syntactic
/// context; whenever a `:wat::kernel::send` or `:wat::kernel::recv`
/// call appears outside the two permitted slots, push an error.
///
/// The rule is local — comm calls live where they're consumed; helper
/// functions that wrap a recv must do the consumption (match-or-expect)
/// internally and return a non-Option value.
fn validate_comm_positions(
    node: &WatAST,
    ctx: CommCtx,
    errors: &mut Vec<CheckError>,
) {
    let WatAST::List(items, _) = node else { return; };
    let (head_str, head_span) = match items.first() {
        Some(WatAST::Keyword(k, hs)) => (k.as_str(), hs),
        _ => {
            for child in items {
                validate_comm_positions(child, CommCtx::Forbidden, errors);
            }
            return;
        }
    };

    // (1) THIS node is a kernel-comm call.
    if matches!(
        head_str,
        ":wat::kernel::send"
            | ":wat::kernel::recv"
            | ":wat::kernel::process-send"
            | ":wat::kernel::process-recv"
    ) {
        let permitted = matches!(
            ctx,
            CommCtx::MatchScrutinee
                | CommCtx::ResultExpectValue
                | CommCtx::OptionExpectValue,
        );
        if !permitted {
            errors.push(CheckError::CommCallOutOfPosition {
                callee: head_str.into(),
                span: head_span.clone(),
            });
        }
        // Comm-call arguments are ordinary expressions; nested comm
        // calls inside them are themselves Forbidden.
        for child in &items[1..] {
            validate_comm_positions(child, CommCtx::Forbidden, errors);
        }
        return;
    }

    // (2) `:wat::core::match` — items[1] is the scrutinee (permitted slot).
    //     Layout: (match scrut -> :T arm1 arm2 ...)
    if head_str == ":wat::core::match" && items.len() >= 4 {
        validate_comm_positions(&items[1], CommCtx::MatchScrutinee, errors);
        for child in &items[2..] {
            validate_comm_positions(child, CommCtx::Forbidden, errors);
        }
        return;
    }

    // (3) Result-side `expect` form — items[3] is the value-position.
    //     Layout: (Result/expect -> :T <res> <msg>). Arc 109 slice 1j
    //     renamed `:wat::core::result::expect` to
    //     `:wat::core::Result/expect`; both heads still dispatch (the
    //     retired form fires a Pattern 2 poison) so both are
    //     recognized here for the duration of the migration.
    //     Arc 111: send/recv now return Result<Option<_>, _>; this is
    //     their natural panic-on-Err home.
    if (head_str == ":wat::core::Result/expect"
        || head_str == ":wat::core::result::expect")
        && items.len() >= 5
    {
        for (i, child) in items.iter().enumerate() {
            let child_ctx = if i == 3 {
                CommCtx::ResultExpectValue
            } else {
                CommCtx::Forbidden
            };
            validate_comm_positions(child, child_ctx, errors);
        }
        return;
    }

    // (4) Option-side `expect` form — items[3] is the value-position.
    //     Layout: (Option/expect -> :T <opt> <msg>). Arc 109 slice 1j
    //     renamed `:wat::core::option::expect` to
    //     `:wat::core::Option/expect`; both heads recognized for the
    //     migration window (same poison-and-dispatch shape as the
    //     Result form above). Pre-arc-111 home for kernel comm; kept
    //     for callers who have ALREADY unwrapped the outer Result
    //     (their own match/expect) and want to panic on the inner
    //     :None.
    if (head_str == ":wat::core::Option/expect"
        || head_str == ":wat::core::option::expect")
        && items.len() >= 5
    {
        for (i, child) in items.iter().enumerate() {
            let child_ctx = if i == 3 {
                CommCtx::OptionExpectValue
            } else {
                CommCtx::Forbidden
            };
            validate_comm_positions(child, child_ctx, errors);
        }
        return;
    }

    // (5) Default — every child descends as Forbidden.
    for child in items {
        validate_comm_positions(child, CommCtx::Forbidden, errors);
    }
}

// ─── Arc 140 — sandbox-scope leak prevention ──────────────────────────
//
// Deftest bodies (and any other `(:wat::kernel::run-sandboxed-ast ...)`
// callers) run in a sub-program whose scope contains ONLY the
// forms-block argument (prelude + auto-generated `:user::main`) plus
// stdlib. Outer-file user defines are NOT captured — sandbox isolation
// is intentional (per `wat/test.wat`'s deftest macro and
// `wat/std/sandbox.wat`'s `run-sandboxed-ast` shape).
//
// The failure mode that has burned the project repeatedly: user puts a
// helper at the top level of a test file, references it from a deftest
// body. Outer freeze type-checks the body with the outer scope visible;
// it passes silently. Sub-program freeze runs with the restricted
// scope; resolve / runtime fires `unknown function: :foo` — generic,
// no scoping explanation.
//
// Arc 140 catches this at outer freeze. For each sandbox-primitive
// call site, build the inner-scope name set (defines in the forms-
// block). Walk inner-form bodies. For each call head that's NOT
// reserved-prefix AND NOT in the inner scope BUT IS in the outer
// `SymbolTable` — fire `CheckError::SandboxScopeLeak` with both spans
// (the offending invocation + the outer-scope define).
fn validate_sandbox_scope_leak(
    node: &WatAST,
    sym: &SymbolTable,
    errors: &mut Vec<CheckError>,
) {
    let WatAST::List(items, _) = node else { return; };

    // Recurse first into all children — handles nested sandbox calls
    // (e.g., a top-level form holding a deftest holding another
    // sandbox primitive).
    for child in items {
        validate_sandbox_scope_leak(child, sym, errors);
    }

    // Check if THIS node is a sandbox-primitive call.
    let head_str = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return,
    };
    let is_sandbox_call = matches!(
        head_str,
        ":wat::kernel::run-sandboxed-ast"
            | ":wat::kernel::run-sandboxed-hermetic-ast"
            | ":wat::kernel::fork-program-ast"
            | ":wat::kernel::spawn-program-ast"
    );
    if !is_sandbox_call || items.len() < 2 {
        return;
    }

    // arg[0] should be a `(:wat::core::forms <inner-form>...)` block.
    let WatAST::List(forms_items, _) = &items[1] else { return; };
    let forms_head_ok = matches!(
        forms_items.first(),
        Some(WatAST::Keyword(k, _)) if k == ":wat::core::forms"
    );
    if !forms_head_ok {
        return;
    }
    let inner_forms = &forms_items[1..];

    // Collect names defined at the top level of the inner forms.
    // Define forms are top-level only — no nested-define recursion
    // needed. Strip `<T,...>` from each name so generic and concrete
    // call sites both resolve against the canonical name.
    let mut inner_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for form in inner_forms {
        if let WatAST::List(items_d, _) = form {
            if let Some(WatAST::Keyword(define_head, _)) = items_d.first() {
                if define_head == ":wat::core::define" && items_d.len() == 3 {
                    if let WatAST::List(sig, _) = &items_d[1] {
                        if let Some(WatAST::Keyword(name, _)) = sig.first() {
                            let canonical = match name.find('<') {
                                Some(i) => name[..i].to_string(),
                                None => name.clone(),
                            };
                            inner_names.insert(canonical);
                        }
                    }
                }
            }
        }
    }

    // Walk each inner form, checking call heads.
    for form in inner_forms {
        check_calls_for_sandbox_leak(form, &inner_names, sym, errors);
    }
}

/// Recursive companion to `validate_sandbox_scope_leak`. Walks an
/// inner-form AST looking for call-position keyword heads that
/// satisfy:
///
/// - NOT a reserved-prefix path (`:wat::*` / `:rust::*`)
/// - NOT in the inner-scope name set (passed-in by the caller)
/// - IS registered in the outer `SymbolTable`
///
/// When all three hold, fire `CheckError::SandboxScopeLeak`. Stops
/// descending into nested sandbox primitives — those have their own
/// inner-scope analyzed by the outer caller's recursion in
/// `validate_sandbox_scope_leak`.
fn check_calls_for_sandbox_leak(
    node: &WatAST,
    inner_names: &std::collections::HashSet<String>,
    sym: &SymbolTable,
    errors: &mut Vec<CheckError>,
) {
    let WatAST::List(items, _) = node else { return; };

    if let Some(WatAST::Keyword(head, head_span)) = items.first() {
        let head_str = head.as_str();
        // Strip `<T,...>` for lookup (the symbol table key is the
        // canonical name without type-parameter annotation; arc 139
        // territory).
        let canonical = match head_str.find('<') {
            Some(i) => &head_str[..i],
            None => head_str,
        };
        let reserved = canonical.starts_with(":wat::") || canonical.starts_with(":rust::");
        if !reserved && !inner_names.contains(canonical) {
            if let Some(outer_func) = sym.get(canonical) {
                errors.push(CheckError::SandboxScopeLeak {
                    offending_name: head_str.to_string(),
                    call_span: head_span.clone(),
                    outer_define_span: outer_func.body.span().clone(),
                });
            }
        }

        // Stop at nested sandbox boundaries — the outer caller's
        // recursion handles those.
        let is_nested_sandbox = matches!(
            head_str,
            ":wat::kernel::run-sandboxed-ast"
                | ":wat::kernel::run-sandboxed-hermetic-ast"
                | ":wat::kernel::fork-program-ast"
                | ":wat::kernel::spawn-program-ast"
        );
        if is_nested_sandbox {
            return;
        }
    }

    // Recurse into all children for nested calls.
    for child in items {
        check_calls_for_sandbox_leak(child, inner_names, sym, errors);
    }
}

// ─── Arc 117 — scope-deadlock prevention ──────────────────────────────
//
// At every `:wat::kernel::spawn-thread` call whose body lambda
// closure-captures a Receiver from a sibling `:wat::kernel::QueuePair`
// AND `recv`s on that Receiver inside the body — the pair's Sender
// clone outlives the worker; any `Thread/join-result` deadlocks. The
// substrate refuses to compile the structural shape; the diagnostic
// names the canonical fix (`SERVICE-PROGRAMS.md` § "The lockstep" —
// inner let* owns the queue + Sender clones; returns the Thread to
// outer; outer then joins).
//
// Walk every let* in the program. Within each binding-block, track
// sibling bindings (name → RHS-AST). When a binding's RHS is a
// spawn-thread call with a lambda body, analyze the lambda for the
// closure-capture-of-sibling-pair shape; if found, emit
// `ScopeDeadlock`.
//
// Limitations (false negatives, never false positives):
// - Function-keyword bodies are skipped (can't trace closure across
//   the function-table boundary).
// - The captured-name's RHS must be `(:wat::core::second pair)` and
//   pair's RHS must be `(:wat::kernel::make-bounded-queue ...)` /
//   `make-unbounded-queue` — no transitive helpers.
// - `select` is not yet pattern-matched (only `recv` / `try-recv`).

/// Arc 117's pre-inference structural scope-deadlock walker. Retired by
/// arc 133 slice 1: the rule is now enforced inside `infer_let_star`
/// after binding inference, covering both typed-name and tuple-destructure
/// shapes. Kept as a reference for the arc 117 approach; callers in
/// `check_program` were removed. Dead code intentional — documents the
/// retired path.
#[allow(dead_code)]
fn validate_scope_deadlock(node: &WatAST, types: &TypeEnv, errors: &mut Vec<CheckError>) {
    walk_for_deadlock(node, types, errors);
}

/// Arc 109 slice 1c — walk every WatAST node, parse keywords as
/// type expressions WITHOUT canonicalization, and structurally
/// detect bare primitive `Path` nodes (`:i64`, `:f64`, `:bool`,
/// `:String`, `:u8`). Emit one [`CheckError::BareLegacyPrimitive`]
/// per occurrence so sonnet sweeps can read off `wat --check`'s
/// diagnostic stream and rename per site.
///
/// Wat is a lisp; types parse to TypeExpr trees. The walker
/// consumes the parsed structure — parametric inners surface as
/// recursive `Path` children, tuple members as `Tuple` elements,
/// fn args/return as `Fn` field children. No textual scanning.
///
/// Pattern 3 from `docs/SUBSTRATE-AS-TEACHER.md` — dedicated
/// CheckError variant + walker, no `collect_hints` involvement.
/// Mirrors arc 110 (`CommCallOutOfPosition`), arc 115
/// (`InnerColonInCompoundArg`), arc 117 (`ScopeDeadlock`).
fn validate_bare_legacy_primitives(node: &WatAST, errors: &mut Vec<CheckError>) {
    walk_for_bare_primitives(node, errors);
}

fn walk_for_bare_primitives(node: &WatAST, errors: &mut Vec<CheckError>) {
    match node {
        WatAST::Keyword(s, span) => {
            // Try parsing as a type expression. Most keywords aren't
            // types (callee paths, value keywords like `:None`); they
            // parse to a plain Path that doesn't match any bare
            // primitive — the walk falls through cleanly.
            //
            // `parse_type_expr_audit` is parse_type_inner with the
            // bare→bare canonicalization suppressed: bare `:i64`
            // produces `Path(":i64")`, FQDN `:wat::core::i64`
            // produces `Path(":wat::core::i64")`. Source spelling
            // preserved; the structural walk distinguishes them.
            if let Some(ty) = crate::types::parse_type_expr_audit(s) {
                walk_type_for_bare(&ty, span, errors);
            }
        }
        WatAST::List(items, _) => {
            for item in items {
                walk_for_bare_primitives(item, errors);
            }
        }
        _ => {}
    }
}

/// The five primitive names retired by arc 109 slice 1c, paired
/// with the canonical FQDN form they replace.
const BARE_PRIMITIVES: &[(&str, &str)] = &[
    (":i64", ":wat::core::i64"),
    (":f64", ":wat::core::f64"),
    (":bool", ":wat::core::bool"),
    (":String", ":wat::core::String"),
    (":u8", ":wat::core::u8"),
];

/// Parametric container heads retired by arc 109 (slices 1e + 1f),
/// paired with the canonical FQDN form they replace. Heads are
/// stored without the leading colon to match
/// `TypeExpr::Parametric` head-string convention. Note that
/// `Vec` → `wat::core::Vector` is BOTH a path move AND a name
/// rename (the only entry where the FQDN tail differs from the
/// bare form); the others are pure-FQDN-moves.
const BARE_CONTAINER_HEADS: &[(&str, &str)] = &[
    ("Option", "wat::core::Option"),    // slice 1e
    ("Result", "wat::core::Result"),    // slice 1e
    ("HashMap", "wat::core::HashMap"),  // slice 1e
    ("HashSet", "wat::core::HashSet"),  // slice 1e
    ("Vec", "wat::core::Vector"),       // slice 1f — rename + move
];

/// Recursively walk a parsed [`TypeExpr`], emitting
/// [`CheckError::BareLegacyPrimitive`] for every `Path` node whose
/// path matches one of the retired bare primitive names. FQDN
/// forms (`:wat::core::i64`) are distinct `Path` strings and pass
/// through silently.
fn walk_type_for_bare(ty: &TypeExpr, span: &Span, errors: &mut Vec<CheckError>) {
    match ty {
        TypeExpr::Path(p) => {
            for (bare, fqdn) in BARE_PRIMITIVES {
                if p == bare {
                    errors.push(CheckError::BareLegacyPrimitive {
                        primitive: (*bare).to_string(),
                        fqdn: (*fqdn).to_string(),
                        span: span.clone(),
                    });
                    return;
                }
            }
        }
        TypeExpr::Parametric { head, args } => {
            // Arc 109 slice 1e — flag bare container heads. The
            // FQDN form ("wat::core::Option" etc.) parses to a
            // distinct head string and passes through silently.
            // Recurse into args regardless so inner-position
            // primitives (slice 1c) and units (slice 1d) still
            // surface.
            for (bare, fqdn) in BARE_CONTAINER_HEADS {
                if head == bare {
                    errors.push(CheckError::BareLegacyContainerHead {
                        head: (*bare).to_string(),
                        fqdn: (*fqdn).to_string(),
                        span: span.clone(),
                    });
                    break;
                }
            }
            for a in args {
                walk_type_for_bare(a, span, errors);
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                walk_type_for_bare(a, span, errors);
            }
            walk_type_for_bare(ret, span, errors);
        }
        TypeExpr::Tuple(elements) => {
            // Arc 109 slice 1d — empty Tuple is the bare unit type
            // annotation `:()`. The FQDN form `:wat::core::unit`
            // parses to a Path (typealias) and lands in the Path arm
            // above, not here. Non-empty tuples are bona-fide tuple
            // types and recurse normally.
            if elements.is_empty() {
                errors.push(CheckError::BareLegacyUnitType {
                    span: span.clone(),
                });
            } else {
                for e in elements {
                    walk_type_for_bare(e, span, errors);
                }
            }
        }
        TypeExpr::Var(_) => {}
    }
}

/// Arc 109 slice 9d — walk every WatAST node, detecting any
/// keyword whose path starts with the legacy
/// `:wat::std::stream::` prefix. Stream stdlib graduated to
/// `:wat::stream::*` per § G's three-tier substrate organization.
///
/// Pattern 3 (dedicated CheckError variant + walker; no
/// `collect_hints` involvement). Same shape as slices 1c/1d/1e but
/// at the keyword-prefix level rather than the parsed-TypeExpr
/// level — this is a pure namespace-prefix retirement, no shape
/// shift involved.
///
/// Catches all positions uniformly (callable head, type
/// annotation, value position) since every legacy use surfaces as
/// a `WatAST::Keyword` node carrying the prefix.
fn validate_legacy_stream_path(node: &WatAST, errors: &mut Vec<CheckError>) {
    walk_for_legacy_stream(node, errors);
}

const LEGACY_STREAM_PREFIX: &str = ":wat::std::stream::";
const CANONICAL_STREAM_PREFIX: &str = ":wat::stream::";

fn walk_for_legacy_stream(node: &WatAST, errors: &mut Vec<CheckError>) {
    match node {
        WatAST::Keyword(s, span) => {
            if s.starts_with(LEGACY_STREAM_PREFIX) {
                let new = format!(
                    "{}{}",
                    CANONICAL_STREAM_PREFIX,
                    &s[LEGACY_STREAM_PREFIX.len()..]
                );
                errors.push(CheckError::BareLegacyStreamPath {
                    old: s.clone(),
                    new,
                    span: span.clone(),
                });
            }
        }
        WatAST::List(items, _) => {
            for item in items {
                walk_for_legacy_stream(item, errors);
            }
        }
        _ => {}
    }
}

/// Arc 109 slice K.telemetry — same shape as `validate_legacy_stream_path`
/// but for the two Service grouping-noun prefixes. Catches both
/// `:wat::telemetry::Service::X` (typealias path) and
/// `:wat::telemetry::Service/X` (verb path); canonical replacement
/// strips the `Service::` or `Service/` segment so the path becomes
/// `:wat::telemetry::X`.
fn validate_legacy_telemetry_service_path(node: &WatAST, errors: &mut Vec<CheckError>) {
    walk_for_legacy_telemetry_service(node, errors);
}

const LEGACY_TELEMETRY_SERVICE_TYPEALIAS_PREFIX: &str = ":wat::telemetry::Service::";
const LEGACY_TELEMETRY_SERVICE_VERB_PREFIX: &str = ":wat::telemetry::Service/";
const CANONICAL_TELEMETRY_PREFIX: &str = ":wat::telemetry::";

fn walk_for_legacy_telemetry_service(node: &WatAST, errors: &mut Vec<CheckError>) {
    match node {
        WatAST::Keyword(s, span) => {
            let stripped = s
                .strip_prefix(LEGACY_TELEMETRY_SERVICE_TYPEALIAS_PREFIX)
                .or_else(|| s.strip_prefix(LEGACY_TELEMETRY_SERVICE_VERB_PREFIX));
            if let Some(tail) = stripped {
                let new = format!("{}{}", CANONICAL_TELEMETRY_PREFIX, tail);
                errors.push(CheckError::BareLegacyTelemetryServicePath {
                    old: s.clone(),
                    new,
                    span: span.clone(),
                });
            }
        }
        WatAST::List(items, _) => {
            for item in items {
                walk_for_legacy_telemetry_service(item, errors);
            }
        }
        _ => {}
    }
}

/// Arc 109 slice K.console — same shape as
/// `validate_legacy_telemetry_service_path` plus the Pattern A
/// channel canonicalization for Tx/Rx → ReqTx/ReqRx. Catches both
/// `:wat::std::service::Console::X` (typealias) and
/// `:wat::std::service::Console/X` (verb); canonical replacement
/// strips the `:wat::std::service::Console::` or
/// `:wat::std::service::Console/` segment and substitutes the
/// canonical leaf for `Tx` / `Rx`.
fn validate_legacy_console_path(node: &WatAST, errors: &mut Vec<CheckError>) {
    walk_for_legacy_console(node, errors);
}

const LEGACY_CONSOLE_TYPEALIAS_PREFIX: &str = ":wat::std::service::Console::";
const LEGACY_CONSOLE_VERB_PREFIX: &str = ":wat::std::service::Console/";
const CANONICAL_CONSOLE_PREFIX: &str = ":wat::console::";

fn canonical_console_leaf(tail: &str) -> &str {
    match tail {
        "Tx" => "ReqTx",
        "Rx" => "ReqRx",
        other => other,
    }
}

fn walk_for_legacy_console(node: &WatAST, errors: &mut Vec<CheckError>) {
    match node {
        WatAST::Keyword(s, span) => {
            if let Some(tail) = s.strip_prefix(LEGACY_CONSOLE_TYPEALIAS_PREFIX) {
                let new = format!(
                    "{}{}",
                    CANONICAL_CONSOLE_PREFIX,
                    canonical_console_leaf(tail)
                );
                errors.push(CheckError::BareLegacyConsolePath {
                    old: s.clone(),
                    new,
                    span: span.clone(),
                });
            } else if let Some(tail) = s.strip_prefix(LEGACY_CONSOLE_VERB_PREFIX) {
                let new = format!("{}{}", CANONICAL_CONSOLE_PREFIX, tail);
                errors.push(CheckError::BareLegacyConsolePath {
                    old: s.clone(),
                    new,
                    span: span.clone(),
                });
            }
        }
        WatAST::List(items, _) => {
            for item in items {
                walk_for_legacy_console(item, errors);
            }
        }
        _ => {}
    }
}

/// Arc 109 slice K.lru — same shape as
/// `validate_legacy_console_path` plus the Pattern B
/// canonicalization for ReqPair → ReqChannel. Catches both
/// `:wat::lru::CacheService::X` (typealias) and
/// `:wat::lru::CacheService/X` (verb); canonical replacement
/// strips the segment and substitutes the canonical leaf for
/// `ReqPair`.
fn validate_legacy_lru_cache_service_path(node: &WatAST, errors: &mut Vec<CheckError>) {
    walk_for_legacy_lru_cache_service(node, errors);
}

const LEGACY_LRU_CACHE_SERVICE_TYPEALIAS_PREFIX: &str = ":wat::lru::CacheService::";
const LEGACY_LRU_CACHE_SERVICE_VERB_PREFIX: &str = ":wat::lru::CacheService/";
const CANONICAL_LRU_PREFIX: &str = ":wat::lru::";

fn canonical_lru_leaf(tail: &str) -> &str {
    match tail {
        "ReqPair" => "ReqChannel",
        other => other,
    }
}

fn walk_for_legacy_lru_cache_service(node: &WatAST, errors: &mut Vec<CheckError>) {
    match node {
        WatAST::Keyword(s, span) => {
            if let Some(tail) = s.strip_prefix(LEGACY_LRU_CACHE_SERVICE_TYPEALIAS_PREFIX) {
                let new = format!(
                    "{}{}",
                    CANONICAL_LRU_PREFIX,
                    canonical_lru_leaf(tail)
                );
                errors.push(CheckError::BareLegacyLruCacheServicePath {
                    old: s.clone(),
                    new,
                    span: span.clone(),
                });
            } else if let Some(tail) = s.strip_prefix(LEGACY_LRU_CACHE_SERVICE_VERB_PREFIX) {
                let new = format!("{}{}", CANONICAL_LRU_PREFIX, tail);
                errors.push(CheckError::BareLegacyLruCacheServicePath {
                    old: s.clone(),
                    new,
                    span: span.clone(),
                });
            }
        }
        WatAST::List(items, _) => {
            for item in items {
                walk_for_legacy_lru_cache_service(item, errors);
            }
        }
        _ => {}
    }
}

/// Arc 109 slice K.kernel-channel — walks every WatAST keyword
/// looking for the five retired kernel `Queue*` family names.
/// Each retired name has a single canonical replacement; the
/// substitution table is hard-coded.
fn validate_legacy_kernel_queue_path(node: &WatAST, errors: &mut Vec<CheckError>) {
    walk_for_legacy_kernel_queue(node, errors);
}

/// Retired kernel `Queue*` family — paired with their canonical
/// replacements. Type names use `<` matching since they appear
/// as parametric heads (e.g. `:wat::kernel::QueueSender<i64>`);
/// verb names match the bare keyword.
const LEGACY_KERNEL_QUEUE_NAMES: &[(&str, &str)] = &[
    (":wat::kernel::QueueSender", ":wat::kernel::Sender"),
    (":wat::kernel::QueueReceiver", ":wat::kernel::Receiver"),
    (":wat::kernel::QueuePair", ":wat::kernel::Channel"),
    (
        ":wat::kernel::make-bounded-queue",
        ":wat::kernel::make-bounded-channel",
    ),
    (
        ":wat::kernel::make-unbounded-queue",
        ":wat::kernel::make-unbounded-channel",
    ),
];

fn walk_for_legacy_kernel_queue(node: &WatAST, errors: &mut Vec<CheckError>) {
    match node {
        WatAST::Keyword(s, span) => {
            for (legacy, canonical) in LEGACY_KERNEL_QUEUE_NAMES {
                // Match the legacy name as a prefix; type aliases
                // can carry `<...>` parametrics tail so we strip-
                // and-rebuild rather than equal-match.
                if let Some(tail) = s.strip_prefix(legacy) {
                    // Boundary check: the legacy name must be
                    // followed by `<`, end-of-string, or a non-
                    // identifier character. This prevents
                    // `:wat::kernel::QueueSenderXYZ` from matching
                    // `QueueSender` (theoretical; not present
                    // today but a sane guard).
                    let boundary_ok = tail.is_empty()
                        || tail.starts_with('<')
                        || !tail
                            .chars()
                            .next()
                            .map(|c| c.is_alphanumeric() || c == '_' || c == '-')
                            .unwrap_or(false);
                    if boundary_ok {
                        let new = format!("{}{}", canonical, tail);
                        errors.push(CheckError::BareLegacyKernelQueuePath {
                            old: s.clone(),
                            new,
                            span: span.clone(),
                        });
                        break;
                    }
                }
            }
        }
        WatAST::List(items, _) => {
            for item in items {
                walk_for_legacy_kernel_queue(item, errors);
            }
        }
        _ => {}
    }
}

/// Arc 117's pre-inference recursive walker. Retired by arc 133 slice 1.
/// Kept as a reference. Dead code intentional.
#[allow(dead_code)]
fn walk_for_deadlock(
    node: &WatAST,
    types: &TypeEnv,
    errors: &mut Vec<CheckError>,
) {
    let WatAST::List(items, _) = node else { return; };
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => {
            for child in items {
                walk_for_deadlock(child, types, errors);
            }
            return;
        }
    };

    // Arc 128 — sandbox-boundary guard. The first argument to a
    // sandbox-program primitive is a `(:wat::core::forms ...)` block
    // representing an INNER program; the inner program has its own
    // freeze cycle when the primitive fires at runtime. Outer freeze
    // must not redundantly walk the inner forms — doing so conflates
    // outer/inner scope and emits errors that belong to the inner
    // program. Skip arg 0 (the forms block); recurse into trailing
    // args (type-list, config arg).
    if matches!(
        head,
        ":wat::kernel::run-sandboxed-ast"
            | ":wat::kernel::run-sandboxed-hermetic-ast"
            | ":wat::kernel::fork-program-ast"
            | ":wat::kernel::spawn-program-ast"
    ) {
        for child in items.iter().skip(2) {
            walk_for_deadlock(child, types, errors);
        }
        return;
    }

    if head == ":wat::core::let*" && items.len() >= 3 {
        let bindings = match &items[1] {
            WatAST::List(xs, _) => xs.clone(),
            _ => return,
        };
        let body_forms: Vec<WatAST> = items[2..].to_vec();
        // Recurse into each binding's RHS first (catches nested let*'s).
        for binding in &bindings {
            if let WatAST::List(parts, _) = binding {
                if parts.len() == 2 {
                    walk_for_deadlock(&parts[1], types, errors);
                }
            }
        }
        for body_form in &body_forms {
            walk_for_deadlock(body_form, types, errors);
        }
        // Run the structural rule at THIS let*'s scope.
        check_let_star_for_scope_deadlock(&bindings, &body_forms, types, errors);
        return;
    }

    // For all other forms, descend.
    for child in items {
        walk_for_deadlock(child, types, errors);
    }
}

/// Arc 117's pre-inference per-let* structural scope-deadlock rule.
/// Retired by arc 133 slice 1 — replaced by
/// `check_let_star_for_scope_deadlock_inferred`. Kept as reference.
/// Dead code intentional.
#[allow(dead_code)]
fn check_let_star_for_scope_deadlock(
    bindings: &[WatAST],
    body_forms: &[WatAST],
    types: &TypeEnv,
    errors: &mut Vec<CheckError>,
) {
    // Collect Thread bindings AND Sender-bearing bindings in this
    // let*'s binding-block, using resolved TypeExpr structure.
    let mut thread_bindings: Vec<(String, Span)> = Vec::new();
    let mut sender_bearing_bindings: Vec<(String, &'static str)> = Vec::new();
    for binding in bindings {
        let (name, type_ann_str, span) = match parse_binding_for_typed_check(binding) {
            Some(t) => t,
            None => continue,
        };
        let parsed = match crate::types::parse_type_expr(&type_ann_str) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if type_is_thread_kind(&parsed, types) {
            thread_bindings.push((name, span));
            continue;
        }
        if let Some(kind) = type_contains_sender_kind(&parsed, types) {
            sender_bearing_bindings.push((name, kind));
        }
    }
    if thread_bindings.is_empty() || sender_bearing_bindings.is_empty() {
        return;
    }
    // For each Thread binding, check whether `Thread/join-result thr`
    // (or `Process/join-result`) appears in body_forms or in any
    // binding's RHS in this let*'s extent.
    for (thr_name, thr_span) in &thread_bindings {
        let join_present = body_forms
            .iter()
            .any(|f| contains_join_on_thread(f, thr_name))
            || bindings
                .iter()
                .any(|b| contains_join_on_thread(b, thr_name));
        if !join_present {
            continue;
        }
        for (sender_name, kind) in &sender_bearing_bindings {
            errors.push(CheckError::ScopeDeadlock {
                thread_binding: thr_name.clone(),
                offending_binding: sender_name.clone(),
                offending_kind: kind,
                span: thr_span.clone(),
            });
        }
    }
}

/// Parse a let* binding `((name :type-annotation) rhs)` → (name,
/// type_annotation_keyword, span). Returns None on shapes that don't
/// fit (untyped bindings, tuple-destructure patterns).
/// Arc 133 — called only by the retired `check_let_star_for_scope_deadlock`.
/// Kept as reference; dead code intentional.
#[allow(dead_code)]
fn parse_binding_for_typed_check(binding: &WatAST) -> Option<(String, String, Span)> {
    let WatAST::List(items, span) = binding else { return None; };
    if items.len() != 2 {
        return None;
    }
    let pattern = &items[0];
    let WatAST::List(parts, _) = pattern else { return None; };
    if parts.len() < 2 {
        return None;
    }
    let name = match &parts[0] {
        WatAST::Symbol(id, _) => id.name.clone(),
        _ => return None,
    };
    let type_ann_str = match &parts[1] {
        WatAST::Keyword(k, _) => k.clone(),
        _ => return None,
    };
    Some((name, type_ann_str, span.clone()))
}

/// Does this type, after alias resolution, denote a Thread/Process/
/// Program (the values one calls `Thread/join-result` on)?
fn type_is_thread_kind(ty: &TypeExpr, types: &TypeEnv) -> bool {
    let canonical = crate::types::expand_alias(ty, types);
    match canonical {
        TypeExpr::Parametric { head, .. } => matches!(
            head.as_str(),
            "wat::kernel::Thread" | "wat::kernel::Process" | "wat::kernel::Program"
        ),
        _ => false,
    }
}

/// Does this type, after alias resolution and recursive descent into
/// arguments and tuple elements, contain a `:wat::kernel::QueueSender<T>`
/// ANYWHERE in its structure? Returns "QueueSender" on hit.
///
/// Why QueueSender (and not also bare `Sender`, `HandlePool`):
///   - `:wat::kernel::QueuePair<T>` is a typealias to
///     `:(QueueSender<T>, QueueReceiver<T>)`. `expand_alias` unwraps
///     it; after resolution, only QueueSender shows up. Detecting
///     QueueSender catches both the explicit `QueuePair` case and
///     any direct `QueueSender` binding.
///   - `:rust::crossbeam_channel::Sender<T>` (the raw substrate
///     primitive) is NOT flagged in isolation — it appears in many
///     non-deadlocking shapes (e.g., a Sender alongside a Thread
///     for unrelated state). Caller-allocated wat-level pairs always
///     surface as `QueueSender<T>` after alias resolution, which IS
///     the deadlock anchor.
///   - `:wat::kernel::HandlePool<T>` IS flagged when T contains
///     a Sender — arc 131 lifted the exclusion. The previous
///     narrowing avoided false-positives on Console's tests, but
///     the structural pattern (pool sibling to Thread with
///     join-result on the destructured Thread) IS deadlock-prone
///     by construction. Console's tests rely on runtime
///     handle-drop ordering; arc 131 makes the discipline
///     structural rather than voluntary. Returns "HandlePool"
///     on hit so the diagnostic names the offending kind.
fn type_contains_sender_kind(ty: &TypeExpr, types: &TypeEnv) -> Option<&'static str> {
    // Match wat-level Sender-anchor heads at the SURFACE first — `expand_alias`
    // would unwrap `Channel` → `(Sender, Receiver)` and then
    // `Sender` → `rust::crossbeam_channel::Sender`, which is too generic
    // to flag (Console.wat etc. would false-positive).
    if let TypeExpr::Parametric { head, args } = ty {
        if matches!(
            head.as_str(),
            "wat::kernel::Channel"
                | "wat::kernel::Sender"
                // Arc 133 — inference-time path reduces alias `wat::kernel::Sender`
                // to its underlying `rust::crossbeam_channel::Sender` via
                // `reduce` during unification. The inferred types in `extended`
                // already carry the expanded form; the surface check must
                // recognise both spellings to avoid false-negatives in
                // `check_let_star_for_scope_deadlock_inferred`.
                | "rust::crossbeam_channel::Sender"
        ) {
            return Some("Sender");
        }
        // Arc 131 — HandlePool is Sender-bearing IFF its parametric T
        // (after alias resolution) contains a Sender structurally.
        // The HandlePool entries are clones of Sender-carrying handles;
        // clients pop one, drop or use it, but the pool's internal
        // storage keeps Sender clones alive until each handle is popped
        // AND dropped. A sibling pool alongside Thread/join-result on
        // the service driver is the canonical service-test mistake
        // (the spawn-tuple-destructure pattern). Hypothetical
        // HandlePool<i64> / HandlePool<unit> shapes — no embedded
        // Sender — pass through silently.
        if head.as_str() == "wat::kernel::HandlePool" {
            for arg in args {
                if type_contains_sender_kind(arg, types).is_some() {
                    return Some("HandlePool");
                }
            }
            return None;
        }
        // Not a direct match; try peeling an alias ONCE in case `head` is a
        // user typealias that points at QueuePair/QueueSender.
        let peeled = crate::types::expand_alias(ty, types);
        if let TypeExpr::Parametric { head: h2, .. } = &peeled {
            if h2 != head {
                return type_contains_sender_kind(&peeled, types);
            }
        } else if !matches!(peeled, TypeExpr::Parametric { .. }) {
            return type_contains_sender_kind(&peeled, types);
        }
        // Recurse into args (handles e.g. `Vec<QueueSender<T>>`).
        for arg in args {
            if let Some(k) = type_contains_sender_kind(arg, types) {
                return Some(k);
            }
        }
        return None;
    }
    if let TypeExpr::Tuple(elements) = ty {
        for e in elements {
            if let Some(k) = type_contains_sender_kind(e, types) {
                return Some(k);
            }
        }
        return None;
    }
    if let TypeExpr::Path(_) = ty {
        // A bare path may be a user typealias.
        let peeled = crate::types::expand_alias(ty, types);
        if let TypeExpr::Path(p2) = &peeled {
            if let TypeExpr::Path(p1) = ty {
                if p1 == p2 {
                    return None;
                }
            }
        }
        return type_contains_sender_kind(&peeled, types);
    }
    None
}

/// Walk `node` looking for `(:wat::kernel::Thread/join-result thr)` or
/// `(:wat::kernel::Process/join-result thr)` (or arc 060's bare
/// `:wat::kernel::join-result` for completeness, even though it
/// retired in arc 114) where `thr` matches the given binding name.
/// True iff such a call is found anywhere in the AST subtree.
fn contains_join_on_thread(node: &WatAST, thread_binding: &str) -> bool {
    let WatAST::List(items, _) = node else { return false; };
    if let Some(WatAST::Keyword(k, _)) = items.first() {
        if matches!(
            k.as_str(),
            ":wat::kernel::Thread/join-result"
                | ":wat::kernel::Process/join-result"
                | ":wat::kernel::join-result"
                | ":wat::kernel::join"
        ) {
            if let Some(WatAST::Symbol(id, _)) = items.get(1) {
                if id.name == thread_binding {
                    return true;
                }
            }
        }
    }
    items
        .iter()
        .any(|child| contains_join_on_thread(child, thread_binding))
}

// ─── Arc 126 — channel-pair-deadlock prevention ───────────────────────
//
// Sibling rule to arc 117 ScopeDeadlock. Same trace machinery, applied
// at call sites instead of spawn-thread closure bodies. Refuses
// function-call shapes where both halves of one channel pair are
// passed to a single callee — the structural shape that produces the
// arc 119 "Pattern B Put-ack helper-verb cycle" deadlock.
//
// The trace walks the let* binding chain:
//   (call ... arg-name ...) → arg-name's RHS in scope → either
//     (:wat::core::first <inner>) / (:wat::core::second <inner>) →
//       recurse on <inner>
//     (:wat::kernel::make-bounded-channel ...) /
//     (:wat::kernel::make-unbounded-channel ...) → THIS is the
//       pair-anchor; return its (binding-name, span)
//     anything else → trace gives up (conservative; false-negative
//       is acceptable, false-positive is not — DESIGN § "False-negative
//       caveats")
//
// Type classification mirrors arc 117 — annotation parsed from the
// let* binding type slot, then `expand_alias` walks user typealiases
// (`PutAckTx` → `:wat::kernel::Sender<unit>` → `:rust::crossbeam_channel::Sender<unit>`)
// to the canonical Rust head. Sender side fires `type_is_sender_kind`;
// Receiver side fires `type_is_receiver_kind`; both sides traced;
// matched anchors fire ChannelPairDeadlock.
//
// Sandbox boundary (arc 128): `walk_for_pair_deadlock` inherits the
// same boundary guard as `walk_for_deadlock`. The first argument of
// `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` /
// `fork-program-ast` / `spawn-program-ast` is a forms-block
// representing an INNER program; the inner program freezes when the
// primitive fires at runtime. Outer freeze must NOT walk into it —
// doing so cascades inner-program errors into the outer file's
// freeze and breaks sibling deftests. The guard skips arg 0; trailing
// args (type-list, config) still get walked.
//
// Limitations (per DESIGN § "False-negative caveats"):
// - Multi-step rx/tx derivations skipped (binding through helper fns).
// - Tuple-typealias unpacks skipped (user struct hiding both halves).
// - Cross-function tracing skipped (pair allocated in caller A, both
//   halves passed through helper B that re-passes them).
// - Type-only arguments skipped (annotation-less bindings).

/// Walk every form in the program looking for the channel-pair-
/// deadlock shape. Called from `check_program` adjacent to
/// `validate_scope_deadlock`. Uses the TypeEnv to resolve aliases —
/// arguments typed `:PutAckTx` (an alias for
/// `:wat::kernel::Sender<wat::core::unit>`) are detected structurally,
/// not by surface-name matching.
fn validate_channel_pair_deadlock(
    node: &WatAST,
    types: &TypeEnv,
    errors: &mut Vec<CheckError>,
) {
    walk_for_pair_deadlock(node, types, &Vec::new(), errors);
}

/// Per-binding scope entry: `(name, type-annotation-keyword, rhs-ast)`.
/// Accumulated through nested let*'s so the call-site check can look
/// up an argument's type AND its RHS for binding-chain tracing.
type PairScopeEntry = (String, String, WatAST);

/// Recursive walker. At every `:wat::core::let*` form, extends the
/// binding scope with this let*'s entries; recurses into RHSes and
/// body forms with the extended scope. At every other List form
/// whose head is a Keyword (potentially a function call), runs
/// `check_call_for_pair_deadlock`.
fn walk_for_pair_deadlock(
    node: &WatAST,
    types: &TypeEnv,
    binding_scope: &[PairScopeEntry],
    errors: &mut Vec<CheckError>,
) {
    let WatAST::List(items, _) = node else { return; };
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => {
            for child in items {
                walk_for_pair_deadlock(child, types, binding_scope, errors);
            }
            return;
        }
    };

    // Arc 128 — sandbox-boundary guard. The first argument to a
    // sandbox-program primitive is a `(:wat::core::forms ...)` block
    // representing an INNER program; the inner program has its own
    // freeze cycle when the primitive fires at runtime. Outer freeze
    // must not redundantly walk the inner forms — doing so conflates
    // outer/inner scope and emits errors that belong to the inner
    // program. Skip arg 0 (the forms block); recurse into trailing
    // args (type-list, config arg). Mirrors the guard in
    // `walk_for_deadlock` so arc 126 inherits the boundary from
    // inception.
    if matches!(
        head,
        ":wat::kernel::run-sandboxed-ast"
            | ":wat::kernel::run-sandboxed-hermetic-ast"
            | ":wat::kernel::fork-program-ast"
            | ":wat::kernel::spawn-program-ast"
    ) {
        for child in items.iter().skip(2) {
            walk_for_pair_deadlock(child, types, binding_scope, errors);
        }
        return;
    }

    if head == ":wat::core::let*" && items.len() >= 3 {
        let bindings = match &items[1] {
            WatAST::List(xs, _) => xs.clone(),
            _ => return,
        };
        // Extend scope with this let*'s typed bindings (typed-name shape)
        // AND with synthetic pair-scope entries for tuple-destructure
        // bindings from make-bounded-channel / make-unbounded-channel
        // (arc 133). Both shapes end up in the same scope vec; the
        // trace logic is identical — it doesn't care whether the entry
        // came from a typed-name or a synthetic expansion.
        let mut extended: Vec<PairScopeEntry> = binding_scope.to_vec();
        let mut synthetic_counter = 0usize;
        for binding in &bindings {
            if let Some((name, type_ann, rhs)) = parse_binding_for_pair_check(binding) {
                extended.push((name, type_ann, rhs));
            } else {
                // Arc 133 — attempt to expand tuple-destructure bindings
                // from make-bounded-channel into synthetic pair-scope entries.
                extend_pair_scope_with_tuple_destructure(
                    binding,
                    &mut synthetic_counter,
                    &mut extended,
                );
            }
        }
        // Recurse into each binding's RHS with the *prefix* scope
        // available at that binding (let* is sequential; later
        // bindings see earlier ones, but a binding's own RHS is
        // evaluated before that name enters scope).
        let mut prefix: Vec<PairScopeEntry> = binding_scope.to_vec();
        let mut synthetic_counter_prefix = 0usize;
        for binding in &bindings {
            if let WatAST::List(parts, _) = binding {
                if parts.len() == 2 {
                    walk_for_pair_deadlock(&parts[1], types, &prefix, errors);
                    if let Some((name, type_ann, rhs)) =
                        parse_binding_for_pair_check(binding)
                    {
                        prefix.push((name, type_ann, rhs));
                    } else {
                        // Arc 133 — same expansion for the prefix scope.
                        extend_pair_scope_with_tuple_destructure(
                            binding,
                            &mut synthetic_counter_prefix,
                            &mut prefix,
                        );
                    }
                }
            }
        }
        // Body forms see the full extended scope.
        for body_form in &items[2..] {
            walk_for_pair_deadlock(body_form, types, &extended, errors);
        }
        return;
    }

    // Skip kernel comm primitives — those are governed by arc 117 /
    // arc 110, not arc 126. Their argument-shape is well-formed by
    // construction (one Sender or one Receiver per call).
    if matches!(
        head,
        ":wat::kernel::send"
            | ":wat::kernel::recv"
            | ":wat::kernel::try-recv"
            | ":wat::kernel::select"
            | ":wat::kernel::process-send"
            | ":wat::kernel::process-recv"
    ) {
        for child in &items[1..] {
            walk_for_pair_deadlock(child, types, binding_scope, errors);
        }
        return;
    }

    // Run the structural rule at THIS call site.
    check_call_for_pair_deadlock(node, binding_scope, types, errors);

    // Descend into children — nested calls / bindings still get checked.
    for child in items {
        walk_for_pair_deadlock(child, types, binding_scope, errors);
    }
}

/// At a function-call site, classify each Symbol argument by type.
/// For each Sender<T>-typed argument, trace its RHS chain to the
/// originating make-bounded-channel / make-unbounded-channel binding
/// (the "pair-anchor"). Same for Receiver<T>-typed arguments. Group
/// by anchor; if any anchor has BOTH a Sender argument AND a Receiver
/// argument, emit `ChannelPairDeadlock`.
fn check_call_for_pair_deadlock(
    call_form: &WatAST,
    binding_scope: &[PairScopeEntry],
    types: &TypeEnv,
    errors: &mut Vec<CheckError>,
) {
    let WatAST::List(items, span) = call_form else { return; };
    if items.len() < 2 {
        return;
    }
    let callee = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.clone(),
        _ => return,
    };

    // Per-anchor: (anchor-name, sender-arg-name, receiver-arg-name).
    // Built incrementally as we classify each argument.
    let mut sender_at_anchor: HashMap<String, String> = HashMap::new();
    let mut receiver_at_anchor: HashMap<String, String> = HashMap::new();

    for arg in &items[1..] {
        let WatAST::Symbol(id, _) = arg else { continue; };
        let arg_name = &id.name;
        // Look up arg in scope for its annotated type.
        let entry = match binding_scope.iter().find(|(n, _, _)| n == arg_name) {
            Some(e) => e,
            None => continue,
        };
        let parsed = match crate::types::parse_type_expr(&entry.1) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if type_is_sender_kind(&parsed, types) {
            if let Some((anchor, _)) = trace_to_pair_anchor(arg_name, binding_scope) {
                sender_at_anchor
                    .entry(anchor)
                    .or_insert_with(|| arg_name.clone());
            }
        } else if type_is_receiver_kind(&parsed, types) {
            if let Some((anchor, _)) = trace_to_pair_anchor(arg_name, binding_scope) {
                receiver_at_anchor
                    .entry(anchor)
                    .or_insert_with(|| arg_name.clone());
            }
        }
    }

    for (anchor, sender_arg) in &sender_at_anchor {
        if let Some(receiver_arg) = receiver_at_anchor.get(anchor) {
            errors.push(CheckError::ChannelPairDeadlock {
                callee: callee.clone(),
                sender_arg: sender_arg.clone(),
                receiver_arg: receiver_arg.clone(),
                pair_anchor: anchor.clone(),
                span: span.clone(),
            });
        }
    }
}

/// Trace a binding name through the let* binding chain to the
/// originating `(:wat::kernel::make-bounded-channel ...)` /
/// `make-unbounded-channel` call. Returns `(anchor-binding-name,
/// anchor-span)` on success; `None` when the chain doesn't bottom
/// out at a make-channel call (conservative — false-negative rather
/// than false-positive, per DESIGN).
///
/// The chain is:
///   name → RHS = (:wat::core::first <inner>) | (:wat::core::second <inner>)
///        → recurse on <inner> if it's a Symbol
///   name → RHS = (:wat::kernel::make-bounded-channel ...) |
///                (:wat::kernel::make-unbounded-channel ...)
///        → return Some((name, span))
///   anything else → None
fn trace_to_pair_anchor(
    name: &str,
    binding_scope: &[PairScopeEntry],
) -> Option<(String, Span)> {
    let entry = binding_scope.iter().find(|(n, _, _)| n == name)?;
    let rhs = &entry.2;
    let WatAST::List(items, span) = rhs else { return None; };
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return None,
    };
    match head {
        ":wat::kernel::make-bounded-channel" | ":wat::kernel::make-unbounded-channel" => {
            Some((name.to_string(), span.clone()))
        }
        ":wat::core::first" | ":wat::core::second" => {
            // (first <inner>) / (second <inner>); inner must be a
            // Symbol for the trace to continue. Anything else
            // (literal, nested call) bottoms out here as unknown.
            let inner = items.get(1)?;
            let WatAST::Symbol(id, _) = inner else { return None; };
            trace_to_pair_anchor(&id.name, binding_scope)
        }
        _ => None,
    }
}

/// Arc 133 — extend the pair-deadlock scope with synthetic entries
/// for a tuple-destructure binding whose RHS is a
/// `make-bounded-channel` or `make-unbounded-channel` call.
///
/// Shape: `((name1 name2 ...) (:wat::kernel::make-bounded-channel ...))`.
///
/// Creates:
///   - A synthetic anchor entry `("__arc133_pair_N", ":wat::kernel::Channel<wat::core::unit>", rhs)`
///     so `trace_to_pair_anchor` finds the allocation at the anchor name.
///   - One entry per name pointing to `(first anchor)` or `(second anchor)`
///     projection ASTs, with fabricated `Sender<unit>` / `Receiver<unit>`
///     type annotations (index 0 → Sender, index 1 → Receiver, beyond → no entry).
///     The type-annotation head is all that matters for classification;
///     the `<unit>` element type is a safe placeholder (the classifier
///     checks head only, not inner T).
///
/// This is conservative: only the first two names get classified
/// (the Sender and Receiver halves of a channel pair). Names beyond
/// index 1 are not added — extra destructured fields from a wider
/// tuple are out-of-scope for the pair-deadlock pattern.
///
/// The synthetic anchor name uses a counter prefix so that multiple
/// tuple-destructure bindings in the same let* get distinct anchors.
fn extend_pair_scope_with_tuple_destructure(
    binding: &WatAST,
    synthetic_counter: &mut usize,
    scope: &mut Vec<PairScopeEntry>,
) {
    let WatAST::List(items, _) = binding else { return; };
    if items.len() != 2 {
        return;
    }
    let WatAST::List(parts, _) = &items[0] else { return; };
    // All parts must be bare symbols (tuple-destructure shape).
    let names: Vec<String> = parts
        .iter()
        .filter_map(|p| match p {
            WatAST::Symbol(id, _) => Some(id.name.clone()),
            _ => None,
        })
        .collect();
    if names.len() != parts.len() || names.is_empty() {
        return;
    }
    // RHS must be make-bounded-channel or make-unbounded-channel.
    let rhs = &items[1];
    let WatAST::List(rhs_items, _) = rhs else { return; };
    let rhs_head = match rhs_items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return,
    };
    if !matches!(
        rhs_head,
        ":wat::kernel::make-bounded-channel" | ":wat::kernel::make-unbounded-channel"
    ) {
        return;
    }

    // Synthetic anchor: a virtual binding whose RHS IS the make-channel
    // call. `trace_to_pair_anchor` recognises make-*-channel as the
    // anchor terminus and returns this binding's name. Both name0 and
    // name1 will trace through their (first/second anchor) projections
    // and land here — same anchor name → pair-deadlock fires.
    let anchor_name = format!("__arc133_tuple_anchor_{}", *synthetic_counter);
    *synthetic_counter += 1;
    scope.push((
        anchor_name.clone(),
        ":wat::kernel::Channel<wat::core::unit>".into(),
        rhs.clone(),
    ));

    // Add sender (index 0) and receiver (index 1) projection entries.
    // Only the first two names participate in the Sender/Receiver
    // classification; further names in wider tuples don't represent
    // channel halves and are ignored.
    let projections = [
        (":wat::core::first", ":wat::kernel::Sender<wat::core::unit>"),
        (":wat::core::second", ":wat::kernel::Receiver<wat::core::unit>"),
    ];
    for (idx, name) in names.into_iter().enumerate() {
        if idx >= projections.len() {
            break;
        }
        let (proj_kw, type_ann) = projections[idx];
        let proj_rhs = WatAST::list(vec![
            WatAST::keyword(proj_kw),
            WatAST::symbol(Identifier::bare(&anchor_name)),
        ]);
        scope.push((name, type_ann.into(), proj_rhs));
    }
}

/// Parse a let* binding `((name :type-annotation) rhs)` →
/// `(name, type_annotation_keyword, rhs)`. Returns `None` on shapes
/// that don't fit (untyped bindings, tuple-destructure patterns —
/// the trace gives up conservatively).
///
/// Sibling to `parse_binding_for_typed_check` (arc 117) — that one
/// returns the binding span; this one returns the RHS so the trace
/// can chain across `(first pair)` / `(second pair)` projections.
fn parse_binding_for_pair_check(binding: &WatAST) -> Option<(String, String, WatAST)> {
    let WatAST::List(items, _) = binding else { return None; };
    if items.len() != 2 {
        return None;
    }
    let pattern = &items[0];
    let WatAST::List(parts, _) = pattern else { return None; };
    if parts.len() < 2 {
        return None;
    }
    let name = match &parts[0] {
        WatAST::Symbol(id, _) => id.name.clone(),
        _ => return None,
    };
    let type_ann_str = match &parts[1] {
        WatAST::Keyword(k, _) => k.clone(),
        _ => return None,
    };
    Some((name, type_ann_str, items[1].clone()))
}

/// Does this type, after alias resolution, denote a `Sender<T>` end
/// of a channel? Mirrors arc 117's `type_contains_sender_kind` but
/// returns a bool focused on the SURFACE type — we want the
/// argument's own type to BE a Sender, not contain one.
///
/// `expand_alias` walks user typealiases to the canonical
/// `:rust::crossbeam_channel::Sender` head. `:wat::kernel::Sender`
/// is a one-step alias to that; `:wat::kernel::Channel` is a tuple
/// (Sender, Receiver) — but a single Channel-typed argument can't
/// be both halves at once and isn't traceable as one half, so this
/// classifier focuses on the bare Sender shape.
fn type_is_sender_kind(ty: &TypeExpr, types: &TypeEnv) -> bool {
    let canonical = crate::types::expand_alias(ty, types);
    match canonical {
        TypeExpr::Parametric { head, .. } => matches!(
            head.as_str(),
            "rust::crossbeam_channel::Sender" | "wat::kernel::Sender"
        ),
        _ => false,
    }
}

/// Does this type, after alias resolution, denote a `Receiver<T>`
/// end of a channel? Mirror of `type_is_sender_kind`. Same one-step
/// alias chain: `:wat::kernel::Receiver` → `:rust::crossbeam_channel::Receiver`.
fn type_is_receiver_kind(ty: &TypeExpr, types: &TypeEnv) -> bool {
    let canonical = crate::types::expand_alias(ty, types);
    match canonical {
        TypeExpr::Parametric { head, .. } => matches!(
            head.as_str(),
            "rust::crossbeam_channel::Receiver" | "wat::kernel::Receiver"
        ),
        _ => false,
    }
}

fn check_function_body(
    path: &str,
    func: &Function,
    scheme: &TypeScheme,
    env: &CheckEnv,
    fresh: &mut InferCtx,
    errors: &mut Vec<CheckError>,
) {
    // Declared type parameters are RIGID inside the body — rigid
    // meaning they unify only with themselves. Represented as
    // `Path(":T")` where T is the declared name; the checker
    // distinguishes rigid names from fresh unification Vars.
    let locals = build_locals(&func.params, &scheme.params);
    let mut subst = Subst::new();
    // Push this function's declared return type so `infer_try`, if it
    // recurses into the body, can unify its propagated `Err` with this
    // function's own `Result<_, E>` shape.
    fresh.push_enclosing_ret(scheme.ret.clone());
    let body_ty = infer(&func.body, env, &locals, fresh, &mut subst, errors);
    fresh.pop_enclosing_ret();
    // Unify body type with declared return type. If unification fails,
    // produce a ReturnTypeMismatch.
    if let Some(body_ty) = body_ty {
        if unify(&body_ty, &scheme.ret, &mut subst, env.types()).is_err() {
            errors.push(CheckError::ReturnTypeMismatch {
                function: path.to_string(),
                expected: format_type(&apply_subst(&scheme.ret, &subst)),
                got: format_type(&apply_subst(&body_ty, &subst)),
                span: func.body.span().clone(),
            });
        }
    }
}

fn check_form(
    form: &WatAST,
    env: &CheckEnv,
    fresh: &mut InferCtx,
    errors: &mut Vec<CheckError>,
) {
    let mut subst = Subst::new();
    let _ = infer(form, env, &HashMap::new(), fresh, &mut subst, errors);
}

// ─── Inference ──────────────────────────────────────────────────────────

/// Infer the type of an expression, recording errors along the way.
///
/// Returns `Some(type)` when a type can be assigned, `None` when the
/// expression's type is opaque at this layer (e.g., lambda
/// application, user symbol that isn't a known local). Errors from
/// nested calls are pushed to `errors`.
fn infer(
    ast: &WatAST,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    match ast {
        WatAST::IntLit(_, _) => Some(TypeExpr::Path(":i64".into())),
        WatAST::FloatLit(_, _) => Some(TypeExpr::Path(":f64".into())),
        WatAST::BoolLit(_, _) => Some(TypeExpr::Path(":bool".into())),
        WatAST::StringLit(_, _) => Some(TypeExpr::Path(":String".into())),
        // `:None` / `:wat::core::None` — nullary constructor of the
        // built-in :Option<T> enum. Infers as `:Option<T>` with a
        // fresh T; unification against the expected type sharpens T
        // at the use site.
        //
        // Arc 109 slice 1h: bare `:None` is a retiring grammar
        // exception. Pattern 2 poison fires synthetic TypeMismatch
        // with redirect to the FQDN form; the type still resolves
        // so the program type-checks the rest of the way.
        WatAST::Keyword(k, kw_span) if (k == ":None" || k == ":wat::core::None") => {
            if k == ":None" {
                errors.push(CheckError::TypeMismatch {
                    callee: ":None".into(),
                    param: "(retired bare-keyword exception)".into(),
                    expected: ":wat::core::None".into(),
                    got: ":None".into(),
                    span: kw_span.clone(),
                });
            }
            Some(TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![fresh.fresh()],
            })
        }
        // Arc 048 — user-enum unit variant. The bare keyword resolves
        // to the enum's type (e.g. `:trading::types::PhaseLabel::Valley`
        // → `:trading::types::PhaseLabel`).
        WatAST::Keyword(k, _) if env.unit_variant_type(k).is_some() => {
            Some(env.unit_variant_type(k).expect("guard").clone())
        }
        // Arc 009 — names are values. If the keyword is a registered
        // function (user define, stdlib define, or builtin primitive),
        // instantiate its scheme and return a `:fn(...)->Ret` type so
        // the keyword can be passed to any `:fn(...)`-typed parameter.
        // Mirrors `infer_spawn`'s long-standing keyword-path path,
        // generalized to every expression position.
        WatAST::Keyword(k, _) if env.get(k).is_some() => {
            let scheme = env.get(k).expect("guard").clone();
            let (params, ret) = instantiate(&scheme, fresh);
            Some(TypeExpr::Fn {
                args: params,
                ret: Box::new(ret),
            })
        }
        WatAST::Keyword(_, _) => Some(TypeExpr::Path(":wat::core::keyword".into())),
        WatAST::Symbol(ident, _) => locals.get(&ident.name).cloned(),
        WatAST::List(items, _) => infer_list(items, env, locals, fresh, subst, errors),
    }
}

fn infer_list(
    items: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // `()` — empty list — is the unit value. Type :() per the
    // existing TypeExpr::Tuple([]) encoding.
    let head = match items.first() {
        Some(h) => h,
        None => return Some(TypeExpr::Tuple(vec![])),
    };

    if let WatAST::Keyword(k, head_span) = head {
        let args = &items[1..];
        match k.as_str() {
            ":wat::core::if" => return infer_if(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::cond" => return infer_cond(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::let" => return infer_let(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::let*" => return infer_let_star(args, head_span, env, locals, fresh, subst, errors),
            // Arc 109 slice 1j — § D' Option/Result method forms.
            // Three retired verbs (Pattern 2 poison + dispatch) and
            // four new canonical heads (the three renames plus the
            // brand-new Option-side propagation primitive).
            ":wat::core::try" => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::try".into(),
                    param: "(retired verb)".into(),
                    expected: ":wat::core::Result/try".into(),
                    got: ":wat::core::try".into(),
                    span: head_span.clone(),
                });
                return infer_try(":wat::core::try", head_span, args, env, locals, fresh, subst, errors);
            }
            ":wat::core::option::expect" => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::option::expect".into(),
                    param: "(retired verb)".into(),
                    expected: ":wat::core::Option/expect".into(),
                    got: ":wat::core::option::expect".into(),
                    span: head_span.clone(),
                });
                return infer_option_expect(
                    ":wat::core::option::expect",
                    head_span, args, env, locals, fresh, subst, errors,
                );
            }
            ":wat::core::result::expect" => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::result::expect".into(),
                    param: "(retired verb)".into(),
                    expected: ":wat::core::Result/expect".into(),
                    got: ":wat::core::result::expect".into(),
                    span: head_span.clone(),
                });
                return infer_result_expect(
                    ":wat::core::result::expect",
                    head_span, args, env, locals, fresh, subst, errors,
                );
            }
            ":wat::core::Result/try" => {
                return infer_try(":wat::core::Result/try", head_span, args, env, locals, fresh, subst, errors);
            }
            ":wat::core::Option/try" => {
                return infer_option_try(
                    ":wat::core::Option/try",
                    head_span, args, env, locals, fresh, subst, errors,
                );
            }
            ":wat::core::Option/expect" => {
                return infer_option_expect(
                    ":wat::core::Option/expect",
                    head_span, args, env, locals, fresh, subst, errors,
                );
            }
            ":wat::core::Result/expect" => {
                return infer_result_expect(
                    ":wat::core::Result/expect",
                    head_span, args, env, locals, fresh, subst, errors,
                );
            }
            ":wat::core::vec" => {
                // Arc 109 slice 1f — :wat::core::vec retires; the
                // canonical constructor is :wat::core::Vector
                // (verb-equals-type per INVENTORY § D). Pattern 2
                // poison: push a synthetic TypeMismatch so every
                // call site fires a hint, but continue to dispatch
                // so the program still type-checks the rest of the
                // way (consumers sweep call-by-call, no cliff).
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::vec".into(),
                    param: "(retired verb)".into(),
                    expected: ":wat::core::Vector".into(),
                    got: ":wat::core::vec".into(),
                    span: head_span.clone(),
                });
                return infer_list_constructor(args, head_span, env, locals, fresh, subst, errors);
            }
            ":wat::core::Vector" => return infer_list_constructor(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::list" => {
                // Arc 109 slice 1g — :wat::core::list retires.
                // Was always a duplicate of :wat::core::vec; both
                // produced :Vec<T>. Post-slice-1f, :wat::core::Vector
                // is the canonical constructor. Pattern 2 poison:
                // synthetic TypeMismatch + redirect; continue to
                // dispatch so the program type-checks.
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::list".into(),
                    param: "(retired verb)".into(),
                    expected: ":wat::core::Vector".into(),
                    got: ":wat::core::list".into(),
                    span: head_span.clone(),
                });
                return infer_list_constructor(args, head_span, env, locals, fresh, subst, errors);
            }
            ":wat::core::tuple" => {
                // Arc 109 slice 1g — :wat::core::tuple retires;
                // canonical constructor is :wat::core::Tuple
                // (verb-equals-type per slice 1f's vec→Vector
                // playbook). Pattern 2 poison.
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::tuple".into(),
                    param: "(retired verb)".into(),
                    expected: ":wat::core::Tuple".into(),
                    got: ":wat::core::tuple".into(),
                    span: head_span.clone(),
                });
                return infer_tuple_constructor(args, head_span, env, locals, fresh, subst, errors);
            }
            ":wat::core::Tuple" => return infer_tuple_constructor(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::string::concat" => return infer_string_concat(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::HashMap" => return infer_hashmap_constructor(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::assoc" => return infer_assoc(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::concat" => return infer_concat(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::dissoc" => return infer_dissoc(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::keys" => return infer_keys(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::values" => return infer_values(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::empty?" => return infer_empty_q(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::conj" => return infer_conj(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::contains?" => return infer_contains_q(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::length" => return infer_length(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::HashSet" => return infer_hashset_constructor(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::get" => return infer_get(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::quote" => {
                // Quote captures an unevaluated AST. The argument is
                // DATA, not an expression — the type checker does not
                // recurse into it. Return type is `:wat::WatAST`.
                if args.len() != 1 {
                    errors.push(CheckError::ArityMismatch {
                        callee: ":wat::core::quote".into(),
                        expected: 1,
                        got: args.len(),
                        span: head_span.clone(),
                    });
                }
                return Some(TypeExpr::Path(":wat::WatAST".into()));
            }
            ":wat::core::forms" => {
                // Variadic sibling of quote. Every positional arg is
                // DATA, captured as `:wat::WatAST`. The checker does
                // not recurse into any of them. Return type is
                // `:Vec<wat::WatAST>` regardless of arity (including
                // zero, which produces an empty Vec).
                return Some(TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":wat::WatAST".into())],
                });
            }
            ":wat::core::struct->form" => {
                // Arc 091 slice 8 — lift a struct VALUE to its
                // constructor-call FORM. ∀T. T → :wat::WatAST. The
                // arg's type is inferred for context but not
                // constrained (the runtime errors if T isn't a
                // Struct). Return type is :wat::WatAST.
                if args.len() != 1 {
                    errors.push(CheckError::ArityMismatch {
                        callee: ":wat::core::struct->form".into(),
                        expected: 1,
                        got: args.len(),
                        span: head_span.clone(),
                    });
                } else {
                    let _ = infer(&args[0], env, locals, fresh, subst, errors);
                }
                return Some(TypeExpr::Path(":wat::WatAST".into()));
            }
            ":wat::runtime::lookup-define"
            | ":wat::runtime::signature-of"
            | ":wat::runtime::body-of" => {
                // Arc 143 slice 1 — runtime introspection primitives.
                // Each takes a keyword name (`:fn::path`) and returns
                // `:Option<wat::holon::HolonAST>`. The argument may be
                // a known function name (inferred as its fn-type by arc
                // 009) or an unknown bare keyword. In either case, arity
                // must be exactly 1; the argument's inferred type is NOT
                // unified against `:wat::core::keyword` — the runtime
                // evaluates the keyword path at dispatch time and does its
                // own lookup. The type scheme registered for these
                // primitives in `register_builtins` handles call sites
                // whose first arg is an ordinary keyword; this special
                // case handles the arc-009 "names are values" path.
                if args.len() != 1 {
                    errors.push(CheckError::ArityMismatch {
                        callee: k.to_string(),
                        expected: 1,
                        got: args.len(),
                        span: head_span.clone(),
                    });
                }
                // Infer the argument for side-effects (e.g., symbol
                // resolution in the local environment) but do not
                // constrain its type — any keyword or function-valued
                // keyword is accepted.
                if args.len() >= 1 {
                    let _ = infer(&args[0], env, locals, fresh, subst, errors);
                }
                return Some(TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":wat::holon::HolonAST".into())],
                });
            }
            ":wat::runtime::rename-callable-name" => {
                // Arc 143 slice 3 — rename-callable-name.
                // (head :HolonAST) (from :keyword) (to :keyword) -> :HolonAST
                // The first arg is a HolonAST value (may come from
                // `signature-of` which returns Option<HolonAST>; caller
                // unwraps it first). We infer all args for side-effects
                // but do not enforce type constraints — the runtime does
                // its own validation. Arity must be exactly 3.
                let expected_arity = 3;
                if args.len() != expected_arity {
                    errors.push(CheckError::ArityMismatch {
                        callee: k.to_string(),
                        expected: expected_arity,
                        got: args.len(),
                        span: head_span.clone(),
                    });
                }
                for arg in args.iter() {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return Some(TypeExpr::Path(":wat::holon::HolonAST".into()));
            }
            ":wat::runtime::extract-arg-names" => {
                // Arc 143 slice 3 — extract-arg-names.
                // (head :HolonAST) -> :Vec<keyword>
                // First arg is a HolonAST (not a keyword), so normal
                // type-scheme unification would fail on arc-009 call
                // sites. Infer for side-effects; return the concrete type.
                if args.len() != 1 {
                    errors.push(CheckError::ArityMismatch {
                        callee: k.to_string(),
                        expected: 1,
                        got: args.len(),
                        span: head_span.clone(),
                    });
                }
                if args.len() >= 1 {
                    let _ = infer(&args[0], env, locals, fresh, subst, errors);
                }
                return Some(TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":wat::core::keyword".into())],
                });
            }
            ":wat::core::macroexpand-1" | ":wat::core::macroexpand" => {
                // Arc 030: macro debugging primitives.
                // (:wat::core::macroexpand{-1}? <wat::WatAST>) -> :wat::WatAST
                if args.len() != 1 {
                    errors.push(CheckError::ArityMismatch {
                        callee: k.clone(),
                        expected: 1,
                        got: args.len(),
                        span: head_span.clone(),
                    });
                    return Some(TypeExpr::Path(":wat::WatAST".into()));
                }
                if let Some(arg_ty) = infer(&args[0], env, locals, fresh, subst, errors) {
                    let expected = TypeExpr::Path(":wat::WatAST".into());
                    if unify(&arg_ty, &expected, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: k.clone(),
                            param: "#1".into(),
                            expected: format_type(&apply_subst(&expected, subst)),
                            got: format_type(&apply_subst(&arg_ty, subst)),
                            span: args[0].span().clone(),
                        });
                    }
                }
                return Some(TypeExpr::Path(":wat::WatAST".into()));
            }
            ":wat::core::match" => {
                return infer_match(args, head_span, env, locals, fresh, subst, errors);
            }
            // Arc 050 — polymorphic comparison/equality. Same-type
            // for non-numeric, cross-numeric promotion for (i64, f64)
            // pairs. Always returns :bool. `not=` (Clojure tradition)
            // shares the inference path with `=` since the rules are
            // identical; only the runtime differs.
            ":wat::core::="
            | ":wat::core::not="
            | ":wat::core::<"
            | ":wat::core::>"
            | ":wat::core::<="
            | ":wat::core::>=" => {
                return infer_polymorphic_compare(k, head_span, args, env, locals, fresh, subst, errors);
            }
            // Arc 050 — polymorphic arithmetic. Both args must be
            // numeric (i64 or f64); result type is f64 if either is
            // f64, else i64.
            ":wat::core::+"
            | ":wat::core::-"
            | ":wat::core::*"
            | ":wat::core::/" => {
                return infer_polymorphic_arith(k, head_span, args, env, locals, fresh, subst, errors);
            }
            // Arc 097 slice 2 — polymorphic Instant ± Duration. Result
            // type depends on the RHS variant:
            //   Instant - Duration -> Instant
            //   Instant - Instant  -> Duration
            //   Instant + Duration -> Instant
            ":wat::time::-" | ":wat::time::+" => {
                return infer_polymorphic_time_arith(
                    k, head_span, args, env, locals, fresh, subst, errors,
                );
            }
            // Arc 098 — Clara-style single-item pattern matcher.
            // Substrate-recognized special form (macros expand before
            // type-checking and can't query the struct registry).
            ":wat::form::matches?" => {
                return infer_form_matches(args, head_span, env, locals, fresh, subst, errors);
            }
            // Arc 052 — polymorphic algebra ops. Cosine and dot accept
            // HolonAST or Vector in either position; simhash accepts
            // HolonAST or Vector as its single argument. Arc 061
            // extends the polymorphism to coincident? (mirroring
            // cosine's shape; differs only in the bool return type).
            ":wat::holon::cosine" | ":wat::holon::dot" => {
                return infer_polymorphic_holon_pair_to_f64(
                    k, head_span, args, env, locals, fresh, subst, errors,
                );
            }
            ":wat::holon::coincident?" => {
                return infer_polymorphic_holon_pair_to_bool(
                    k, head_span, args, env, locals, fresh, subst, errors,
                );
            }
            ":wat::holon::coincident-explain" => {
                return infer_polymorphic_holon_pair_to_path(
                    k,
                    head_span,
                    args,
                    env,
                    locals,
                    fresh,
                    subst,
                    errors,
                    ":wat::holon::CoincidentExplanation",
                );
            }
            ":wat::holon::simhash" => {
                return infer_polymorphic_holon_to_i64(
                    k, head_span, args, env, locals, fresh, subst, errors,
                );
            }
            ":wat::kernel::make-bounded-queue" => {
                return infer_make_queue(
                    args,
                    head_span,
                    env,
                    locals,
                    fresh,
                    subst,
                    errors,
                    ":wat::kernel::make-bounded-queue",
                    /*with_capacity=*/ true,
                );
            }
            ":wat::kernel::make-unbounded-queue" => {
                return infer_make_queue(
                    args,
                    head_span,
                    env,
                    locals,
                    fresh,
                    subst,
                    errors,
                    ":wat::kernel::make-unbounded-queue",
                    /*with_capacity=*/ false,
                );
            }
            ":wat::kernel::drop" => {
                return infer_drop(args, head_span, env, locals, fresh, subst, errors);
            }
            ":wat::kernel::spawn" => {
                return infer_spawn(args, head_span, env, locals, fresh, subst, errors);
            }
            // Arc 114 — `:wat::kernel::join` and `:wat::kernel::join-result`
            // retire alongside `:wat::kernel::spawn`. Push synthetic
            // TypeMismatches at every call site so the arc 114 migration
            // hint fires; continue inferring the args (so additional
            // mismatches still surface) but the call itself is now an
            // error regardless of arg shapes.
            ":wat::kernel::join" => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::kernel::join".into(),
                    param: "(retired verb)".into(),
                    expected: ":wat::kernel::Thread/join-result".into(),
                    got: ":wat::kernel::join".into(),
                    span: head_span.clone(),
                });
                for arg in args {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return Some(fresh.fresh());
            }
            ":wat::kernel::join-result" => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::kernel::join-result".into(),
                    param: "(retired verb)".into(),
                    expected: ":wat::kernel::Thread/join-result".into(),
                    got: ":wat::kernel::join-result".into(),
                    span: head_span.clone(),
                });
                for arg in args {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return Some(fresh.fresh());
            }
            ":wat::core::first" => {
                return infer_positional_accessor(args, head_span, env, locals, fresh, subst, errors, ":wat::core::first", 0);
            }
            ":wat::core::second" => {
                return infer_positional_accessor(args, head_span, env, locals, fresh, subst, errors, ":wat::core::second", 1);
            }
            ":wat::core::third" => {
                return infer_positional_accessor(args, head_span, env, locals, fresh, subst, errors, ":wat::core::third", 2);
            }
            ":wat::core::and" | ":wat::core::or" => {
                return infer_boolean_shortcircuit(args, head_span, env, locals, fresh, subst, errors);
            }
            ":wat::core::lambda" => return infer_lambda(args, head_span, env, locals, fresh, subst, errors),
            ":wat::core::use!" => {
                // use! is a resolve-pass declaration. It validates at
                // resolve time; the type checker treats it as a no-op
                // returning :(). The argument is a keyword path; we
                // don't recurse into it.
                return Some(TypeExpr::Tuple(vec![]));
            }
            _ if k.starts_with(":rust::") => {
                return dispatch_rust_scheme(k, head_span, args, env, locals, fresh, subst, errors);
            }
            ":wat::core::define"
            | ":wat::core::struct"
            | ":wat::core::enum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
            | ":wat::core::defmacro"
            | ":wat::load-file!"
            | ":wat::digest-load!"
            | ":wat::signed-load!"
            | ":wat::core::unquote"
            | ":wat::core::unquote-splicing" => {
                // Top-level forms / reader-macro heads don't participate
                // in expression-level inference.
                return None;
            }
            ":wat::core::quasiquote" => {
                // Arc 091 slice 8 — runtime quasiquote returns
                // :wat::WatAST. Body isn't fully type-checked (it's
                // a template); unquoted expressions infer into
                // local context but their types don't constrain the
                // outer result.
                return Some(TypeExpr::Path(":wat::WatAST".into()));
            }
            _ if k.starts_with(":wat::config::set-") => return None,
            _ if (k.starts_with(":wat::kernel::") || k.starts_with(":wat::std::"))
                && !k.starts_with(":wat::std::math::")
                && env.get(k).is_none() =>
            {
                // Unknown kernel / std path with no registered scheme —
                // accept and recurse. Math lives at `:wat::std::math::*`
                // and has registered schemes; exclude it so the normal
                // scheme lookup below kicks in.
                for arg in args {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return None;
            }
            _ => {}
        }

        // Normal call: look up scheme, instantiate, unify args.
        // Arc 139 — strip turbofish `<T,...>` from the head before
        // env.get. The substrate registers user defines under the
        // canonical name (sans turbofish); call sites that use
        // turbofish resolve to the same scheme. Symmetric registration
        // vs lookup. See `runtime::canonical_callable_name` for the
        // full rationale.
        let canonical_k = crate::runtime::canonical_callable_name(k);
        let scheme = match env.get(canonical_k) {
            Some(s) => s,
            None => {
                // Resolve pass validated the name; we just don't have
                // a scheme for it (e.g., user function not registered
                // in this run). Still recurse into args for nested
                // checks.
                for arg in args {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return None;
            }
        };

        let (param_types, ret_type) = instantiate(scheme, fresh);

        if args.len() != param_types.len() {
            errors.push(CheckError::ArityMismatch {
                callee: k.clone(),
                expected: param_types.len(),
                got: args.len(),
                span: head_span.clone(),
            });
            for arg in args {
                let _ = infer(arg, env, locals, fresh, subst, errors);
            }
            return Some(apply_subst(&ret_type, subst));
        }

        for (i, (arg, expected)) in args.iter().zip(&param_types).enumerate() {
            let arg_ty = infer(arg, env, locals, fresh, subst, errors);
            if let Some(arg_ty) = arg_ty {
                if unify(&arg_ty, expected, subst, env.types()).is_err() {
                    errors.push(CheckError::TypeMismatch {
                        callee: k.clone(),
                        param: format!("#{}", i + 1),
                        expected: format_type(&apply_subst(expected, subst)),
                        got: format_type(&apply_subst(&arg_ty, subst)),
                        span: arg.span().clone(),
                    });
                }
            }
        }
        return Some(apply_subst(&ret_type, subst));
    }

    // Arc 109 slice 1h — Option `Some` constructor recognized at
    // both bare-Symbol (legacy grammar exception, poisoned) and
    // FQDN-Keyword (canonical) heads. `(Some expr)` and
    // `(:wat::core::Some expr)` both infer as `:Option<T>` where T
    // is the argument's type.
    let head_is_some_bare = matches!(
        head,
        WatAST::Symbol(ident, _) if ident.as_str() == "Some"
    );
    let head_is_some_fqdn = matches!(
        head,
        WatAST::Keyword(k, _) if k == ":wat::core::Some"
    );
    if head_is_some_bare || head_is_some_fqdn {
        if head_is_some_bare {
            // Pattern 2 poison — bare `Some` is a retiring grammar
            // exception. Push synthetic TypeMismatch with redirect
            // to the FQDN form; continue dispatching so the program
            // type-checks the rest of the way.
            errors.push(CheckError::TypeMismatch {
                callee: "Some".into(),
                param: "(retired bare-symbol exception)".into(),
                expected: ":wat::core::Some".into(),
                got: "Some".into(),
                span: head.span().clone(),
            });
        }
        let args = &items[1..];
        if args.len() != 1 {
            errors.push(CheckError::ArityMismatch {
                callee: if head_is_some_bare { "Some".into() } else { ":wat::core::Some".into() },
                expected: 1,
                got: args.len(),
                span: head.span().clone(),
            });
            for arg in args {
                let _ = infer(arg, env, locals, fresh, subst, errors);
            }
            return Some(TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![fresh.fresh()],
            });
        }
        let inner_ty = infer(&args[0], env, locals, fresh, subst, errors)
            .unwrap_or_else(|| fresh.fresh());
        return Some(TypeExpr::Parametric {
            head: "Option".into(),
            args: vec![inner_ty],
        });
    }
    // Arc 109 slice 1i — Result `Ok` / `Err` constructors recognized
    // at both bare-Symbol (legacy grammar exception, poisoned) and
    // FQDN-Keyword (canonical) heads. Mirrors slice 1h's Some shape.
    let head_is_ok_bare = matches!(
        head,
        WatAST::Symbol(ident, _) if ident.as_str() == "Ok"
    );
    let head_is_ok_fqdn = matches!(
        head,
        WatAST::Keyword(k, _) if k == ":wat::core::Ok"
    );
    if head_is_ok_bare || head_is_ok_fqdn {
        if head_is_ok_bare {
            errors.push(CheckError::TypeMismatch {
                callee: "Ok".into(),
                param: "(retired bare-symbol exception)".into(),
                expected: ":wat::core::Ok".into(),
                got: "Ok".into(),
                span: head.span().clone(),
            });
        }
        let args = &items[1..];
        if args.len() != 1 {
            errors.push(CheckError::ArityMismatch {
                callee: if head_is_ok_bare { "Ok".into() } else { ":wat::core::Ok".into() },
                expected: 1,
                got: args.len(),
                span: head.span().clone(),
            });
            for arg in args {
                let _ = infer(arg, env, locals, fresh, subst, errors);
            }
            return Some(TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![fresh.fresh(), fresh.fresh()],
            });
        }
        let t_ty = infer(&args[0], env, locals, fresh, subst, errors)
            .unwrap_or_else(|| fresh.fresh());
        let e_var = fresh.fresh();
        return Some(TypeExpr::Parametric {
            head: "Result".into(),
            args: vec![t_ty, e_var],
        });
    }
    let head_is_err_bare = matches!(
        head,
        WatAST::Symbol(ident, _) if ident.as_str() == "Err"
    );
    let head_is_err_fqdn = matches!(
        head,
        WatAST::Keyword(k, _) if k == ":wat::core::Err"
    );
    if head_is_err_bare || head_is_err_fqdn {
        if head_is_err_bare {
            errors.push(CheckError::TypeMismatch {
                callee: "Err".into(),
                param: "(retired bare-symbol exception)".into(),
                expected: ":wat::core::Err".into(),
                got: "Err".into(),
                span: head.span().clone(),
            });
        }
        let args = &items[1..];
        if args.len() != 1 {
            errors.push(CheckError::ArityMismatch {
                callee: if head_is_err_bare { "Err".into() } else { ":wat::core::Err".into() },
                expected: 1,
                got: args.len(),
                span: head.span().clone(),
            });
            for arg in args {
                let _ = infer(arg, env, locals, fresh, subst, errors);
            }
            return Some(TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![fresh.fresh(), fresh.fresh()],
            });
        }
        let e_ty = infer(&args[0], env, locals, fresh, subst, errors)
            .unwrap_or_else(|| fresh.fresh());
        let t_var = fresh.fresh();
        return Some(TypeExpr::Parametric {
            head: "Result".into(),
            args: vec![t_var, e_ty],
        });
    }

    // Non-keyword head (bare symbol or inline expression). Not typed
    // at this layer pending your call on explicit let-binding type
    // annotations. Recurse into args so nested keyword-headed calls
    // still get checked.
    for item in items {
        let _ = infer(item, env, locals, fresh, subst, errors);
    }
    None
}

/// Type-check `(:wat::core::match scrutinee arm...)`. Scrutinee must
/// be `:Option<T>` (the only built-in enum in this slice). Each arm's
/// pattern introduces bindings visible in its body; every arm body's
/// type unifies to a common result type. Exhaustiveness: at least one
/// arm matches `:None` (either the `:None` pattern or a wildcard) and
/// at least one arm matches `(Some _)` (either the `Some` pattern or
/// a wildcard).
/// `(:wat::core::match scrutinee -> :T arm1 arm2 ...)` — typed match.
///
/// Per the 2026-04-20 INSCRIPTION, match now requires an explicit
/// `-> :T` declaration between the scrutinee and the arms. Every
/// arm body is checked against `:T` independently so divergent
/// arms produce a per-arm TypeMismatch naming the declared type.
/// The old no-annotation form is refused with a migration-hint
/// MalformedForm.
fn infer_match(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // Pre-inscription shape detection: if args[1] isn't `->`, this
    // is the old form. Surface a migration-hint error before the
    // standard arity check so authors see the right guidance.
    if args.len() >= 2
        && !matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->")
    {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: "`:wat::core::match` now requires `-> :T` between scrutinee and arms; write (:wat::core::match scrut -> :T (pat body) ...)".into(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    if args.len() < 4 {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: format!(
                "expected (:wat::core::match scrut -> :T arm1 arm2 ...) — at least 4 args; got {}",
                args.len()
            ),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    // Parse the declared `:T`.
    let declared_ty = match &args[2] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e),
                    span: args[2].span().clone(),
                });
                return None;
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: "expected type keyword after `->`".into(),
                span: args[2].span().clone(),
            });
            return None;
        }
    };

    // Detect shape from the arms (arms begin at args[3..]).
    let arm_refs: Vec<&WatAST> = args[3..].iter().collect();
    let shape = detect_match_shape(&arm_refs, env, fresh);

    // Scrutinee must unify with the detected shape.
    let scrutinee_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let expected_scrutinee = shape.as_type();
    if let Some(sty) = &scrutinee_ty {
        if unify(sty, &expected_scrutinee, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::core::match".into(),
                param: "scrutinee".into(),
                expected: format_type(&expected_scrutinee),
                got: format_type(&apply_subst(sty, subst)),
                span: args[0].span().clone(),
            });
        }
    }

    // Arc 055 — resolve the shape's inner types via the substitution
    // *now* so recursive sub-pattern checking sees concrete types
    // (e.g. `Option<fresh>` → `Option<(i64,i64,i64)>` once the
    // scrutinee unifies with a let-bound variable).
    let shape = match &shape {
        MatchShape::Option(t) => MatchShape::Option(apply_subst(t, subst)),
        MatchShape::Result(t, e) => {
            MatchShape::Result(apply_subst(t, subst), apply_subst(e, subst))
        }
        MatchShape::Enum(p, args) => MatchShape::Enum(
            p.clone(),
            args.iter().map(|a| apply_subst(a, subst)).collect(),
        ),
    };

    let mut covers_option_none = false;
    let mut covers_option_some = false;
    let mut covers_result_ok = false;
    let mut covers_result_err = false;
    let mut wildcard_seen = false;
    // Arc 111 — when the Result's Ok-inner type is Option<T>, the
    // caller may write two partial Ok arms — `(Ok (Some v))` and
    // `(Ok :None)` — that together cover all Ok cases. Track them
    // separately so the combined check can set covers_result_ok.
    let mut covers_result_ok_inner_some = false;
    let mut covers_result_ok_inner_none = false;
    // Arc 048 — track which user-enum variant names have arms.
    let mut covered_enum_variants: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for (idx, arm) in args[3..].iter().enumerate() {
        let arm_items = match arm {
            WatAST::List(items, _) if items.len() == 2 => items,
            _ => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!("arm #{} must be `(pattern body)`", idx + 1),
                    span: arm.span().clone(),
                });
                continue;
            }
        };
        let pattern = &arm_items[0];
        let body = &arm_items[1];

        let mut arm_locals = locals.clone();
        match pattern_coverage(pattern, &shape, env, &mut arm_locals, errors) {
            Some(Coverage::OptionNone) => covers_option_none = true,
            // Arc 055 — partial Some (e.g. `(Some (1 _))`) does not
            // satisfy Some-coverage; needs a fallback arm.
            Some(Coverage::OptionSome { full: true }) => covers_option_some = true,
            Some(Coverage::OptionSome { full: false }) => {}
            Some(Coverage::ResultOk { full: true }) => covers_result_ok = true,
            // Arc 111 — partial Ok arm. Check if this is one of the
            // two canonical Option-unwrapping sub-arms:
            //   `(Ok (Some v))` → covers_result_ok_inner_some
            //   `(Ok :None)`    → covers_result_ok_inner_none
            // If both are seen and the inner type IS Option<T>, the
            // pair together constitutes full Ok coverage.
            Some(Coverage::ResultOk { full: false }) => {
                if let WatAST::List(pat_items, _) = pattern {
                    if let Some(sub) = pat_items.get(1) {
                        match sub {
                            WatAST::List(sub_items, _)
                                if sub_items.first().map(|h| {
                                    matches!(h, WatAST::Symbol(s, _) if s.as_str() == "Some")
                                        || matches!(h, WatAST::Keyword(k, _) if k == ":wat::core::Some")
                                }).unwrap_or(false) =>
                            {
                                covers_result_ok_inner_some = true;
                            }
                            WatAST::Keyword(k, _) if (k.as_str() == ":None" || k.as_str() == ":wat::core::None") => {
                                covers_result_ok_inner_none = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Some(Coverage::ResultErr { full: true }) => covers_result_err = true,
            Some(Coverage::ResultErr { full: false }) => {}
            Some(Coverage::EnumVariant { name, full: true }) => {
                covered_enum_variants.insert(name);
            }
            Some(Coverage::EnumVariant { full: false, .. }) => {}
            Some(Coverage::Wildcard) => {
                wildcard_seen = true;
                covers_option_none = true;
                covers_option_some = true;
                covers_result_ok = true;
                covers_result_err = true;
            }
            None => continue,
        }
        // Arc 111 — combined Option-unwrapping Ok coverage: if the
        // Result's inner Ok type is Option<T> and we've seen both
        // `(Ok (Some _))` and `(Ok :None)` arms, promote to full Ok
        // coverage. This lets recv-loop workers use the 3-arm pattern
        // `(Ok (Some v)) / (Ok :None) / (Err _died)` without a
        // fallback arm.
        if covers_result_ok_inner_some && covers_result_ok_inner_none {
            if let MatchShape::Result(ok_ty, _) = &shape {
                if matches!(ok_ty, TypeExpr::Parametric { head, .. } if head == "Option") {
                    covers_result_ok = true;
                }
            }
        }

        // Each arm body checked against the declared `:T` independently.
        let arm_ty = infer(body, env, &arm_locals, fresh, subst, errors);
        if let Some(t) = arm_ty {
            if unify(&t, &declared_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::match".into(),
                    param: format!("arm #{}", idx + 1),
                    expected: format_type(&apply_subst(&declared_ty, subst)),
                    got: format_type(&apply_subst(&t, subst)),
                    span: body.span().clone(),
                });
            }
        }
    }

    let exhaustive = match &shape {
        MatchShape::Option(_) => covers_option_none && covers_option_some,
        MatchShape::Result(_, _) => covers_result_ok && covers_result_err,
        MatchShape::Enum(enum_path, _) => {
            if wildcard_seen {
                true
            } else if let Some(crate::types::TypeDef::Enum(e)) = env.types().get(enum_path) {
                e.variants.iter().all(|v| {
                    let name = match v {
                        crate::types::EnumVariant::Unit(n) => n,
                        crate::types::EnumVariant::Tagged { name, .. } => name,
                    };
                    covered_enum_variants.contains(name)
                })
            } else {
                false
            }
        }
    };
    if !exhaustive {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::match".into(),
            reason: match &shape {
                MatchShape::Option(_) => "non-exhaustive: :Option<T> needs arms for both :None and (Some _), or a wildcard. (Arc 055 — narrowing patterns like `(Some (1 _))` are partial; add a fallback `_` arm.)".into(),
                MatchShape::Result(_, _) => "non-exhaustive: :Result<T,E> needs arms for both (Ok _) and (Err _), or a wildcard. (Arc 055 — narrowing patterns like `(Ok 200)` are partial; add a fallback `_` arm.)".into(),
                MatchShape::Enum(enum_path, _) => {
                    if let Some(crate::types::TypeDef::Enum(e)) = env.types().get(enum_path) {
                        let missing: Vec<String> = e.variants.iter().filter_map(|v| {
                            let name = match v {
                                crate::types::EnumVariant::Unit(n) => n,
                                crate::types::EnumVariant::Tagged { name, .. } => name,
                            };
                            if covered_enum_variants.contains(name) {
                                None
                            } else {
                                Some(name.clone())
                            }
                        }).collect();
                        format!(
                            "non-exhaustive: enum {} missing arm(s) for variant(s): {} (or include `_` wildcard)",
                            enum_path,
                            missing.join(", ")
                        )
                    } else {
                        format!("non-exhaustive: enum {} missing arms (or include `_` wildcard)", enum_path)
                    }
                }
            },
            span: head_span.clone(),
        });
    }

    Some(apply_subst(&declared_ty, subst))
}

/// Coverage class for a match pattern. Spans built-in `:Option<T>`,
/// `:Result<T,E>`, and (arc 048) user-defined enums. Wildcard covers
/// any shape.
///
/// Arc 055 — variant-carrying coverage classes carry a `full` flag.
/// `full=true` means the variant arm's inner sub-pattern is fully
/// general (bare symbol or `_` recursively); `full=false` means the
/// arm narrows the variant's space (a literal or nested variant
/// somewhere inside) and a fallback wildcard arm is required to
/// remain exhaustive.
enum Coverage {
    OptionNone,
    OptionSome { full: bool },
    ResultOk { full: bool },
    ResultErr { full: bool },
    /// Arc 048 — user-enum variant covered. Carries the variant's
    /// bare name (e.g. "Valley") for exhaustiveness checking against
    /// the enum's declared variant set. Arc 055 — `full` flag tracks
    /// whether the inner sub-pattern is fully general.
    EnumVariant {
        name: String,
        full: bool,
    },
    Wildcard,
}

/// Which shape the match dispatches on. Determined by inspecting the
/// first variant-constructor arm.
#[derive(Clone, Debug)]
enum MatchShape {
    /// :Option<T> — inner_ty is T.
    Option(TypeExpr),
    /// :Result<T,E> — t_ty is T (Ok-inner), e_ty is E (Err-inner).
    Result(TypeExpr, TypeExpr),
    /// Arc 048 — user-defined enum. Carries the enum's full type path
    /// (e.g. `:trading::types::PhaseLabel`) and its type arguments
    /// (fresh tyvars at construction; resolved via subst as inference
    /// proceeds). Empty arg vec ↔ non-parametric enum (`PhaseLabel`);
    /// non-empty ↔ parametric enum (`Request<K,V>`). Arc 119 surfaced
    /// the parametric case — Option/Result already carried their args
    /// via dedicated variants; user-defined enums needed the same so
    /// the scrutinee type unifies against `Enum<…>` not bare `Enum`.
    Enum(String, Vec<TypeExpr>),
}

impl MatchShape {
    fn as_type(&self) -> TypeExpr {
        match self {
            MatchShape::Option(t) => TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![t.clone()],
            },
            MatchShape::Result(t, e) => TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![t.clone(), e.clone()],
            },
            MatchShape::Enum(path, args) => {
                if args.is_empty() {
                    TypeExpr::Path(path.clone())
                } else {
                    TypeExpr::Parametric {
                        head: path.trim_start_matches(':').to_string(),
                        args: args.clone(),
                    }
                }
            }
        }
    }
}

/// Construct a MatchShape::Enum carrying the right arity of fresh
/// type variables for the named enum's parametric signature. For a
/// non-parametric enum the args vec is empty and `as_type()` produces
/// `TypeExpr::Path`. For a parametric enum (`Request<K,V>`) the args
/// vec has one fresh tyvar per type param and `as_type()` produces
/// `TypeExpr::Parametric { head, args }` so the scrutinee unification
/// preserves the parametric envelope. Arc 119 surfaced this — see
/// `parametric_user_enum_*_match` tests.
fn enum_match_shape(
    enum_path: String,
    env: &CheckEnv,
    fresh: &mut InferCtx,
) -> MatchShape {
    let arity = match env.types().get(&enum_path) {
        Some(crate::types::TypeDef::Enum(e)) => e.type_params.len(),
        _ => 0,
    };
    let args = (0..arity).map(|_| fresh.fresh()).collect();
    MatchShape::Enum(enum_path, args)
}

/// Scan the match arms to decide which shape the scrutinee matches.
/// First arm with a recognized variant-constructor pattern wins:
/// - `:None` or `(Some _)` → Option<T>
/// - `(Ok _)` or `(Err _)` → Result<T,E>
/// - `:enum::Variant` (unit) or `(:enum::Variant ...)` (tagged) → Enum
///   (arc 048). The keyword is split on the last `::` to separate
///   enum path from variant name; the prefix is looked up in the type
///   env to confirm it's a registered enum. Arc 119: parametric enums
///   carry fresh type vars in the MatchShape so unification against
///   the scrutinee's declared parametric type succeeds.
///
/// If no arm is definitive (all wildcards), defaults to Option with
/// a fresh T.
fn detect_match_shape(arms: &[&WatAST], env: &CheckEnv, fresh: &mut InferCtx) -> MatchShape {
    for arm in arms {
        if let WatAST::List(items, _) = arm {
            if items.len() == 2 {
                let pat = &items[0];
                match pat {
                    WatAST::Keyword(k, _) if (k == ":None" || k == ":wat::core::None") => {
                        return MatchShape::Option(fresh.fresh());
                    }
                    WatAST::Keyword(k, _) => {
                        // Arc 048 — user-enum variant pattern (unit
                        // shape). First try the registered unit-variant
                        // map; falling back to enum-prefix lookup so a
                        // misapplied keyword pattern (e.g. tagged-variant
                        // name used in unit position) still classifies
                        // as Enum and produces the right error in
                        // pattern_coverage.
                        if let Some(TypeExpr::Path(enum_path)) = env.unit_variant_type(k) {
                            return enum_match_shape(enum_path.clone(), env, fresh);
                        }
                        if let Some((enum_path, _)) = k.rsplit_once("::") {
                            if matches!(
                                env.types().get(enum_path),
                                Some(crate::types::TypeDef::Enum(_))
                            ) {
                                return enum_match_shape(enum_path.to_string(), env, fresh);
                            }
                        }
                    }
                    WatAST::List(pat_items, _) => {
                        if let Some(WatAST::Symbol(ident, _)) = pat_items.first() {
                            match ident.as_str() {
                                "Some" => return MatchShape::Option(fresh.fresh()),
                                "Ok" | "Err" => {
                                    return MatchShape::Result(fresh.fresh(), fresh.fresh());
                                }
                                _ => {}
                            }
                        }
                        // Arc 109 slice 1h — FQDN keyword forms for
                        // Option variant patterns.
                        // Arc 109 slice 1i — FQDN keyword forms for
                        // Result variant patterns (Ok / Err).
                        if let Some(WatAST::Keyword(k, _)) = pat_items.first() {
                            if k == ":wat::core::Some" {
                                return MatchShape::Option(fresh.fresh());
                            }
                            if k == ":wat::core::Ok" || k == ":wat::core::Err" {
                                return MatchShape::Result(fresh.fresh(), fresh.fresh());
                            }
                        }
                        // Arc 048 — user-enum tagged variant pattern
                        // `(:enum::Variant binders...)`. Split the
                        // head keyword on the last `::` to get the
                        // enum path; if the path resolves to a
                        // declared enum, that's the shape.
                        if let Some(WatAST::Keyword(head_path, _)) = pat_items.first() {
                            if let Some((enum_path, _variant)) = head_path.rsplit_once("::") {
                                let enum_path_owned = enum_path.to_string();
                                if matches!(
                                    env.types().get(&enum_path_owned),
                                    Some(crate::types::TypeDef::Enum(_))
                                ) {
                                    return enum_match_shape(enum_path_owned, env, fresh);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    MatchShape::Option(fresh.fresh())
}

/// Validate `pattern` against the match shape, push bindings into
/// `bindings`, and report its coverage class.
fn pattern_coverage(
    pattern: &WatAST,
    shape: &MatchShape,
    env: &CheckEnv,
    bindings: &mut HashMap<String, TypeExpr>,
    errors: &mut Vec<CheckError>,
) -> Option<Coverage> {
    match pattern {
        WatAST::Keyword(k, _) if (k == ":None" || k == ":wat::core::None") => match shape {
            MatchShape::Option(_) => Some(Coverage::OptionNone),
            MatchShape::Result(_, _) | MatchShape::Enum(_, _) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        ":None pattern on a {} scrutinee",
                        format_type(&shape.as_type())
                    ),
                    span: pattern.span().clone(),
                });
                None
            }
        },
        // Arc 048 — user-enum unit variant pattern. The keyword
        // path must split as `<enum>::<Variant>` where `<enum>`
        // matches the scrutinee shape's Enum path AND `<Variant>`
        // is a unit variant of that enum.
        WatAST::Keyword(k, _) => match shape {
            MatchShape::Enum(enum_path, _) => {
                let (prefix, variant_name) = match k.rsplit_once("::") {
                    Some(p) => p,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "keyword pattern {} must be `<enum>::<Variant>`",
                                k
                            ),
                            span: pattern.span().clone(),
                        });
                        return None;
                    }
                };
                if prefix != enum_path {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "variant pattern {} doesn't belong to scrutinee enum {}",
                            k, enum_path
                        ),
                        span: pattern.span().clone(),
                    });
                    return None;
                }
                // Verify Variant is declared (and is a unit variant).
                if let Some(crate::types::TypeDef::Enum(e)) = env.types().get(enum_path) {
                    let is_unit = e.variants.iter().any(|v| {
                        matches!(v, crate::types::EnumVariant::Unit(n) if n == variant_name)
                    });
                    let is_tagged = e.variants.iter().any(|v| {
                        matches!(v, crate::types::EnumVariant::Tagged { name, .. } if name == variant_name)
                    });
                    if !is_unit && is_tagged {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "{} is a tagged variant; pattern must be (`{}` binders...)",
                                k, k
                            ),
                            span: pattern.span().clone(),
                        });
                        return None;
                    }
                    if !is_unit {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "variant {} is not declared on enum {}",
                                variant_name, enum_path
                            ),
                            span: pattern.span().clone(),
                        });
                        return None;
                    }
                    // Unit variant — no fields, vacuously fully general.
                    Some(Coverage::EnumVariant {
                        name: variant_name.to_string(),
                        full: true,
                    })
                } else {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!("enum {} not declared", enum_path),
                        span: pattern.span().clone(),
                    });
                    None
                }
            }
            _ => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "keyword pattern {} not valid on a {} scrutinee",
                        k,
                        format_type(&shape.as_type())
                    ),
                    span: pattern.span().clone(),
                });
                None
            }
        },
        WatAST::Symbol(ident, _) if ident.as_str() == "_" => Some(Coverage::Wildcard),
        WatAST::Symbol(ident, _) => {
            // Bare name binds the whole scrutinee.
            bindings.insert(ident.as_str().to_string(), shape.as_type());
            Some(Coverage::Wildcard)
        }
        WatAST::List(items, _) => {
            let (head, rest) = match items.split_first() {
                Some(pair) => pair,
                None => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: "empty list pattern".into(),
                        span: pattern.span().clone(),
                    });
                    return None;
                }
            };
            // Arc 048 — user-enum tagged variant pattern: head is a
            // keyword path `:enum::Variant`. Split, validate, bind
            // fields by position.
            //
            // Arc 109 slice 1h — but FQDN built-in variant constructors
            // (`:wat::core::Some`, `:wat::core::Ok`, `:wat::core::Err`)
            // are NOT user enums; let them fall through to the
            // built-in dispatch below (line ~2620).
            if let WatAST::Keyword(variant_path, _) = head {
                let is_builtin_fqdn = variant_path == ":wat::core::Some"
                    || variant_path == ":wat::core::Ok"
                    || variant_path == ":wat::core::Err";
                if !is_builtin_fqdn {
                let enum_path = match shape {
                    MatchShape::Enum(p, _) => p,
                    _ => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "keyword variant pattern {} on a {} scrutinee",
                                variant_path,
                                format_type(&shape.as_type())
                            ),
                            span: pattern.span().clone(),
                        });
                        return None;
                    }
                };
                let (prefix, variant_name) = match variant_path.rsplit_once("::") {
                    Some(p) => p,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "variant constructor pattern {} must be `<enum>::<Variant>`",
                                variant_path
                            ),
                            span: pattern.span().clone(),
                        });
                        return None;
                    }
                };
                if prefix != enum_path {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "variant constructor {} doesn't belong to scrutinee enum {}",
                            variant_path, enum_path
                        ),
                        span: pattern.span().clone(),
                    });
                    return None;
                }
                let enum_def = match env.types().get(enum_path) {
                    Some(crate::types::TypeDef::Enum(e)) => e,
                    _ => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!("enum {} not declared", enum_path),
                            span: pattern.span().clone(),
                        });
                        return None;
                    }
                };
                let fields = enum_def.variants.iter().find_map(|v| {
                    if let crate::types::EnumVariant::Tagged { name, fields } = v {
                        if name == variant_name {
                            return Some(fields);
                        }
                    }
                    None
                });
                let fields = match fields {
                    Some(f) => f,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "{} is not a tagged variant of {}",
                                variant_path, enum_path
                            ),
                            span: pattern.span().clone(),
                        });
                        return None;
                    }
                };
                if rest.len() != fields.len() {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "({} ...) takes {} field(s), got {} binder(s)",
                            variant_path,
                            fields.len(),
                            rest.len()
                        ),
                        span: pattern.span().clone(),
                    });
                    return None;
                }
                // Arc 055 — recurse into each field's sub-pattern.
                let mut all_full = true;
                for (binder_ast, (_field_name, field_type)) in rest.iter().zip(fields.iter()) {
                    match check_subpattern(binder_ast, field_type, env, bindings, errors) {
                        Some(full) => all_full &= full,
                        None => return None,
                    }
                }
                return Some(Coverage::EnumVariant {
                    name: variant_name.to_string(),
                    full: all_full,
                });
                } // close `if !is_builtin_fqdn`
            }
            // Arc 109 slice 1h — list-pattern head accepts both
            // bare-Symbol (legacy grammar exception) and FQDN-keyword
            // (canonical) forms for variant constructors. Map FQDN
            // keywords to the bare ident strings so the downstream
            // dispatch table works unchanged.
            let ident = match head {
                WatAST::Symbol(i, _) => i.as_str(),
                WatAST::Keyword(k, _) if k == ":wat::core::Some" => "Some",
                WatAST::Keyword(k, _) if k == ":wat::core::Ok" => "Ok",
                WatAST::Keyword(k, _) if k == ":wat::core::Err" => "Err",
                other => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "list pattern head must be a variant constructor; got {}",
                            ast_variant_name_check(other)
                        ),
                        span: other.span().clone(),
                    });
                    return None;
                }
            };
            // Arc 055 — variant arm dispatches on shape, then recurses
            // into the inner sub-pattern via `check_subpattern`. The
            // returned `full` flag tracks whether the sub-pattern is
            // fully general (bare symbol or `_` recursively); narrowing
            // sub-patterns produce `full: false` and require a fallback.
            let (ctor_name, mk_coverage, expected_bind_ty): (
                &str,
                fn(bool) -> Coverage,
                TypeExpr,
            ) = match (ident, shape) {
                ("Some", MatchShape::Option(t)) => (
                    "Some",
                    |full| Coverage::OptionSome { full },
                    t.clone(),
                ),
                ("Ok", MatchShape::Result(t, _)) => (
                    "Ok",
                    |full| Coverage::ResultOk { full },
                    t.clone(),
                ),
                ("Err", MatchShape::Result(_, e)) => (
                    "Err",
                    |full| Coverage::ResultErr { full },
                    e.clone(),
                ),
                (other, _) => {
                    // Arc 105 follow-up — when the bare-symbol head
                    // (`other`) names an actual variant of the
                    // scrutinee's user enum, hint at the keyword
                    // form. Bare-symbol heads are reserved for
                    // built-in `Some` / `Ok` / `Err`; user enums
                    // require the namespaced keyword path. The pre-
                    // hint message correctly identifies the failure
                    // but doesn't tell the user how to fix it.
                    let reason = if let MatchShape::Enum(enum_path, _) = shape {
                        let is_variant = matches!(
                            env.types().get(enum_path.as_str()),
                            Some(crate::types::TypeDef::Enum(e))
                                if e.variants.iter().any(|v| match v {
                                    crate::types::EnumVariant::Tagged { name, .. } => name == other,
                                    crate::types::EnumVariant::Unit(name) => name == other,
                                })
                        );
                        if is_variant {
                            format!(
                                "match arm pattern `({} ...)` uses a bare-symbol head, \
                                 but `{}` is a variant of user enum `{}`. Bare-symbol \
                                 heads are reserved for built-in `Some` / `Ok` / `Err`; \
                                 user-enum variants must use the keyword form: write \
                                 `({}::{} ...)` instead.",
                                other, other, enum_path, enum_path, other
                            )
                        } else {
                            format!(
                                "variant constructor `{}` does not match scrutinee shape ({})",
                                other,
                                format_type(&shape.as_type())
                            )
                        }
                    } else {
                        format!(
                            "variant constructor `{}` does not match scrutinee shape ({})",
                            other,
                            format_type(&shape.as_type())
                        )
                    };
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason,
                        span: pattern.span().clone(),
                    });
                    return None;
                }
            };
            if rest.len() != 1 {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "({} _) takes exactly one field, got {}",
                        ctor_name,
                        rest.len()
                    ),
                    span: pattern.span().clone(),
                });
                return None;
            }
            check_subpattern(&rest[0], &expected_bind_ty, env, bindings, errors)
                .map(mk_coverage)
        }
        other => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::match".into(),
                reason: format!(
                    "pattern must be keyword, symbol, or list; got {}",
                    ast_variant_name_check(other)
                ),
                span: other.span().clone(),
            });
            None
        }
    }
}

/// Arc 055 — recursive sub-pattern checker.
///
/// Validates a sub-pattern (anywhere inside a variant or tuple) against
/// the type expected at that position. Populates `bindings` with any
/// bare-symbol binders introduced. Returns `Some(full)` on success
/// (where `full` indicates the sub-pattern is bare-symbol-or-wildcard
/// at every level — a fully-general match), `None` on type/shape
/// mismatch (with errors pushed).
///
/// Disambiguation at list-position is by `expected_ty`:
/// - `Option<U>`: list head Symbol "Some" is the variant constructor.
/// - `Result<T,E>`: list head Symbol "Ok" / "Err" are constructors.
/// - Enum: list head Keyword `:enum::Variant` is the constructor.
/// - Tuple `(T1,...,Tn)`: list is positional destructure; recurse on
///   each element type. The head can be any sub-pattern (bare symbol,
///   variant, literal, nested tuple) — no special "constructor" status.
///
/// `full` is conservative: any literal, variant constructor, or
/// keyword-narrowed pattern at any depth makes the result `false`. The
/// v1 exhaustiveness rule then demands a fallback wildcard arm at the
/// top level. A more sophisticated literal-narrowing analyzer can ship
/// later without changing this helper's contract.
fn check_subpattern(
    pat: &WatAST,
    expected_ty: &TypeExpr,
    env: &CheckEnv,
    bindings: &mut HashMap<String, TypeExpr>,
    errors: &mut Vec<CheckError>,
) -> Option<bool> {
    match pat {
        // Wildcard — fully general.
        WatAST::Symbol(s, _) if s.as_str() == "_" => Some(true),
        // Bare binder — fully general; binds the matched value.
        WatAST::Symbol(s, _) => {
            bindings.insert(s.as_str().to_string(), expected_ty.clone());
            Some(true)
        }
        // Literal sub-patterns — narrow the variant's space; partial.
        WatAST::IntLit(_, _) => match expected_ty {
            TypeExpr::Path(p) if p == ":i64" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "int literal pattern in {} position",
                        format_type(other)
                    ),
                    span: pat.span().clone(),
                });
                None
            }
        },
        WatAST::FloatLit(_, _) => match expected_ty {
            TypeExpr::Path(p) if p == ":f64" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "float literal pattern in {} position",
                        format_type(other)
                    ),
                    span: pat.span().clone(),
                });
                None
            }
        },
        WatAST::BoolLit(_, _) => match expected_ty {
            TypeExpr::Path(p) if p == ":bool" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "bool literal pattern in {} position",
                        format_type(other)
                    ),
                    span: pat.span().clone(),
                });
                None
            }
        },
        WatAST::StringLit(_, _) => match expected_ty {
            TypeExpr::Path(p) if p == ":String" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "string literal pattern in {} position",
                        format_type(other)
                    ),
                    span: pat.span().clone(),
                });
                None
            }
        },
        // Keyword sub-patterns:
        // - `:None` — only valid at Option<U> position; partial (only None).
        // - `:enum::Variant` (unit) — valid at enum position.
        // - bare keyword payload (rare in pattern position) — error.
        WatAST::Keyword(k, _) if (k == ":None" || k == ":wat::core::None") => match expected_ty {
            TypeExpr::Parametric { head, .. } if head == "Option" => Some(false),
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        ":None pattern in {} position",
                        format_type(other)
                    ),
                    span: pat.span().clone(),
                });
                None
            }
        },
        WatAST::Keyword(k, _) => {
            // User-enum unit variant pattern: `:enum::Variant` against
            // the matching enum type at this position.
            let (prefix, variant_name) = match k.rsplit_once("::") {
                Some(p) => p,
                None => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "keyword sub-pattern {} must be `<enum>::<Variant>` or `:None`",
                            k
                        ),
                        span: pat.span().clone(),
                    });
                    return None;
                }
            };
            let enum_path = match expected_ty {
                TypeExpr::Path(p) => p.as_str(),
                other => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "keyword variant pattern {} in {} position",
                            k,
                            format_type(other)
                        ),
                        span: pat.span().clone(),
                    });
                    return None;
                }
            };
            if prefix != enum_path {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!(
                        "variant pattern {} doesn't belong to expected enum {}",
                        k, enum_path
                    ),
                    span: pat.span().clone(),
                });
                return None;
            }
            if let Some(crate::types::TypeDef::Enum(e)) = env.types().get(enum_path) {
                let is_unit = e.variants.iter().any(|v| {
                    matches!(v, crate::types::EnumVariant::Unit(n) if n == variant_name)
                });
                if !is_unit {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "{} is not a unit variant of {} (use `({} ...)` for tagged variants)",
                            k, enum_path, k
                        ),
                        span: pat.span().clone(),
                    });
                    return None;
                }
                Some(false)
            } else {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::match".into(),
                    reason: format!("enum {} not declared", enum_path),
                    span: pat.span().clone(),
                });
                None
            }
        }
        WatAST::List(items, _) => {
            let head = match items.first() {
                Some(h) => h,
                None => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: "empty list sub-pattern".into(),
                        span: pat.span().clone(),
                    });
                    return None;
                }
            };
            // Variant-constructor list at this sub-position:
            // dispatch on expected_ty's shape.
            // Built-in Some/Ok/Err — head is Symbol (legacy bare) OR
            // FQDN keyword (arc 109 slice 1h+1i canonical form).
            let builtin_ident = match head {
                WatAST::Symbol(ident, _) => Some(ident.as_str()),
                WatAST::Keyword(k, _) if k == ":wat::core::Some" => Some("Some"),
                WatAST::Keyword(k, _) if k == ":wat::core::Ok" => Some("Ok"),
                WatAST::Keyword(k, _) if k == ":wat::core::Err" => Some("Err"),
                _ => None,
            };
            if let Some(ident) = builtin_ident {
                match (ident, expected_ty) {
                    ("Some", TypeExpr::Parametric { head: h, args })
                        if h == "Option" && args.len() == 1 =>
                    {
                        if items.len() != 2 {
                            errors.push(CheckError::MalformedForm {
                                head: ":wat::core::match".into(),
                                reason: format!(
                                    "(Some _) takes exactly one field, got {}",
                                    items.len() - 1
                                ),
                                span: pat.span().clone(),
                            });
                            return None;
                        }
                        let _inner_full =
                            check_subpattern(&items[1], &args[0], env, bindings, errors)?;
                        return Some(false);
                    }
                    ("Ok", TypeExpr::Parametric { head: h, args })
                        if h == "Result" && args.len() == 2 =>
                    {
                        if items.len() != 2 {
                            errors.push(CheckError::MalformedForm {
                                head: ":wat::core::match".into(),
                                reason: format!(
                                    "(Ok _) takes exactly one field, got {}",
                                    items.len() - 1
                                ),
                                span: pat.span().clone(),
                            });
                            return None;
                        }
                        let _inner_full =
                            check_subpattern(&items[1], &args[0], env, bindings, errors)?;
                        return Some(false);
                    }
                    ("Err", TypeExpr::Parametric { head: h, args })
                        if h == "Result" && args.len() == 2 =>
                    {
                        if items.len() != 2 {
                            errors.push(CheckError::MalformedForm {
                                head: ":wat::core::match".into(),
                                reason: format!(
                                    "(Err _) takes exactly one field, got {}",
                                    items.len() - 1
                                ),
                                span: pat.span().clone(),
                            });
                            return None;
                        }
                        let _inner_full =
                            check_subpattern(&items[1], &args[1], env, bindings, errors)?;
                        return Some(false);
                    }
                    _ => {
                        // Fall through to tuple destructure below.
                    }
                }
            }
            // User-enum tagged variant: head is Keyword `:enum::Variant`.
            // Arc 109 slice 1h — FQDN built-in variant constructors
            // (`:wat::core::Some`, `:wat::core::Ok`, `:wat::core::Err`)
            // are handled by the built-in dispatch above; if we got here
            // with an unmatched expected_ty, fall through to tuple/error
            // path so the diagnostic surfaces cleanly as "(Some _) takes
            // exactly one field" or similar, not a spurious "user enum"
            // mismatch.
            if let WatAST::Keyword(variant_path, _) = head {
                let is_builtin_fqdn = variant_path == ":wat::core::Some"
                    || variant_path == ":wat::core::Ok"
                    || variant_path == ":wat::core::Err";
                if is_builtin_fqdn {
                    // Built-in already dispatched above; if we reach
                    // here, the `expected_ty` didn't match the
                    // constructor's shape. Surface a precise mismatch.
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "{} pattern in {} position",
                            variant_path,
                            format_type(expected_ty)
                        ),
                        span: pat.span().clone(),
                    });
                    return None;
                }
                let enum_path = match expected_ty {
                    TypeExpr::Path(p) => p.as_str(),
                    other => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "keyword variant pattern {} in {} position",
                                variant_path,
                                format_type(other)
                            ),
                            span: pat.span().clone(),
                        });
                        return None;
                    }
                };
                let (prefix, variant_name) = match variant_path.rsplit_once("::") {
                    Some(p) => p,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "variant constructor pattern {} must be `<enum>::<Variant>`",
                                variant_path
                            ),
                            span: pat.span().clone(),
                        });
                        return None;
                    }
                };
                if prefix != enum_path {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "variant constructor {} doesn't belong to expected enum {}",
                            variant_path, enum_path
                        ),
                        span: pat.span().clone(),
                    });
                    return None;
                }
                let enum_def = match env.types().get(enum_path) {
                    Some(crate::types::TypeDef::Enum(e)) => e,
                    _ => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!("enum {} not declared", enum_path),
                            span: pat.span().clone(),
                        });
                        return None;
                    }
                };
                let fields = enum_def.variants.iter().find_map(|v| {
                    if let crate::types::EnumVariant::Tagged { name, fields } = v {
                        if name == variant_name {
                            return Some(fields);
                        }
                    }
                    None
                });
                let fields = match fields {
                    Some(f) => f,
                    None => {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "{} is not a tagged variant of {}",
                                variant_path, enum_path
                            ),
                            span: pat.span().clone(),
                        });
                        return None;
                    }
                };
                let rest = &items[1..];
                if rest.len() != fields.len() {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "({} ...) takes {} field(s), got {}",
                            variant_path,
                            fields.len(),
                            rest.len()
                        ),
                        span: pat.span().clone(),
                    });
                    return None;
                }
                for (sub_pat, (_field_name, field_type)) in rest.iter().zip(fields.iter()) {
                    check_subpattern(sub_pat, field_type, env, bindings, errors)?;
                }
                return Some(false);
            }
            // Tuple destructure: expected_ty must be a tuple of matching arity.
            match expected_ty {
                TypeExpr::Tuple(elem_tys) => {
                    if items.len() != elem_tys.len() {
                        errors.push(CheckError::MalformedForm {
                            head: ":wat::core::match".into(),
                            reason: format!(
                                "tuple pattern arity {} mismatched with type arity {}",
                                items.len(),
                                elem_tys.len()
                            ),
                            span: pat.span().clone(),
                        });
                        return None;
                    }
                    let mut all_full = true;
                    for (sub_pat, sub_ty) in items.iter().zip(elem_tys.iter()) {
                        match check_subpattern(sub_pat, sub_ty, env, bindings, errors) {
                            Some(full) => all_full &= full,
                            None => return None,
                        }
                    }
                    Some(all_full)
                }
                other => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::match".into(),
                        reason: format!(
                            "list sub-pattern in {} position (expected tuple, Option, Result, or enum)",
                            format_type(other)
                        ),
                        span: pat.span().clone(),
                    });
                    None
                }
            }
        }
    }
}

fn ast_variant_name_check(ast: &WatAST) -> &'static str {
    match ast {
        WatAST::IntLit(_, _) => "int",
        WatAST::FloatLit(_, _) => "float",
        WatAST::BoolLit(_, _) => "bool",
        WatAST::StringLit(_, _) => "string",
        WatAST::Keyword(_, _) => "keyword",
        WatAST::Symbol(_, _) => "symbol",
        WatAST::List(_, _) => "list",
    }
}

/// `(:wat::core::if cond -> :T then else)` — typed conditional per
/// the 2026-04-20 INSCRIPTION.
///
/// Arity: 5 args exactly. Positions: [cond, `->`, `:T`, then, else].
/// The declared `:T` is the expected type for BOTH branches; each
/// branch body is checked against it independently so the error
/// message names WHICH branch diverged (rather than "branches
/// didn't unify" which doesn't name the author's intent).
///
/// The old 3-arg form is refused with a migration-hint MalformedForm
/// at resolve time via the runtime's eval_if; by the time we reach
/// infer_if with the wrong arity, we emit MalformedForm and bail.
fn infer_if(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() == 3 {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::if".into(),
            reason: "`:wat::core::if` now requires `-> :T` between cond and then-branch; write (:wat::core::if cond -> :T then else)".into(),
            span: head_span.clone(),
        });
        // Still recurse into the body so inner errors surface too.
        let _ = infer(&args[0], env, locals, fresh, subst, errors);
        let _ = infer(&args[1], env, locals, fresh, subst, errors);
        let _ = infer(&args[2], env, locals, fresh, subst, errors);
        return None;
    }
    if args.len() != 5 {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::if".into(),
            reason: format!(
                "expected (:wat::core::if cond -> :T then else) — 5 args; got {}",
                args.len()
            ),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    // Validate the `->` marker and parse the declared type.
    match &args[1] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::if".into(),
                reason: "expected `->` between cond and type".into(),
                span: args[1].span().clone(),
            });
            return None;
        }
    }
    let declared_ty = match &args[2] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::if".into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e),
                    span: args[2].span().clone(),
                });
                return None;
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::if".into(),
                reason: "expected type keyword after `->`".into(),
                span: args[2].span().clone(),
            });
            return None;
        }
    };
    // Condition must be :bool.
    let cond_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(c) = cond_ty {
        if unify(&c, &TypeExpr::Path(":bool".into()), subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::core::if".into(),
                param: "cond".into(),
                expected: ":bool".into(),
                got: format_type(&apply_subst(&c, subst)),
                span: args[0].span().clone(),
            });
        }
    }
    // Each branch body checked against the declared `:T` independently.
    // Errors name the branch so the author sees where the divergence is.
    let then_ty = infer(&args[3], env, locals, fresh, subst, errors);
    if let Some(t) = then_ty {
        if unify(&t, &declared_ty, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::core::if".into(),
                param: "then-branch".into(),
                expected: format_type(&apply_subst(&declared_ty, subst)),
                got: format_type(&apply_subst(&t, subst)),
                span: args[3].span().clone(),
            });
        }
    }
    let else_ty = infer(&args[4], env, locals, fresh, subst, errors);
    if let Some(e) = else_ty {
        if unify(&e, &declared_ty, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::core::if".into(),
                param: "else-branch".into(),
                expected: format_type(&apply_subst(&declared_ty, subst)),
                got: format_type(&apply_subst(&e, subst)),
                span: args[4].span().clone(),
            });
        }
    }
    Some(apply_subst(&declared_ty, subst))
}

/// `(:wat::core::cond -> :T arm1 arm2 ... (:else default))`.
///
/// Multi-way conditional; sibling of [`infer_if`]. Typed once at the
/// head via `-> :T`; every arm's body type-unifies with `:T`. Each
/// arm is a 2-element list `(test body)`; tests type-unify with
/// `:bool`. The final arm must be `(:else body)` — enforced here
/// and at runtime.
///
/// Per-arm error messages name which arm diverged (arm #N test /
/// arm #N body / :else body), matching `infer_if`'s branch-specific
/// diagnostics.
fn infer_cond(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() < 3 {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::cond".into(),
            reason: format!(
                "expected (:wat::core::cond -> :T (:else body)) — at least 3 args; got {}",
                args.len()
            ),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    match &args[0] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::cond".into(),
                reason: "expected `->` at position 1".into(),
                span: args[0].span().clone(),
            });
            return None;
        }
    }
    let declared_ty = match &args[1] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::cond".into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e),
                    span: args[1].span().clone(),
                });
                return None;
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::cond".into(),
                reason: "expected type keyword at position 2 (after `->`)".into(),
                span: args[1].span().clone(),
            });
            return None;
        }
    };

    let arms = &args[2..];
    // Validate last arm is `:else`. Report once at the checker layer
    // so users get the diagnostic before the runtime sees it.
    let last = &arms[arms.len() - 1];
    let last_items = match last {
        WatAST::List(xs, _) if xs.len() == 2 => xs,
        WatAST::List(xs, _) => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::cond".into(),
                reason: format!(
                    "last arm must be (:else body); got {}-element list",
                    xs.len()
                ),
                span: last.span().clone(),
            });
            return None;
        }
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::cond".into(),
                reason: "last arm must be a list (:else body)".into(),
                span: last.span().clone(),
            });
            return None;
        }
    };
    let last_is_else = matches!(&last_items[0], WatAST::Keyword(k, _) if k == ":else");
    if !last_is_else {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::cond".into(),
            reason: "last arm must be (:else body) — cond requires an explicit default".into(),
            span: last.span().clone(),
        });
    }

    for (i, arm) in arms.iter().enumerate() {
        let items = match arm {
            WatAST::List(xs, _) if xs.len() == 2 => xs,
            WatAST::List(xs, _) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::cond".into(),
                    reason: format!(
                        "arm #{} must be (test body); got {}-element list",
                        i + 1,
                        xs.len()
                    ),
                    span: arm.span().clone(),
                });
                continue;
            }
            other => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::cond".into(),
                    reason: format!(
                        "arm #{} must be a list (test body); got {:?}",
                        i + 1,
                        other
                    ),
                    span: other.span().clone(),
                });
                continue;
            }
        };
        let is_last = i + 1 == arms.len();
        let is_else_arm =
            is_last && matches!(&items[0], WatAST::Keyword(k, _) if k == ":else");

        if !is_else_arm {
            // Test must unify with :bool.
            let test_ty = infer(&items[0], env, locals, fresh, subst, errors);
            if let Some(t) = test_ty {
                if unify(&t, &TypeExpr::Path(":bool".into()), subst, env.types()).is_err() {
                    errors.push(CheckError::TypeMismatch {
                        callee: ":wat::core::cond".into(),
                        param: format!("arm #{} test", i + 1),
                        expected: ":bool".into(),
                        got: format_type(&apply_subst(&t, subst)),
                        span: items[0].span().clone(),
                    });
                }
            }
        }
        // Body must unify with declared_ty.
        let body_ty = infer(&items[1], env, locals, fresh, subst, errors);
        if let Some(b) = body_ty {
            if unify(&b, &declared_ty, subst, env.types()).is_err() {
                let param = if is_else_arm {
                    ":else body".to_string()
                } else {
                    format!("arm #{} body", i + 1)
                };
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::cond".into(),
                    param,
                    expected: format_type(&apply_subst(&declared_ty, subst)),
                    got: format_type(&apply_subst(&b, subst)),
                    span: items[1].span().clone(),
                });
            }
        }
    }
    Some(apply_subst(&declared_ty, subst))
}

fn infer_let(
    args: &[WatAST],
    _head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        return None;
    }
    let bindings = match &args[0] {
        WatAST::List(items, _) => items,
        _ => return None,
    };
    // Each binding is either typed-single `((name :Type) rhs)` or
    // untyped destructure `((a b ...) rhs)`. Parallel let — all RHSs
    // see the OUTER locals, not each other.
    let mut extended = locals.clone();
    for pair in bindings {
        process_let_binding(pair, env, locals, &mut extended, fresh, subst, errors, ":wat::core::let");
    }
    infer(&args[1], env, &extended, fresh, subst, errors)
}

/// `(:wat::core::try <result-expr>)` — the error-propagation form.
///
/// Type rules:
/// 1. Exactly one argument. Otherwise `ArityMismatch`.
/// 2. The innermost enclosing function/lambda must declare its return
///    type as `:Result<_, E>`. Otherwise `MalformedForm` — `try` has
///    nowhere to propagate to.
/// 3. The argument's type must unify with `:Result<T, E>` where `E` is
///    the enclosing function's `Err` variant. Mismatched `E` surfaces
///    as `TypeMismatch` (strict equality per the 2026-04-19 stance —
///    no auto-conversion, no From-trait analogue). Polymorphic error
///    handling is expressed via explicit enum-wrap at the boundary.
/// 4. On success, the form's type is `T` — the `Ok`-inner of the
///    argument's `Result`.
///
/// Runtime behavior (see `crate::runtime::eval_try`):
/// - `Ok(v)` → evaluates to `v`.
/// - `Err(e)` → raises `RuntimeError::TryPropagate(e)`; the innermost
///   `apply_function` packages it as the function's own `Err(e)`
///   return value.
fn infer_try(
    callee: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: callee.into(),
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        // Still infer the arg(s) so any internal errors surface.
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }

    // The enclosing function's return type must exist and must itself
    // be `Result<_, E>`. Otherwise `try` has no propagation target.
    let enclosing = match fresh.enclosing_ret().cloned() {
        Some(r) => r,
        None => {
            errors.push(CheckError::MalformedForm {
                head: callee.into(),
                reason: format!(
                    "used outside any function or lambda body; `{}` requires an enclosing Result-returning scope to propagate into",
                    callee
                ),
                span: head_span.clone(),
            });
            let _ = infer(&args[0], env, locals, fresh, subst, errors);
            return None;
        }
    };
    // Reduce so a typealias over Result<T,E> is recognized as
    // Result<T,E> here. (`:my::Res<T> = Result<T,String>` would
    // otherwise be rejected as "not a Result" at this match.)
    let enclosing_reduced = reduce(&enclosing, subst, env.types());
    let enclosing_err_ty = match &enclosing_reduced {
        TypeExpr::Parametric { head, args: type_args }
            if head == "Result" && type_args.len() == 2 =>
        {
            type_args[1].clone()
        }
        _ => {
            errors.push(CheckError::MalformedForm {
                head: callee.into(),
                reason: format!(
                    "enclosing function returns {}; `{}` requires the enclosing function to return :Result<T,E>",
                    format_type(&enclosing),
                    callee
                ),
                span: head_span.clone(),
            });
            let _ = infer(&args[0], env, locals, fresh, subst, errors);
            return None;
        }
    };

    // Argument must unify with Result<fresh_T, enclosing_err_ty>.
    // Building the expected type this way enforces both that the arg
    // is a Result and that its Err variant matches the enclosing
    // function's Err variant in one unification.
    let arg_ty = infer(&args[0], env, locals, fresh, subst, errors)?;
    let fresh_t = fresh.fresh();
    let expected = TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![fresh_t.clone(), enclosing_err_ty],
    };
    if unify(&arg_ty, &expected, subst, env.types()).is_err() {
        errors.push(CheckError::TypeMismatch {
            callee: callee.into(),
            param: "arg".into(),
            expected: format_type(&apply_subst(&expected, subst)),
            got: format_type(&apply_subst(&arg_ty, subst)),
            span: args[0].span().clone(),
        });
        return None;
    }

    // The try expression's type is T — the Ok-inner of the argument's
    // Result, now refined by unification with the enclosing function's
    // shape.
    Some(apply_subst(&fresh_t, subst))
}

/// `(:wat::core::Option/try <option-expr>)` — Arc 109 slice 1j. The
/// Option-side mirror of `Result/try`.
///
/// Type rules:
/// 1. Exactly one argument.
/// 2. The innermost enclosing function/lambda must declare its return
///    type as `:Option<_>`. Otherwise `MalformedForm` — `Option/try`
///    has nowhere to propagate to.
/// 3. The argument's type must unify with `:Option<T>`.
/// 4. On success, the form's type is `T` — the Some-inner of the
///    argument's Option.
///
/// Runtime behavior (see `crate::runtime::eval_option_try`):
/// - `Some(v)` → evaluates to `v`.
/// - `:None` → raises `RuntimeError::OptionPropagate`; the innermost
///   `apply_function` packages it as the function's own
///   `Value::Option(None)` return value.
fn infer_option_try(
    callee: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: callee.into(),
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }

    let enclosing = match fresh.enclosing_ret().cloned() {
        Some(r) => r,
        None => {
            errors.push(CheckError::MalformedForm {
                head: callee.into(),
                reason: format!(
                    "used outside any function or lambda body; `{}` requires an enclosing Option-returning scope to propagate into",
                    callee
                ),
                span: head_span.clone(),
            });
            let _ = infer(&args[0], env, locals, fresh, subst, errors);
            return None;
        }
    };
    let enclosing_reduced = reduce(&enclosing, subst, env.types());
    match &enclosing_reduced {
        TypeExpr::Parametric { head, args: type_args }
            if head == "Option" && type_args.len() == 1 => {}
        _ => {
            errors.push(CheckError::MalformedForm {
                head: callee.into(),
                reason: format!(
                    "enclosing function returns {}; `{}` requires the enclosing function to return :Option<T>",
                    format_type(&enclosing),
                    callee
                ),
                span: head_span.clone(),
            });
            let _ = infer(&args[0], env, locals, fresh, subst, errors);
            return None;
        }
    };

    // Argument must unify with Option<fresh_T>. Unlike Result/try the
    // shape carries no Err variant to reconcile against the enclosing
    // function — :None is the sole propagation payload.
    let arg_ty = infer(&args[0], env, locals, fresh, subst, errors)?;
    let fresh_t = fresh.fresh();
    let expected = TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![fresh_t.clone()],
    };
    if unify(&arg_ty, &expected, subst, env.types()).is_err() {
        errors.push(CheckError::TypeMismatch {
            callee: callee.into(),
            param: "arg".into(),
            expected: format_type(&apply_subst(&expected, subst)),
            got: format_type(&apply_subst(&arg_ty, subst)),
            span: args[0].span().clone(),
        });
        return None;
    }

    Some(apply_subst(&fresh_t, subst))
}

/// `(:wat::core::option::expect -> :T <opt> <msg>)` — the
/// panic-on-:None companion to `:wat::core::try`'s propagation form.
/// Arc 108.
///
/// Argument order parallels `if` / `match` semantically but with the
/// type declared FIRST (before any value producer): in `if`/`match`,
/// the first arg (cond/scrutinee) is a dispatch-determiner that does
/// NOT itself produce the result; `-> :T` lands between the
/// determiner and the value-producing arms. In `expect`, the value
/// expression IS a producer (Some/Ok-arm yields its inner). So the
/// honest position for `-> :T` is HEAD-POSITION — declared before
/// any producer.
///
/// Type rules:
/// 1. Exactly four arguments (`->`, type, opt, msg). Otherwise
///    `MalformedForm` naming the expected shape.
/// 2. `args[0]` is the symbol `->`.
/// 3. `args[1]` is the declared arm-result type `:T`. Parsed via
///    `parse_type_expr`.
/// 4. `args[2]` (the opt expression) must unify with `:Option<T>`.
/// 5. `args[3]` (the msg expression) must unify with `:String`.
///
/// On success the form's type is `T` — the Some-inner refined by
/// unification with the declared arm-result type.
fn infer_option_expect(
    callee: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 4 {
        errors.push(CheckError::MalformedForm {
            head: callee.into(),
            reason: format!(
                "expected ({} -> :T <opt> <msg>) — 4 args; got {}",
                callee,
                args.len()
            ),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    // `->` marker at head position.
    match &args[0] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        _ => {
            errors.push(CheckError::MalformedForm {
                head: callee.into(),
                reason: format!(
                    "expected `->` as the first argument; ({} -> :T <opt> <msg>)",
                    callee
                ),
                span: args[0].span().clone(),
            });
            return None;
        }
    }
    // Declared arm-result type.
    let declared_ty = match &args[1] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                errors.push(CheckError::MalformedForm {
                    head: callee.into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e),
                    span: args[1].span().clone(),
                });
                return None;
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: callee.into(),
                reason: "expected type keyword after `->`".into(),
                span: args[1].span().clone(),
            });
            return None;
        }
    };
    // Opt expression must unify with :Option<T> where T is the
    // declared arm-result type.
    let opt_ty = infer(&args[2], env, locals, fresh, subst, errors)?;
    let expected_opt = TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![declared_ty.clone()],
    };
    if unify(&opt_ty, &expected_opt, subst, env.types()).is_err() {
        errors.push(CheckError::TypeMismatch {
            callee: callee.into(),
            param: "opt".into(),
            expected: format_type(&apply_subst(&expected_opt, subst)),
            got: format_type(&apply_subst(&opt_ty, subst)),
            span: args[2].span().clone(),
        });
        return None;
    }
    // Msg must be :String.
    let msg_ty = infer(&args[3], env, locals, fresh, subst, errors);
    if let Some(m) = msg_ty {
        if unify(&m, &TypeExpr::Path(":String".into()), subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: callee.into(),
                param: "msg".into(),
                expected: ":String".into(),
                got: format_type(&apply_subst(&m, subst)),
                span: args[3].span().clone(),
            });
        }
    }
    Some(apply_subst(&declared_ty, subst))
}

/// `(:wat::core::result::expect -> :T <res> <msg>)` — the panic-on-Err
/// sibling of `option::expect`. Arc 108.
///
/// Same shape as `option::expect` but `args[2]` unifies with
/// `:Result<T, fresh_E>` (Err variant is discarded at runtime; its
/// type is left to inference).
fn infer_result_expect(
    callee: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 4 {
        errors.push(CheckError::MalformedForm {
            head: callee.into(),
            reason: format!(
                "expected ({} -> :T <res> <msg>) — 4 args; got {}",
                callee,
                args.len()
            ),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return None;
    }
    match &args[0] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        _ => {
            errors.push(CheckError::MalformedForm {
                head: callee.into(),
                reason: format!(
                    "expected `->` as the first argument; ({} -> :T <res> <msg>)",
                    callee
                ),
                span: args[0].span().clone(),
            });
            return None;
        }
    }
    let declared_ty = match &args[1] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                errors.push(CheckError::MalformedForm {
                    head: callee.into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e),
                    span: args[1].span().clone(),
                });
                return None;
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: callee.into(),
                reason: "expected type keyword after `->`".into(),
                span: args[1].span().clone(),
            });
            return None;
        }
    };
    let res_ty = infer(&args[2], env, locals, fresh, subst, errors)?;
    let fresh_e = fresh.fresh();
    let expected_res = TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![declared_ty.clone(), fresh_e],
    };
    if unify(&res_ty, &expected_res, subst, env.types()).is_err() {
        errors.push(CheckError::TypeMismatch {
            callee: callee.into(),
            param: "res".into(),
            expected: format_type(&apply_subst(&expected_res, subst)),
            got: format_type(&apply_subst(&res_ty, subst)),
            span: args[2].span().clone(),
        });
        return None;
    }
    let msg_ty = infer(&args[3], env, locals, fresh, subst, errors);
    if let Some(m) = msg_ty {
        if unify(&m, &TypeExpr::Path(":String".into()), subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: callee.into(),
                param: "msg".into(),
                expected: ":String".into(),
                got: format_type(&apply_subst(&m, subst)),
                span: args[3].span().clone(),
            });
        }
    }
    Some(apply_subst(&declared_ty, subst))
}

/// Arc 133 — find the source span of the binding that introduces
/// `name` in a let* binding list. Returns the span of the full
/// `((name ...) rhs)` form on match, or `Span::unknown()` when the
/// name isn't found (shouldn't happen in practice — the name comes
/// from the same binding list).
///
/// Used by `check_let_star_for_scope_deadlock_inferred` to attach the
/// thread-binding's source location to the `ScopeDeadlock` diagnostic.
fn find_binding_span(name: &str, bindings: &[WatAST]) -> Span {
    for binding in bindings {
        let WatAST::List(items, span) = binding else { continue; };
        if items.len() != 2 {
            continue;
        }
        let WatAST::List(parts, _) = &items[0] else { continue; };
        // Check whether any symbol in the pattern matches `name`.
        let found = parts.iter().any(|p| match p {
            WatAST::Symbol(id, _) => id.name == name,
            _ => false,
        });
        if found {
            return span.clone();
        }
    }
    Span::unknown()
}

/// Arc 133 — post-inference scope-deadlock check. Replaces the
/// pre-inference structural walker (`validate_scope_deadlock` /
/// `check_let_star_for_scope_deadlock`) for ALL binding shapes.
///
/// After `process_let_binding` runs for every binding in a `let*`,
/// the `extended` map holds the inferred `TypeExpr` for every bound
/// name — whether it was introduced via a typed-name annotation or
/// via a tuple-destructure pattern. This function reads from that map
/// directly, so the same classification logic that powered the
/// pre-inference walker now fires for BOTH binding shapes uniformly.
///
/// Classification is identical to the structural walker:
///   - `type_is_thread_kind` → Thread binding (join-result target)
///   - `type_contains_sender_kind` → Sender-bearing binding (deadlock anchor)
///
/// Scope: the `extended` map contains names from outer scopes (passed
/// into `infer_let_star` as `locals`) as well as this let*'s own
/// bindings. To detect SIBLING bindings (same let* block), we
/// identify which names came from THIS let*'s binding list: only
/// those names participate in the sibling-deadlock check.
///
/// Body is `args[1]` from `infer_let_star`; we check whether
/// `Thread/join-result thr` appears there.
fn check_let_star_for_scope_deadlock_inferred(
    bindings: &[WatAST],
    body: &WatAST,
    extended: &HashMap<String, TypeExpr>,
    types: &TypeEnv,
    errors: &mut Vec<CheckError>,
) {
    // Collect Thread bindings AND Sender-bearing bindings among
    // names that were bound in THIS let*'s binding list.
    let mut thread_bindings: Vec<(String, Span)> = Vec::new();
    let mut sender_bearing_bindings: Vec<(String, &'static str)> = Vec::new();

    // Enumerate names introduced by this let*'s own bindings
    // (to avoid flagging names from outer scopes that happen to be
    // Sender-bearing — they're already in `extended` but are sibling
    // only in their OWN let*, which will be checked when that let*
    // is processed by `infer_let_star`).
    let binding_names: Vec<String> = bindings
        .iter()
        .flat_map(|b| {
            let WatAST::List(items, _) = b else { return vec![]; };
            if items.len() != 2 { return vec![]; }
            let WatAST::List(parts, _) = &items[0] else { return vec![]; };
            parts.iter().filter_map(|p| match p {
                WatAST::Symbol(id, _) => Some(id.name.clone()),
                _ => None,
            }).collect()
        })
        .collect();

    for name in &binding_names {
        let ty = match extended.get(name) {
            Some(t) => t,
            None => continue,
        };
        if type_is_thread_kind(ty, types) {
            let span = find_binding_span(name, bindings);
            thread_bindings.push((name.clone(), span));
            continue;
        }
        if let Some(kind) = type_contains_sender_kind(ty, types) {
            sender_bearing_bindings.push((name.clone(), kind));
        }
    }

    if thread_bindings.is_empty() || sender_bearing_bindings.is_empty() {
        return;
    }

    // For each Thread binding, check whether `Thread/join-result thr`
    // appears in the body (or in any sibling binding's RHS, mirroring
    // the structural walker's check on `bindings`).
    for (thr_name, thr_span) in &thread_bindings {
        let join_present = contains_join_on_thread(body, thr_name)
            || bindings
                .iter()
                .any(|b| contains_join_on_thread(b, thr_name));
        if !join_present {
            continue;
        }
        // Arc 134 — body-form narrowing. The deadlock shape arc 117/131
        // catches requires the spawned function's body to have a recv
        // call (a recv-loop on a Receiver paired with some sibling
        // Sender). If the spawn-thread argument is an inline lambda
        // and its body contains NO `(:wat::kernel::recv ...)` calls
        // anywhere, no recv-loop can exist; no Sender's lifetime can
        // deadlock the thread. Exempt every Sender for this Thread.
        //
        // This is the second arc 134 narrowing alongside the
        // origin-trace exemption below. Together they cover the two
        // canonical non-deadlocking patterns the OLD pre-arc-133
        // walker accidentally allowed via its `:rust::crossbeam_channel`
        // source-annotation bypass:
        //   1. Thread/input <thr> sibling pattern (origin-trace)
        //   2. parent-allocated channel + thread closure that doesn't
        //      recv (this body-form check)
        //
        // Limitations: we only inspect inline `(:wat::core::lambda
        // ...)` bodies. A spawn-thread call whose first arg is a
        // keyword-path (named function) requires substrate function-
        // body lookup we don't have at this hook; the body-form check
        // skips those cases conservatively (the rule still fires; the
        // origin-trace exemption may still apply). Transitive recv
        // through called functions inside the lambda body is also not
        // analyzed — a body that calls `(my-helper rx)` where
        // my-helper recvs in a loop slips through this check (but is
        // legitimately deadlock-prone — a real false negative we
        // accept for simplicity).
        if spawn_thread_lambda_body_has_no_recv(thr_name, bindings) {
            continue;
        }
        for (sender_name, kind) in &sender_bearing_bindings {
            // Arc 134 — origin-trace narrowing. A Sender whose binding
            // RHS is `(:wat::kernel::Thread/input <X>)` (or its Process
            // sibling) extracts the parent-side end of an internal
            // pipe owned by the Thread/Process struct itself. The
            // pair-Receiver is the spawned function's `in` parameter
            // — lifetime-coupled to the Thread, not parent scope. The
            // Sender's coexistence with any Thread in this let* does
            // not constitute the deadlock shape arc 117/131 catches
            // (parent-allocated channel whose Receiver was passed to a
            // thread's recv-loop). Exempt the pair.
            //
            // Heuristic note: if the spawned function's body has an
            // UNCONDITIONAL recv-loop on its input pipe, parent's
            // Sender alive does keep the channel open and the recv-
            // loop blocks. The exemption trusts the canonical
            // Thread<I,O> / Process<I,O> convention (recv-once or
            // paired-coordination body). Bodies that violate the
            // convention deadlock at runtime; the rule won't catch
            // them.
            if sender_originates_from_thread_pipe(sender_name, bindings) {
                continue;
            }
            errors.push(CheckError::ScopeDeadlock {
                thread_binding: thr_name.clone(),
                offending_binding: sender_name.clone(),
                offending_kind: kind,
                span: thr_span.clone(),
            });
        }
    }
}

/// Arc 134 — does the binding for `sender_name` originate from a
/// Thread/Process input-pipe extractor? Returns true if the binding's
/// RHS is `(:wat::kernel::Thread/input <_>)` or
/// `(:wat::kernel::Process/input <_>)`. Such Senders are the parent-
/// side end of an internal pipe; their pair-Receiver is owned by the
/// Thread/Process struct (the spawned function's `in` parameter), not
/// by parent code.
///
/// Used by `check_let_star_for_scope_deadlock_inferred` to exempt
/// canonical Thread<I,O> / Process<I,O> usage from firing
/// ScopeDeadlock. See the call site for the heuristic note.
fn sender_originates_from_thread_pipe(
    sender_name: &str,
    bindings: &[WatAST],
) -> bool {
    for binding in bindings {
        let WatAST::List(items, _) = binding else { continue; };
        if items.len() != 2 { continue; }
        let WatAST::List(parts, _) = &items[0] else { continue; };
        let has_name = parts.iter().any(|p| {
            matches!(p, WatAST::Symbol(id, _) if id.name == sender_name)
        });
        if !has_name { continue; }
        return rhs_is_thread_input_extractor(&items[1]);
    }
    false
}

/// True iff `rhs` is a call of the form
/// `(:wat::kernel::Thread/input <symbol>)` or
/// `(:wat::kernel::Process/input <symbol>)`. Helper for arc 134's
/// origin-trace narrowing.
fn rhs_is_thread_input_extractor(rhs: &WatAST) -> bool {
    let WatAST::List(call, _) = rhs else { return false; };
    let head_str = match call.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return false,
    };
    matches!(
        head_str,
        ":wat::kernel::Thread/input" | ":wat::kernel::Process/input"
    )
}

/// Arc 134 — body-form narrowing. Find `thr_name`'s binding RHS; if
/// it's a `(:wat::kernel::spawn-thread <fn> ...)` or
/// `(:wat::kernel::spawn-program ...)` / `(:wat::kernel::fork-program ...)`
/// call whose `<fn>` argument is an inline `(:wat::core::lambda ...)`,
/// walk the lambda body looking for any `(:wat::kernel::recv ...)`
/// (or `try-recv` / `select`) call. Returns true ONLY when we
/// affirmatively walk a lambda body and find zero recv calls — the
/// thread cannot have a recv-loop, so no Sender lifetime can deadlock
/// it.
///
/// Returns false when:
///   - `thr_name`'s binding can't be located,
///   - the RHS isn't a recognised spawn primitive,
///   - the spawn argument is a keyword-path (named function — we don't
///     do substrate function-body lookup at this hook),
///   - the lambda body contains at least one recv call.
///
/// The conservative default (false → fire) ensures the rule still
/// catches genuine deadlock-prone shapes when we lack body visibility.
fn spawn_thread_lambda_body_has_no_recv(
    thr_name: &str,
    bindings: &[WatAST],
) -> bool {
    // Find thr_name's binding.
    for binding in bindings {
        let WatAST::List(items, _) = binding else { continue; };
        if items.len() != 2 { continue; }
        let WatAST::List(parts, _) = &items[0] else { continue; };
        let has_name = parts.iter().any(|p| {
            matches!(p, WatAST::Symbol(id, _) if id.name == thr_name)
        });
        if !has_name { continue; }
        return rhs_spawn_lambda_has_no_recv(&items[1]);
    }
    false
}

/// True iff `rhs` is a spawn-thread / spawn-program / fork-program
/// call whose function argument is an inline lambda whose body does
/// not contain any kernel recv call. See
/// `spawn_thread_lambda_body_has_no_recv` for the framing.
fn rhs_spawn_lambda_has_no_recv(rhs: &WatAST) -> bool {
    let WatAST::List(call, _) = rhs else { return false; };
    if call.len() < 2 { return false; }
    let head_str = match &call[0] {
        WatAST::Keyword(k, _) => k.as_str(),
        _ => return false,
    };
    let is_spawn = matches!(
        head_str,
        ":wat::kernel::spawn-thread"
            | ":wat::kernel::spawn-program"
            | ":wat::kernel::fork-program"
    );
    if !is_spawn { return false; }
    let fn_arg = &call[1];
    // Must be an inline lambda — keyword-path arguments require
    // function-body lookup we don't perform here.
    let WatAST::List(lambda_call, _) = fn_arg else { return false; };
    if lambda_call.len() < 3 { return false; }
    let lambda_head = match &lambda_call[0] {
        WatAST::Keyword(k, _) => k.as_str(),
        _ => return false,
    };
    if lambda_head != ":wat::core::lambda" { return false; }
    // Lambda shape: (:wat::core::lambda <param-list> body+)
    // params are at index 1; body forms at index 2..
    let body_forms = &lambda_call[2..];
    !body_forms.iter().any(node_contains_recv)
}

/// Walk `node` looking for any `(:wat::kernel::recv ...)`,
/// `(:wat::kernel::try-recv ...)`, or `(:wat::kernel::select ...)`
/// call. Helper for arc 134's body-form narrowing.
fn node_contains_recv(node: &WatAST) -> bool {
    let WatAST::List(items, _) = node else { return false; };
    if let Some(WatAST::Keyword(k, _)) = items.first() {
        if matches!(
            k.as_str(),
            ":wat::kernel::recv"
                | ":wat::kernel::try-recv"
                | ":wat::kernel::select"
        ) {
            return true;
        }
    }
    items.iter().any(node_contains_recv)
}

/// Sequential let — same binding shapes as parallel `let`, but each
/// RHS is checked with the cumulatively extended locals so later
/// bindings may reference earlier ones.
fn infer_let_star(
    args: &[WatAST],
    _head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        return None;
    }
    let bindings = match &args[0] {
        WatAST::List(items, _) => items,
        _ => return None,
    };
    let mut extended = locals.clone();
    for pair in bindings {
        // let* threads the cumulative extended locals through each RHS.
        // We pass `extended` as BOTH the RHS-inference scope and the
        // mutable target; the parallel variant passes the outer
        // `locals` as the RHS scope.
        let cumulative = extended.clone();
        process_let_binding(pair, env, &cumulative, &mut extended, fresh, subst, errors, ":wat::core::let*");
    }

    // Arc 133 — post-inference scope-deadlock check. Fires for BOTH
    // typed-name bindings (arc 117 shape) and tuple-destructure
    // bindings (arc 133 gap). Runs after all bindings are processed
    // so `extended` holds the fully-inferred type for every bound name.
    // The pre-inference structural walker (`validate_scope_deadlock`)
    // was retired when this path was added — inference is now the
    // single enforcement path, eliminating duplicate-firing.
    check_let_star_for_scope_deadlock_inferred(
        bindings,
        &args[1],
        &extended,
        env.types(),
        errors,
    );

    infer(&args[1], env, &extended, fresh, subst, errors)
}

/// Type-check `(:wat::kernel::spawn <fn> arg1 arg2 ...)`.
/// Variadic in the args (one per function parameter) — rank-1 HM
/// can't express variadic schemes, so spawn is special-cased.
///
/// The first argument may be either of two shapes, mirroring the
/// runtime dispatch (see `eval_kernel_spawn`):
///
/// - A keyword-path literal → the function's declared scheme is
///   looked up in `CheckEnv` and instantiated.
/// - Any expression whose inferred type is `:fn(T1,T2,...)->R` → the
///   parameter types and return type come from the inferred Fn type
///   directly.
///
/// Either way, the remaining args are unified against the parameter
/// types, and the spawn's return is `:ProgramHandle<R>`.
fn infer_spawn(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // Arc 114 — `:wat::kernel::spawn` retired. Emit a synthetic
    // TypeMismatch so the arc 114 migration hint fires at every call
    // site. Continue type-checking the args (so additional unrelated
    // mismatches still surface) but the spawn-call itself is now an
    // error regardless of its arity / arg shapes.
    errors.push(CheckError::TypeMismatch {
        callee: ":wat::kernel::spawn".into(),
        param: "(retired verb)".into(),
        expected: ":wat::kernel::spawn-thread".into(),
        got: ":wat::kernel::spawn".into(),
        span: head_span.clone(),
    });
    if args.is_empty() {
        return Some(TypeExpr::Parametric {
            head: "wat::kernel::ProgramHandle".into(),
            args: vec![fresh.fresh()],
        });
    }
    // Resolve the first arg's signature — keyword path path or
    // infer-and-extract-Fn path.
    let (param_types, ret_type, callee_label) = match &args[0] {
        WatAST::Keyword(fn_path, _) => match env.get(fn_path) {
            Some(scheme) => {
                let (ps, r) = instantiate(&scheme.clone(), fresh);
                (ps, r, format!(":wat::kernel::spawn {}", fn_path))
            }
            None => {
                // Function not registered — may be a primitive / future
                // slice / driver. Produce a ProgramHandle<?> so the call
                // site keeps checking.
                for arg in &args[1..] {
                    let _ = infer(arg, env, locals, fresh, subst, errors);
                }
                return Some(TypeExpr::Parametric {
                    head: "wat::kernel::ProgramHandle".into(),
                    args: vec![fresh.fresh()],
                });
            }
        },
        _ => {
            // Non-keyword: infer as a value, expect `:fn(...)->R`. Use
            // reduce so a typealias over an fn type still matches.
            let inferred = infer(&args[0], env, locals, fresh, subst, errors);
            let surface_ty = match &inferred {
                Some(t) => apply_subst(t, subst),
                None => {
                    return Some(TypeExpr::Parametric {
                        head: "wat::kernel::ProgramHandle".into(),
                        args: vec![fresh.fresh()],
                    });
                }
            };
            let fn_ty = reduce(&surface_ty, subst, env.types());
            match fn_ty {
                TypeExpr::Fn { args: ps, ret } => (ps, *ret, ":wat::kernel::spawn <lambda>".to_string()),
                _ => {
                    errors.push(CheckError::TypeMismatch {
                        callee: ":wat::kernel::spawn".into(),
                        param: "#1".into(),
                        expected: "function keyword path or fn(...) value".into(),
                        got: format_type(&surface_ty),
                        span: args[0].span().clone(),
                    });
                    for arg in &args[1..] {
                        let _ = infer(arg, env, locals, fresh, subst, errors);
                    }
                    return Some(TypeExpr::Parametric {
                        head: "wat::kernel::ProgramHandle".into(),
                        args: vec![fresh.fresh()],
                    });
                }
            }
        }
    };
    let spawn_args = &args[1..];
    if spawn_args.len() != param_types.len() {
        errors.push(CheckError::ArityMismatch {
            callee: callee_label.clone(),
            expected: param_types.len(),
            got: spawn_args.len(),
            span: head_span.clone(),
        });
    }
    for (i, (arg, expected)) in spawn_args.iter().zip(&param_types).enumerate() {
        if let Some(arg_ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&arg_ty, expected, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: callee_label.clone(),
                    param: format!("#{}", i + 1),
                    expected: format_type(&apply_subst(expected, subst)),
                    got: format_type(&apply_subst(&arg_ty, subst)),
                    span: arg.span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "wat::kernel::ProgramHandle".into(),
        args: vec![apply_subst(&ret_type, subst)],
    })
}

/// Type-check `(:wat::core::first xs)` / `second` / `third`.
/// Polymorphic over Vec<T> and tuple — both are index-addressed.
/// Rank-1 HM can't express the union, so this is special-cased:
/// inspect the argument's type after substitution and return the
/// element at `index` from whichever container shape matches.
#[allow(clippy::too_many_arguments)]
fn infer_positional_accessor(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
    op: &str,
    index: usize,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(fresh.fresh());
    }
    let arg_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ty) = arg_ty {
        // Reduce to canonical structural form for the match; keep the
        // surface-name form for error display.
        let reduced = reduce(&ty, subst, env.types());
        match &reduced {
            // Tuple: return element at `index`.
            TypeExpr::Tuple(elements) => {
                if let Some(elem) = elements.get(index) {
                    return Some(apply_subst(elem, subst));
                } else {
                    errors.push(CheckError::TypeMismatch {
                        callee: op.into(),
                        param: "#1".into(),
                        expected: format!("tuple with ≥ {} element(s)", index + 1),
                        got: format_type(&apply_subst(&ty, subst)),
                        span: args[0].span().clone(),
                    });
                    return Some(fresh.fresh());
                }
            }
            // Vec<T>: return Option<T> (arc 047 — empty/short is a
            // runtime fact, signature surfaces it honestly).
            TypeExpr::Parametric { head, args: targs } if head == "Vec" => {
                if let Some(inner) = targs.first() {
                    return Some(TypeExpr::Parametric {
                        head: "Option".into(),
                        args: vec![apply_subst(inner, subst)],
                    });
                } else {
                    return Some(fresh.fresh());
                }
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: op.into(),
                    param: "#1".into(),
                    expected: "tuple or Vec<T>".into(),
                    got: format_type(&apply_subst(&ty, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(fresh.fresh())
}

/// Type-check `(:wat::kernel::drop handle)`. The handle is either a
/// `Sender<T>` or `Receiver<T>` — rank-1 HM can't express a union, so
/// this is special-cased: accept either parametric head, produce `:()`.
fn infer_drop(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::kernel::drop".into(),
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Tuple(vec![]));
    }
    let arg_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ty) = arg_ty {
        // Reduce for the shape match; keep the surface-name form for
        // the error display.
        let reduced = reduce(&ty, subst, env.types());
        let is_channel_handle = matches!(
            &reduced,
            TypeExpr::Parametric { head, .. }
                if head == "rust::crossbeam_channel::Sender"
                    || head == "rust::crossbeam_channel::Receiver"
        );
        if !is_channel_handle {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::kernel::drop".into(),
                param: "#1".into(),
                expected: "rust::crossbeam_channel::Sender<T> | rust::crossbeam_channel::Receiver<T>".into(),
                got: format_type(&apply_subst(&ty, subst)),
                span: args[0].span().clone(),
            });
        }
    }
    Some(TypeExpr::Tuple(vec![]))
}

/// Type-check `(make-bounded-queue :T N)` / `(make-unbounded-queue :T)`.
/// First argument is a type keyword (introspected directly, not
/// inferred as a value); optional second argument is the capacity,
/// which must unify to `:i64`. Return type is
/// `:(Sender<T>, Receiver<T>)`.
///
/// Written as a special form because the `∀T. ...` shape expresses T
/// through a type-keyword argument — the value-level checker can't
/// extract T from `infer(args[0])` the way rank-1 HM would want.
#[allow(clippy::too_many_arguments)]
fn infer_make_queue(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
    form: &str,
    with_capacity: bool,
) -> Option<TypeExpr> {
    let expected_arity = if with_capacity { 2 } else { 1 };
    if args.len() != expected_arity {
        errors.push(CheckError::ArityMismatch {
            callee: form.into(),
            expected: expected_arity,
            got: args.len(),
            span: head_span.clone(),
        });
        // Still recurse into any extra args for nested checks.
        for arg in args.iter().skip(1) {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        // Return a best-effort tuple with a fresh inner so the call
        // site can continue checking.
        let t = fresh.fresh();
        return Some(TypeExpr::Tuple(vec![
            TypeExpr::Parametric {
                head: "rust::crossbeam_channel::Sender".into(),
                args: vec![t.clone()],
            },
            TypeExpr::Parametric {
                head: "rust::crossbeam_channel::Receiver".into(),
                args: vec![t],
            },
        ]));
    }
    // Extract T from the type-keyword argument.
    let t_ty = match &args[0] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: form.into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                    span: args[0].span().clone(),
                });
                fresh.fresh()
            }
        },
        other => {
            errors.push(CheckError::MalformedForm {
                head: form.into(),
                reason: format!(
                    "first argument must be a type keyword; got {}",
                    match other {
                        WatAST::IntLit(_, _) => "int",
                        WatAST::FloatLit(_, _) => "float",
                        WatAST::BoolLit(_, _) => "bool",
                        WatAST::StringLit(_, _) => "string",
                        WatAST::Symbol(_, _) => "symbol",
                        WatAST::List(_, _) => "list",
                        WatAST::Keyword(_, _) => unreachable!(),
                    }
                ),
                span: other.span().clone(),
            });
            fresh.fresh()
        }
    };
    // If bounded, check capacity unifies to :i64.
    if with_capacity {
        let cap_ty = infer(&args[1], env, locals, fresh, subst, errors);
        if let Some(cap_ty) = cap_ty {
            let i64_ty = TypeExpr::Path(":i64".into());
            if unify(&cap_ty, &i64_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: form.into(),
                    param: "capacity".into(),
                    expected: "i64".into(),
                    got: format_type(&apply_subst(&cap_ty, subst)),
                    span: args[1].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Tuple(vec![
        TypeExpr::Parametric {
            head: "rust::crossbeam_channel::Sender".into(),
            args: vec![t_ty.clone()],
        },
        TypeExpr::Parametric {
            head: "rust::crossbeam_channel::Receiver".into(),
            args: vec![t_ty],
        },
    ]))
}

/// Process one binding — single-typed or destructure. Infers the RHS
/// in `rhs_scope` and adds the binding(s) to `out_scope`.
#[allow(clippy::too_many_arguments)]
fn process_let_binding(
    pair: &WatAST,
    env: &CheckEnv,
    rhs_scope: &HashMap<String, TypeExpr>,
    out_scope: &mut HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
    form: &str,
) {
    let kv = match pair {
        WatAST::List(items, _) if items.len() == 2 => items,
        _ => return, // runtime parser surfaces the shape error
    };
    let binder = match &kv[0] {
        WatAST::List(inner, _) => inner,
        _ => return, // bare `(name rhs)` refused at runtime; check silently skips
    };
    let rhs = &kv[1];

    let is_typed_single = binder.len() == 2
        && matches!(&binder[0], WatAST::Symbol(_, _))
        && matches!(&binder[1], WatAST::Keyword(_, _));

    if is_typed_single {
        let name = match &binder[0] {
            WatAST::Symbol(ident, _) => ident.name.clone(),
            _ => return,
        };
        let declared = match &binder[1] {
            WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
                Ok(t) => t,
                Err(e) => {
                    // Arc 115 slice 2 — surface the parse error as a
                    // type-check error instead of silently dropping
                    // it. Pre-arc-115 the substrate accepted a
                    // malformed-but-recognizable type annotation
                    // (e.g., `:Vec<:String>`) silently and let the
                    // user discover it via a downstream "expects X;
                    // got Y" mismatch. Now the parser-level error
                    // (with the new InnerColonInCompoundArg
                    // variant's self-describing message) surfaces
                    // directly at the binding site.
                    errors.push(CheckError::MalformedForm {
                        head: form.into(),
                        reason: format!("binding '{}': {}", name, e),
                        span: binder[1].span().clone(),
                    });
                    return;
                }
            },
            _ => return,
        };
        let rhs_ty = infer(rhs, env, rhs_scope, fresh, subst, errors);
        if let Some(rhs_ty) = rhs_ty {
            if unify(&rhs_ty, &declared, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: form.into(),
                    param: format!("binding '{}'", name),
                    expected: format_type(&apply_subst(&declared, subst)),
                    got: format_type(&apply_subst(&rhs_ty, subst)),
                    span: rhs.span().clone(),
                });
            }
        }
        out_scope.insert(name, declared);
        return;
    }

    // Destructure: each element is a bare symbol. Generate one fresh
    // type variable per name; unify the RHS against a tuple of those
    // vars; bind each name to its (post-substitution) element type.
    let mut names = Vec::with_capacity(binder.len());
    for item in binder {
        match item {
            WatAST::Symbol(ident, _) => names.push(ident.name.clone()),
            _ => return, // runtime parser surfaces the shape error
        }
    }
    if names.is_empty() {
        return;
    }
    let elem_vars: Vec<TypeExpr> = (0..names.len()).map(|_| fresh.fresh()).collect();
    let tuple_ty = TypeExpr::Tuple(elem_vars.clone());
    let rhs_ty = infer(rhs, env, rhs_scope, fresh, subst, errors);
    if let Some(rhs_ty) = rhs_ty {
        if unify(&rhs_ty, &tuple_ty, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: form.into(),
                param: format!("destructure ({})", names.join(" ")),
                expected: format_type(&apply_subst(&tuple_ty, subst)),
                got: format_type(&apply_subst(&rhs_ty, subst)),
                span: rhs.span().clone(),
            });
        }
    }
    for (name, ev) in names.into_iter().zip(elem_vars.into_iter()) {
        out_scope.insert(name, apply_subst(&ev, subst));
    }
}

/// Type-check `(:wat::core::HashSet :T x1 x2 ...)`. First arg is a
/// type keyword; remaining args are elements, each unifying with T.
/// Explicit typing required (matches the vec/list / make-queue
/// resource-constructor discipline — shape never depends on context).
fn infer_hashset_constructor(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::HashSet".into(),
            expected: 1,
            got: 0,
            span: head_span.clone(),
        });
        return Some(TypeExpr::Parametric {
            head: "HashSet".into(),
            args: vec![fresh.fresh()],
        });
    }
    let t_ty = match &args[0] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::HashSet".into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                    span: args[0].span().clone(),
                });
                fresh.fresh()
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::HashSet".into(),
                reason: "first argument must be a type keyword (e.g., :i64)".into(),
                span: args[0].span().clone(),
            });
            fresh.fresh()
        }
    };
    for (i, arg) in args[1..].iter().enumerate() {
        if let Some(ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&ty, &t_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::HashSet".into(),
                    param: format!("element #{}", i + 1),
                    expected: format_type(&apply_subst(&t_ty, subst)),
                    got: format_type(&apply_subst(&ty, subst)),
                    span: arg.span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "HashSet".into(),
        args: vec![apply_subst(&t_ty, subst)],
    })
}

/// Arc 050 — polymorphic comparison/equality inference.
///
/// For `:wat::core::=`, `<`, `>`, `<=`, `>=`. Same-type-for-non-
/// numeric, cross-numeric-promotion-for-(i64,f64) pairs. Always
/// returns `:bool`.
///
/// The runtime path (`eval_compare`, `values_equal` post-arc-050)
/// already handles the cross-numeric case; this checker branch
/// makes the runtime path reachable.
fn infer_polymorphic_compare(
    op: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let bool_ty = TypeExpr::Path(":bool".into());
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(bool_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let b_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let (Some(a), Some(b)) = (a_ty, b_ty) {
        let a_resolved = apply_subst(&a, subst);
        let b_resolved = apply_subst(&b, subst);
        // Numeric cross-type allowed: (i64, f64) and (f64, i64) accepted.
        if is_numeric(&a_resolved) && is_numeric(&b_resolved) {
            return Some(bool_ty);
        }
        // Non-numeric: same-type required (preserves prior
        // ∀T. T → T → :bool semantics for strings, bools, etc.).
        if unify(&a_resolved, &b_resolved, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#2".into(),
                expected: format_type(&apply_subst(&a_resolved, subst)),
                got: format_type(&apply_subst(&b_resolved, subst)),
                span: args[1].span().clone(),
            });
        }
    }
    Some(bool_ty)
}

/// Arc 050 — polymorphic arithmetic inference.
///
/// For `:wat::core::+`, `-`, `*`, `/`. Both args must be numeric
/// (`:i64` or `:f64`). Result type is `:f64` if either is `:f64`,
/// else `:i64`. Mixed inputs promote at runtime (i64 cast to f64).
fn infer_polymorphic_arith(
    op: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let i64_ty = TypeExpr::Path(":i64".into());
    let f64_ty = TypeExpr::Path(":f64".into());
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(f64_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let b_ty = infer(&args[1], env, locals, fresh, subst, errors);
    let a_resolved = a_ty.as_ref().map(|t| apply_subst(t, subst));
    let b_resolved = b_ty.as_ref().map(|t| apply_subst(t, subst));

    // Push diagnostic if either arg is non-numeric.
    if let Some(t) = &a_resolved {
        if !is_numeric(t) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#1".into(),
                expected: ":i64 or :f64".into(),
                got: format_type(t),
                span: args[0].span().clone(),
            });
        }
    }
    if let Some(t) = &b_resolved {
        if !is_numeric(t) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#2".into(),
                expected: ":i64 or :f64".into(),
                got: format_type(t),
                span: args[1].span().clone(),
            });
        }
    }

    match (&a_resolved, &b_resolved) {
        (Some(a), Some(b)) if is_i64(a) && is_i64(b) => Some(i64_ty),
        (Some(a), Some(b)) if is_numeric(a) && is_numeric(b) => Some(f64_ty),
        // Either non-numeric or unknown — fall back to f64 so downstream
        // inference doesn't cascade more errors.
        _ => Some(f64_ty),
    }
}

/// Arc 050 — predicate. Recognizes `:i64` and `:f64` paths.
fn is_numeric(t: &TypeExpr) -> bool {
    matches!(t, TypeExpr::Path(p) if p == ":i64" || p == ":f64")
}

/// Arc 097 slice 2 — polymorphic Instant ± Duration arithmetic.
///
/// Three valid shapes (LHS is always Instant):
///
/// ```text
/// (:wat::time::- Instant Duration) -> Instant
/// (:wat::time::- Instant Instant)  -> Duration
/// (:wat::time::+ Instant Duration) -> Instant
/// ```
///
/// The result type depends on (operator, RHS-variant). LHS-Duration
/// is rejected; we don't ship Duration arithmetic in this slice.
fn infer_polymorphic_time_arith(
    op: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let instant_ty = TypeExpr::Path(":wat::time::Instant".into());
    let duration_ty = TypeExpr::Path(":wat::time::Duration".into());

    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(instant_ty);
    }

    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let b_ty = infer(&args[1], env, locals, fresh, subst, errors);
    let a_resolved = a_ty.as_ref().map(|t| apply_subst(t, subst));
    let b_resolved = b_ty.as_ref().map(|t| apply_subst(t, subst));

    // LHS must be an Instant. Push a diagnostic if not, but continue
    // and pick a fallback result type so downstream inference doesn't
    // cascade.
    let a_is_instant = a_resolved
        .as_ref()
        .map(|t| matches!(t, TypeExpr::Path(p) if p == ":wat::time::Instant"))
        .unwrap_or(false);
    if !a_is_instant {
        if let Some(t) = &a_resolved {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#1".into(),
                expected: ":wat::time::Instant".into(),
                got: format_type(t),
                span: args[0].span().clone(),
            });
        }
    }

    // Dispatch on RHS variant.
    match (op, &b_resolved) {
        (":wat::time::-", Some(b))
            if matches!(b, TypeExpr::Path(p) if p == ":wat::time::Instant") =>
        {
            Some(duration_ty)
        }
        (":wat::time::-", Some(b))
            if matches!(b, TypeExpr::Path(p) if p == ":wat::time::Duration") =>
        {
            Some(instant_ty)
        }
        (":wat::time::+", Some(b))
            if matches!(b, TypeExpr::Path(p) if p == ":wat::time::Duration") =>
        {
            Some(instant_ty)
        }
        // RHS is something else — push a diagnostic, fall back to
        // Instant so downstream callers see a stable type.
        _ => {
            if let Some(t) = &b_resolved {
                let expected = if op == ":wat::time::+" {
                    ":wat::time::Duration"
                } else {
                    ":wat::time::Duration or :wat::time::Instant"
                };
                errors.push(CheckError::TypeMismatch {
                    callee: op.into(),
                    param: "#2".into(),
                    expected: expected.into(),
                    got: format_type(t),
                    span: args[1].span().clone(),
                });
            }
            Some(instant_ty)
        }
    }
}

/// Arc 098 — type-check `:wat::form::matches?`. Substrate-recognized
/// special form (not a user defmacro); macros expand before
/// type-checking and can't query the struct registry, so the matcher
/// has to dispatch directly through `infer_call`.
///
/// Shape:
///
/// ```text
/// (:wat::form::matches? SUBJECT
///   (:TYPE-NAME (= ?var :field) ... <constraint> ...))
/// ```
///
/// The subject's static type is unconstrained — the matcher returns
/// `false` at runtime when the subject is `:None`, non-Struct, or a
/// Struct of the wrong type. We `infer` it anyway so nested errors
/// (e.g. unknown function in a nested call) still surface.
fn infer_form_matches(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let bool_ty = TypeExpr::Path(":bool".into());

    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::form::matches?".into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(bool_ty);
    }

    // Subject — drive nested errors but accept any type.
    let _ = infer(&args[0], env, locals, fresh, subst, errors);

    // Pattern — must be `(:TYPE-NAME clause ...)`.
    let pattern_items = match &args[1] {
        WatAST::List(items, _) if !items.is_empty() => items,
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::form::matches?".into(),
                reason: "pattern must be a list `(:TYPE-NAME clause ...)`".into(),
                span: args[1].span().clone(),
            });
            return Some(bool_ty);
        }
    };
    let type_name = match &pattern_items[0] {
        WatAST::Keyword(k, _) => k.as_str(),
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::form::matches?".into(),
                reason: "pattern head must be a struct type keyword".into(),
                span: pattern_items[0].span().clone(),
            });
            return Some(bool_ty);
        }
    };

    // Resolve struct fields.
    let fields: Vec<(String, TypeExpr)> = match env.types().get(type_name) {
        Some(crate::types::TypeDef::Struct(s)) => s.fields.clone(),
        Some(_) => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::form::matches?".into(),
                reason: format!(
                    "pattern head {} names a non-struct type; matches? walks struct fields",
                    type_name
                ),
                span: pattern_items[0].span().clone(),
            });
            return Some(bool_ty);
        }
        None => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::form::matches?".into(),
                reason: format!("unknown struct type {}", type_name),
                span: pattern_items[0].span().clone(),
            });
            return Some(bool_ty);
        }
    };

    // Walk clauses, threading binding scope through `pattern_locals`.
    // Bindings push `?var → field-type`; constraint sub-clauses
    // (and/or/not) re-use the same scope (sub-clauses cannot
    // introduce new bindings — see DESIGN §what's NOT in this arc).
    let mut pattern_locals = locals.clone();
    for clause in &pattern_items[1..] {
        check_clause(
            clause,
            type_name,
            &fields,
            env,
            &mut pattern_locals,
            fresh,
            subst,
            errors,
        );
    }

    Some(bool_ty)
}

/// Type-check a single clause inside a `:wat::form::matches?`
/// pattern. Mutates `locals` to register fresh bindings; pushes
/// errors for grammar / semantic violations.
#[allow(clippy::too_many_arguments)]
fn check_clause(
    clause: &WatAST,
    type_name: &str,
    fields: &[(String, TypeExpr)],
    env: &CheckEnv,
    locals: &mut HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) {
    use crate::form_match::{
        classify_clause, keyword_payload, logic_var_name, RawClause,
    };

    let raw = match classify_clause(clause) {
        Ok(r) => r,
        Err(e) => {
            errors.push(grammar_error_to_check_error(e, clause.span().clone()));
            return;
        }
    };

    match raw {
        RawClause::Eq { left, right } => {
            // Disambiguate binding vs equality by the LHS shape and
            // whether the variable is already in scope.
            if let Some(var) = logic_var_name(left) {
                if !locals.contains_key(var) {
                    // Fresh ?var — this is a binding. RHS must be a
                    // field keyword that exists on the struct.
                    let field_name = match keyword_payload(right) {
                        Some(k) => k,
                        None => {
                            errors.push(CheckError::MalformedForm {
                                head: ":wat::form::matches?".into(),
                                reason: format!(
                                    "binding RHS for {} must be a field keyword like :field-name",
                                    var
                                ),
                                span: right.span().clone(),
                            });
                            return;
                        }
                    };
                    // Field name is the keyword stripped of leading `:`.
                    let field_lookup = field_name.strip_prefix(':').unwrap_or(field_name);
                    let field_ty = match fields.iter().find(|(n, _)| n == field_lookup) {
                        Some((_, t)) => t.clone(),
                        None => {
                            errors.push(CheckError::MalformedForm {
                                head: ":wat::form::matches?".into(),
                                reason: format!(
                                    "struct {} has no field {}",
                                    type_name, field_name
                                ),
                                span: right.span().clone(),
                            });
                            return;
                        }
                    };
                    locals.insert(var.to_string(), field_ty);
                    return;
                }
                // ?var already bound — fall through to comparison.
            } else {
                // LHS isn't a ?var. Could be a literal-vs-?var
                // comparison (e.g. `(= "Grace" ?outcome)`); both
                // sides type-check below.
            }
            check_comparison(left, right, env, locals, fresh, subst, errors);
        }
        RawClause::Compare { left, right, .. } => {
            check_comparison(left, right, env, locals, fresh, subst, errors);
        }
        RawClause::And(subs) | RawClause::Or(subs) => {
            for sub in subs {
                check_clause(sub, type_name, fields, env, locals, fresh, subst, errors);
            }
        }
        RawClause::Not(sub) => {
            check_clause(sub, type_name, fields, env, locals, fresh, subst, errors);
        }
        RawClause::Where(body) => {
            // `where` body is arbitrary wat in the binding scope;
            // it must type to `:bool`.
            let body_ty = infer(body, env, locals, fresh, subst, errors);
            if let Some(t) = body_ty {
                let bool_ty = TypeExpr::Path(":bool".into());
                if unify(&t, &bool_ty, subst, env.types()).is_err() {
                    errors.push(CheckError::TypeMismatch {
                        callee: ":wat::form::matches?".into(),
                        param: "where-body".into(),
                        expected: format_type(&bool_ty),
                        got: format_type(&apply_subst(&t, subst)),
                        span: body.span().clone(),
                    });
                }
            }
        }
    }
}

/// Type-check the two operands of a comparison clause. Both sides
/// are inferred; if both have known types, they must unify to a
/// common type (matches arc 050 polymorphic-compare semantics).
fn check_comparison(
    left: &WatAST,
    right: &WatAST,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) {
    let l_ty = infer(left, env, locals, fresh, subst, errors);
    let r_ty = infer(right, env, locals, fresh, subst, errors);
    if let (Some(l), Some(r)) = (l_ty, r_ty) {
        if unify(&l, &r, subst, env.types()).is_err() {
            errors.push(CheckError::TypeMismatch {
                callee: ":wat::form::matches?".into(),
                param: "comparison".into(),
                expected: format_type(&apply_subst(&l, subst)),
                got: format_type(&apply_subst(&r, subst)),
                span: right.span().clone(),
            });
        }
    }
}

fn grammar_error_to_check_error(e: crate::form_match::ClauseGrammarError, span: Span) -> CheckError {
    use crate::form_match::ClauseGrammarError as G;
    let reason = match e {
        G::NotAList(_) => "clause must be a list `(head ...)`".to_string(),
        G::EmptyList(_) => "empty clause `()` — clauses need a head".to_string(),
        G::NonKeywordHead(_) => "clause head must be a keyword (=, <, and, where, ...)".to_string(),
        G::UnknownHead(h, _) => format!(
            "unknown matcher head: {}; recognized: =, <, >, <=, >=, not=, and, or, not, where",
            h
        ),
        G::NotArity { got, .. } => format!("`not` takes exactly 1 sub-clause; got {}", got),
        G::WhereArity { got, .. } => format!("`where` takes exactly 1 expression; got {}", got),
        G::BinaryArity { op, got, .. } => format!("`{}` takes exactly 2 args; got {}", op.as_str(), got),
    };
    CheckError::MalformedForm {
        head: ":wat::form::matches?".into(),
        reason,
        span,
    }
}

/// Arc 050 — predicate. Recognizes `:i64` path specifically.
fn is_i64(t: &TypeExpr) -> bool {
    matches!(t, TypeExpr::Path(p) if p == ":i64")
}

/// Arc 052 — predicate. Recognizes `:wat::holon::HolonAST` and
/// `:wat::holon::Vector` — the two algebra-tier value types accepted
/// by polymorphic cosine / dot / simhash.
fn is_holon_or_vector(t: &TypeExpr) -> bool {
    matches!(
        t,
        TypeExpr::Path(p)
            if p == ":wat::holon::HolonAST" || p == ":wat::holon::Vector"
    )
}

/// Arc 052 — polymorphic two-arg holon-algebra inference.
///
/// For `:wat::holon::cosine` and `:wat::holon::dot`. Both args must be
/// HolonAST or Vector; result type is `:f64`. Mixed inputs are
/// permitted (the runtime promotes the AST side by encoding at the
/// Vector side's d).
fn infer_polymorphic_holon_pair_to_f64(
    op: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let f64_ty = TypeExpr::Path(":f64".into());
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(f64_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let b_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(t) = &a_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#1".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
                span: args[0].span().clone(),
            });
        }
    }
    if let Some(t) = &b_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#2".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
                span: args[1].span().clone(),
            });
        }
    }
    Some(f64_ty)
}

/// Arc 061 — polymorphic two-arg holon-algebra inference returning
/// `:bool`. For `:wat::holon::coincident?` — accepts HolonAST or
/// Vector in either position. Mirrors
/// [`infer_polymorphic_holon_pair_to_f64`] exactly; differs only in
/// return type.
fn infer_polymorphic_holon_pair_to_bool(
    op: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let bool_ty = TypeExpr::Path(":bool".into());
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(bool_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let b_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(t) = &a_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#1".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
                span: args[0].span().clone(),
            });
        }
    }
    if let Some(t) = &b_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#2".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
                span: args[1].span().clone(),
            });
        }
    }
    Some(bool_ty)
}

/// Arc 069 — polymorphic two-arg holon-algebra inference returning a
/// fixed struct path. For `:wat::holon::coincident-explain` —
/// returns `:wat::holon::CoincidentExplanation`. Same arg discipline
/// as the bool / f64 siblings; differs only in the return type
/// arity (a registered struct path).
#[allow(clippy::too_many_arguments)]
fn infer_polymorphic_holon_pair_to_path(
    op: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
    return_path: &str,
) -> Option<TypeExpr> {
    let ret_ty = TypeExpr::Path(return_path.into());
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(ret_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let b_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(t) = &a_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#1".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
                span: args[0].span().clone(),
            });
        }
    }
    if let Some(t) = &b_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#2".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
                span: args[1].span().clone(),
            });
        }
    }
    Some(ret_ty)
}

/// Arc 052 — polymorphic one-arg holon-algebra inference returning
/// `:i64`. For `:wat::holon::simhash` — accepts HolonAST or Vector.
fn infer_polymorphic_holon_to_i64(
    op: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let i64_ty = TypeExpr::Path(":i64".into());
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: op.into(),
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        for arg in args {
            let _ = infer(arg, env, locals, fresh, subst, errors);
        }
        return Some(i64_ty);
    }
    let a_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(t) = &a_ty {
        let resolved = apply_subst(t, subst);
        if !is_holon_or_vector(&resolved) {
            errors.push(CheckError::TypeMismatch {
                callee: op.into(),
                param: "#1".into(),
                expected: ":wat::holon::HolonAST or :wat::holon::Vector".into(),
                got: format_type(&resolved),
                span: args[0].span().clone(),
            });
        }
    }
    Some(i64_ty)
}

/// Type-check `(:wat::core::get container locator)`. Polymorphic over
/// HashMap and HashSet; dispatch by arg shape. Rank-1 HM can't
/// express the union at the SCHEME layer, so special-case: inspect
/// the first arg's type and produce the matching return type.
///   HashMap<K,V>, K → Option<V>
///   HashSet<T>,   T → Option<T>
fn infer_get(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::get".into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Parametric {
            head: "Option".into(),
            args: vec![fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let key_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        // Reduce for the shape match — a user typealias over HashMap
        // / HashSet (e.g., `(typealias :my::Row :HashMap<String,i64>)`)
        // must be recognized by its structural root here.
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                let v = apply_subst(&ta[1], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &k, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::get".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&k, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                return Some(TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![apply_subst(&v, subst)],
                });
            }
            // Arc 025: Vec support. `(get xs i)` with :i64 index
            // returns `:Option<T>`. Unify key with i64; container's
            // element type is the Option's T. 058-026 INSCRIPTION.
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    let i64_ty = TypeExpr::Path(":i64".into());
                    if unify(&key_ty, &i64_ty, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::get".into(),
                            param: "key".into(),
                            expected: "i64".into(),
                            got: format_type(&apply_subst(&key_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                return Some(TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![apply_subst(&t, subst)],
                });
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::get".into(),
                            param: "element".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                return Some(TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![apply_subst(&t, subst)],
                });
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::get".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V> | HashSet<T> | Vec<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![fresh.fresh()],
    })
}

/// Arc 020 — `(:wat::core::assoc container key value)`. Clojure
/// `assoc`: associate key with value in a HashMap, return new map.
/// For `HashMap<K,V>`: unifies key-ty with K, value-ty with V;
/// returns the input HashMap type. Matches `infer_get`'s dispatch-
/// on-container shape; extends to other containers if demand
/// surfaces.
fn infer_assoc(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 3 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::assoc".into(),
            expected: 3,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Parametric {
            head: "HashMap".into(),
            args: vec![fresh.fresh(), fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let key_ty = infer(&args[1], env, locals, fresh, subst, errors);
    let value_ty = infer(&args[2], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                let v = apply_subst(&ta[1], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &k, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::assoc".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&k, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                if let Some(value_ty) = value_ty {
                    if unify(&value_ty, &v, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::assoc".into(),
                            param: "value".into(),
                            expected: format_type(&apply_subst(&v, subst)),
                            got: format_type(&apply_subst(&value_ty, subst)),
                            span: args[2].span().clone(),
                        });
                    }
                }
                return Some(reduced);
            }
            // Arc 025: Vec support. `(assoc xs i v)` replaces xs[i]
            // with v; i must unify with :i64, v must unify with T.
            // Returns Vec<T>. Out-of-range i is a runtime error, not
            // a type error.
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    let i64_ty = TypeExpr::Path(":i64".into());
                    if unify(&key_ty, &i64_ty, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::assoc".into(),
                            param: "key".into(),
                            expected: "i64".into(),
                            got: format_type(&apply_subst(&key_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                if let Some(value_ty) = value_ty {
                    if unify(&value_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::assoc".into(),
                            param: "value".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&value_ty, subst)),
                            span: args[2].span().clone(),
                        });
                    }
                }
                return Some(reduced);
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::assoc".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V> | Vec<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "HashMap".into(),
        args: vec![fresh.fresh(), fresh.fresh()],
    })
}

/// Arc 058 — `(:wat::core::dissoc m k)`. Returns a NEW HashMap
/// without `k`; original unchanged. Missing key is no-op
/// (returns clone of input). Mirrors Clojure's dissoc.
///   ∀K, V. HashMap<K,V> × K → HashMap<K,V>
fn infer_dissoc(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::dissoc".into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Parametric {
            head: "HashMap".into(),
            args: vec![fresh.fresh(), fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let key_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &k, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::dissoc".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&k, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                return Some(reduced);
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::dissoc".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "HashMap".into(),
        args: vec![fresh.fresh(), fresh.fresh()],
    })
}

/// Arc 058 — `(:wat::core::keys m)`. Materializes the map's keys
/// as a Vec (order unspecified — Rust's HashMap iteration order
/// depends on hash randomization; sort the result if you need
/// determinism).
///   ∀K, V. HashMap<K,V> → Vec<K>
fn infer_keys(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::keys".into(),
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                return Some(TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![k],
                });
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::keys".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![fresh.fresh()],
    })
}

/// Arc 058 — `(:wat::core::values m)`. Materializes the map's
/// values as a Vec (order unspecified — same caveat as `keys`).
///   ∀K, V. HashMap<K,V> → Vec<V>
fn infer_values(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::values".into(),
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let v = apply_subst(&ta[1], subst);
                return Some(TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![v],
                });
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::values".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![fresh.fresh()],
    })
}

/// Arc 058 — `(:wat::core::empty? container)`. Polymorphic empty-check;
/// mirrors `length`'s polymorphism shape:
///   ∀T.   Vec<T>       → bool
///   ∀K,V. HashMap<K,V> → bool
///   ∀T.   HashSet<T>   → bool
fn infer_empty_q(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::empty?".into(),
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Path(":bool".into()));
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let _ = ta;
                return Some(TypeExpr::Path(":bool".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let _ = ta;
                return Some(TypeExpr::Path(":bool".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let _ = ta;
                return Some(TypeExpr::Path(":bool".into()));
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::empty?".into(),
                    param: "container".into(),
                    expected: "Vec<T> | HashMap<K,V> | HashSet<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Path(":bool".into()))
}

/// Arc 025 — `(:wat::core::conj container value)`. Polymorphic
/// over Vec and HashSet; HashMap illegal (no key-value pairing —
/// use assoc).
///   ∀T. Vec<T>     × T -> Vec<T>
///   ∀T. HashSet<T> × T -> HashSet<T>
fn infer_conj(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::conj".into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![fresh.fresh()],
        });
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let value_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(value_ty) = value_ty {
                    if unify(&value_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::conj".into(),
                            param: "value".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&value_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                return Some(reduced);
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(value_ty) = value_ty {
                    if unify(&value_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::conj".into(),
                            param: "value".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&value_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                return Some(reduced);
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::conj".into(),
                    param: "container".into(),
                    expected: "Vec<T> | HashSet<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![fresh.fresh()],
    })
}

/// Arc 035 — `(:wat::core::length container)`. Polymorphic size:
///   ∀T.   Vec<T>       -> i64    (elements)
///   ∀K,V. HashMap<K,V> -> i64    (entries)
///   ∀T.   HashSet<T>   -> i64    (elements)
/// Tuple is deliberately excluded — arity is structural and known
/// at type-check time.
fn infer_length(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 1 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::length".into(),
            expected: 1,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Path(":i64".into()));
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                let _ = ta;
                return Some(TypeExpr::Path(":i64".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let _ = ta;
                return Some(TypeExpr::Path(":i64".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let _ = ta;
                return Some(TypeExpr::Path(":i64".into()));
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::length".into(),
                    param: "container".into(),
                    expected: "Vec<T> | HashMap<K,V> | HashSet<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Path(":i64".into()))
}

/// Arc 025 — `(:wat::core::contains? container key)`. Polymorphic
/// membership/key predicate:
///   ∀K,V. HashMap<K,V> × K -> bool    (has key)
///   ∀T.   HashSet<T>   × T -> bool    (has element)
///   ∀T.   Vec<T>       × i64 -> bool  (has valid index)
/// Retires `:wat::std::member?` — contains? covers it now.
fn infer_contains_q(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::contains?".into(),
            expected: 2,
            got: args.len(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Path(":bool".into()));
    }
    let container_ty = infer(&args[0], env, locals, fresh, subst, errors);
    let key_ty = infer(&args[1], env, locals, fresh, subst, errors);
    if let Some(ct) = container_ty {
        let reduced = reduce(&ct, subst, env.types());
        match &reduced {
            TypeExpr::Parametric { head, args: ta } if head == "HashMap" && ta.len() == 2 => {
                let k = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &k, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::contains?".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&k, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                return Some(TypeExpr::Path(":bool".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "HashSet" && ta.len() == 1 => {
                let t = apply_subst(&ta[0], subst);
                if let Some(key_ty) = key_ty {
                    if unify(&key_ty, &t, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::contains?".into(),
                            param: "key".into(),
                            expected: format_type(&apply_subst(&t, subst)),
                            got: format_type(&apply_subst(&key_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                return Some(TypeExpr::Path(":bool".into()));
            }
            TypeExpr::Parametric { head, args: ta } if head == "Vec" && ta.len() == 1 => {
                if let Some(key_ty) = key_ty {
                    let i64_ty = TypeExpr::Path(":i64".into());
                    if unify(&key_ty, &i64_ty, subst, env.types()).is_err() {
                        errors.push(CheckError::TypeMismatch {
                            callee: ":wat::core::contains?".into(),
                            param: "key".into(),
                            expected: "i64".into(),
                            got: format_type(&apply_subst(&key_ty, subst)),
                            span: args[1].span().clone(),
                        });
                    }
                }
                // suppress unused-arg warnings in this arm
                let _ = ta;
                return Some(TypeExpr::Path(":bool".into()));
            }
            _ => {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::contains?".into(),
                    param: "container".into(),
                    expected: "HashMap<K,V> | HashSet<T> | Vec<T>".into(),
                    got: format_type(&apply_subst(&ct, subst)),
                    span: args[0].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Path(":bool".into()))
}

/// Type-check `(:wat::core::HashMap :(K,V) k1 v1 k2 v2 ...)`. First arg
/// is a tuple-type keyword `:(K,V)` encoding both parameters; the
/// remaining args are alternating key/value pairs. Keys unify with K,
/// values with V. Explicit typing required (matches vec/list / make-queue
/// resource-constructor discipline).
fn infer_hashmap_constructor(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::HashMap".into(),
            expected: 1,
            got: 0,
            span: head_span.clone(),
        });
        return Some(TypeExpr::Parametric {
            head: "HashMap".into(),
            args: vec![fresh.fresh(), fresh.fresh()],
        });
    }
    let (k_ty, v_ty) = match &args[0] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            // Expand typealiases before the Tuple-shape check so users
            // can name `:(K,V)` once at top level and reuse the alias
            // at every HashMap construction site. Mirrors the
            // "aliases resolve structurally at call sites" rule
            // documented in CONVENTIONS.md (e.g. `:wat::core::Bytes ≡
            // :Vec<u8>`). Without this, a tuple-shaped alias parses
            // as `TypeExpr::Path(...)` and fails the Tuple match
            // before ever being unwrapped — even though every other
            // site treats the alias as its expansion.
            Ok(parsed) => match crate::types::expand_alias(&parsed, env.types()) {
                TypeExpr::Tuple(ts) if ts.len() == 2 => (ts[0].clone(), ts[1].clone()),
                other => {
                    errors.push(CheckError::MalformedForm {
                        head: ":wat::core::HashMap".into(),
                        reason: format!(
                            "first argument must be a tuple type :(K,V); got {}",
                            format_type(&other)
                        ),
                        span: args[0].span().clone(),
                    });
                    (fresh.fresh(), fresh.fresh())
                }
            },
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::HashMap".into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                    span: args[0].span().clone(),
                });
                (fresh.fresh(), fresh.fresh())
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::HashMap".into(),
                reason: "first argument must be a tuple type keyword :(K,V)".into(),
                span: args[0].span().clone(),
            });
            (fresh.fresh(), fresh.fresh())
        }
    };
    let pairs = &args[1..];
    if !pairs.len().is_multiple_of(2) {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::HashMap".into(),
            reason: "arity after :(K,V) must be even (alternating key/value)".into(),
            span: head_span.clone(),
        });
    }
    for (i, chunk) in pairs.chunks(2).enumerate() {
        if let Some(k_arg_ty) = infer(&chunk[0], env, locals, fresh, subst, errors) {
            if unify(&k_arg_ty, &k_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::HashMap".into(),
                    param: format!("key #{}", i + 1),
                    expected: format_type(&apply_subst(&k_ty, subst)),
                    got: format_type(&apply_subst(&k_arg_ty, subst)),
                    span: chunk[0].span().clone(),
                });
            }
        }
        if let Some(v_arg_ty) = chunk
            .get(1)
            .and_then(|a| infer(a, env, locals, fresh, subst, errors))
        {
            if unify(&v_arg_ty, &v_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::HashMap".into(),
                    param: format!("value #{}", i + 1),
                    expected: format_type(&apply_subst(&v_ty, subst)),
                    got: format_type(&apply_subst(&v_arg_ty, subst)),
                    span: chunk[1].span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "HashMap".into(),
        args: vec![apply_subst(&k_ty, subst), apply_subst(&v_ty, subst)],
    })
}

/// Type-check `(:wat::core::tuple a b c ...)`. Heterogeneous — each
/// arg contributes its own inferred type, and the return type is the
/// concrete tuple shape. Variadic; rank-1 HM can't express a
/// per-position scheme, so special-cased.
fn infer_tuple_constructor(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::MalformedForm {
            head: ":wat::core::tuple".into(),
            reason: "tuple must have at least one element".into(),
            span: head_span.clone(),
        });
        return Some(TypeExpr::Tuple(vec![fresh.fresh()]));
    }
    let mut elements = Vec::with_capacity(args.len());
    for arg in args {
        let ty = infer(arg, env, locals, fresh, subst, errors).unwrap_or_else(|| fresh.fresh());
        elements.push(apply_subst(&ty, subst));
    }
    Some(TypeExpr::Tuple(elements))
}

/// `(:wat::core::string::concat s1 s2 ... sn) -> :String`.
///
/// Variadic; each arg must unify with :String. Special-cased here
/// rather than registered as a polymorphic scheme because the type
/// checker has no first-class variadic-arity scheme today (same
/// rationale as `vec` / `tuple`). Empty arg list errors at the
/// runtime; the checker accepts arity 0 and returns `:String` so the
/// runtime owns the diagnostic — this mirrors how `tuple` behaves.
/// Arc 059 — `(:wat::core::concat v1 v2 ...)`. Variadic Vec
/// concatenation; ≥1 arg required (zero-arg ambiguous on T, same
/// reasoning as `:wat::core::vec`'s rejection of zero-arg).
///   ∀T. (Vec<T>)+ → Vec<T>
/// All args must unify on the same `Vec<T>` — no implicit coercion
/// (a `Vec<i64>` and `Vec<f64>` don't concat). Mirrors the
/// `string::concat` shape but with a fresh element type variable
/// instead of a fixed `:String`.
fn infer_concat(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::concat".into(),
            expected: 1,
            got: 0,
            span: head_span.clone(),
        });
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![fresh.fresh()],
        });
    }
    let elem_ty = fresh.fresh();
    let vec_ty = TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![elem_ty],
    };
    for arg in args {
        if let Some(ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&ty, &vec_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::concat".into(),
                    param: "arg".into(),
                    expected: format_type(&apply_subst(&vec_ty, subst)),
                    got: format_type(&apply_subst(&ty, subst)),
                    span: arg.span().clone(),
                });
            }
        }
    }
    Some(apply_subst(&vec_ty, subst))
}

fn infer_string_concat(
    args: &[WatAST],
    _head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let string_ty = TypeExpr::Path(":String".into());
    for arg in args {
        if let Some(ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&ty, &string_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::string::concat".into(),
                    param: "arg".into(),
                    expected: ":String".into(),
                    got: format_type(&apply_subst(&ty, subst)),
                    span: arg.span().clone(),
                });
            }
        }
    }
    Some(string_ty)
}

fn infer_list_constructor(
    args: &[WatAST],
    head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // :wat::core::vec / :wat::core::list — `(vec :T x1 x2 ...) -> Vec<T>`.
    // First arg is a type keyword (read, not inferred); remaining args
    // must unify with T. Explicit typing is required even for non-empty
    // literals — the shape never depends on content or context.
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::vec".into(),
            expected: 1,
            got: 0,
            span: head_span.clone(),
        });
        let t = fresh.fresh();
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![t],
        });
    }
    let elem_ty = match &args[0] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(_) => {
                errors.push(CheckError::MalformedForm {
                    head: ":wat::core::vec".into(),
                    reason: format!("first argument {} is not a valid type keyword", k),
                    span: args[0].span().clone(),
                });
                fresh.fresh()
            }
        },
        _ => {
            errors.push(CheckError::MalformedForm {
                head: ":wat::core::vec".into(),
                reason: "first argument must be a type keyword (e.g., :i64)".into(),
                span: args[0].span().clone(),
            });
            fresh.fresh()
        }
    };
    for (i, arg) in args[1..].iter().enumerate() {
        let arg_ty = infer(arg, env, locals, fresh, subst, errors);
        if let Some(arg_ty) = arg_ty {
            if unify(&arg_ty, &elem_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::vec".into(),
                    param: format!("#{}", i + 2),
                    expected: format_type(&apply_subst(&elem_ty, subst)),
                    got: format_type(&apply_subst(&arg_ty, subst)),
                    span: arg.span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![apply_subst(&elem_ty, subst)],
    })
}

/// A lambda expression's type is `:fn(<param types>) -> <return type>`.
/// The signature is mandatory per 058-029 — every param and the
/// return are annotated. The body is checked against the declared
/// return type (same discipline as `check_function_body`).
fn infer_lambda(
    args: &[WatAST],
    _head_span: &Span,
    env: &CheckEnv,
    outer_locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        return None;
    }
    let sig = &args[0];
    let body = &args[1];
    let (param_names, param_types, ret_type) = parse_lambda_signature_for_check(sig).ok()?;

    // Check body against declared return type under extended locals.
    let mut body_locals = outer_locals.clone();
    for (name, ty) in param_names.iter().zip(param_types.iter()) {
        body_locals.insert(name.clone(), ty.clone());
    }
    // Push this lambda's declared return type onto the enclosing-ret
    // stack so `try` inside the body propagates to the lambda's
    // boundary (matches Rust's `?`-operator scoping — short-circuits
    // the innermost fn or closure, not the outer function).
    fresh.push_enclosing_ret(ret_type.clone());
    let body_ty = infer(body, env, &body_locals, fresh, subst, errors);
    fresh.pop_enclosing_ret();
    if let Some(body_ty) = body_ty {
        if unify(&body_ty, &ret_type, subst, env.types()).is_err() {
            errors.push(CheckError::ReturnTypeMismatch {
                function: format!("<lambda@{}>", body.span()),
                expected: format_type(&apply_subst(&ret_type, subst)),
                got: format_type(&apply_subst(&body_ty, subst)),
                span: body.span().clone(),
            });
        }
    }

    Some(TypeExpr::Fn {
        args: param_types,
        ret: Box::new(ret_type),
    })
}

/// Mirror of [`crate::runtime::parse_lambda_signature`] shape for the
/// check pass — returns (names, types, ret). Errors are silenced; if
/// the lambda is malformed, runtime parsing catches it and the
/// checker simply returns None.
fn parse_lambda_signature_for_check(
    sig: &WatAST,
) -> Result<(Vec<String>, Vec<TypeExpr>, TypeExpr), ()> {
    let items = match sig {
        WatAST::List(items, _) => items,
        _ => return Err(()),
    };
    let mut names = Vec::new();
    let mut types = Vec::new();
    let mut ret: Option<TypeExpr> = None;
    let mut saw_arrow = false;
    for item in items {
        if saw_arrow {
            if ret.is_some() {
                return Err(());
            }
            match item {
                WatAST::Keyword(k, _) => {
                    ret = Some(crate::types::parse_type_expr(k).map_err(|_| ())?);
                }
                _ => return Err(()),
            }
            continue;
        }
        match item {
            WatAST::Symbol(s, _) if s.as_str() == "->" => saw_arrow = true,
            WatAST::List(pair, _) => {
                if pair.len() != 2 {
                    return Err(());
                }
                let name = match &pair[0] {
                    WatAST::Symbol(s, _) => s.name.clone(),
                    _ => return Err(()),
                };
                let ty = match &pair[1] {
                    WatAST::Keyword(k, _) => crate::types::parse_type_expr(k).map_err(|_| ())?,
                    _ => return Err(()),
                };
                names.push(name);
                types.push(ty);
            }
            _ => return Err(()),
        }
    }
    Ok((names, types, ret.ok_or(())?))
}

fn infer_boolean_shortcircuit(
    args: &[WatAST],
    _head_span: &Span,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    // `and` / `or` take any number of :bool args, return :bool.
    for (i, arg) in args.iter().enumerate() {
        let arg_ty = infer(arg, env, locals, fresh, subst, errors);
        if let Some(arg_ty) = arg_ty {
            if unify(&arg_ty, &TypeExpr::Path(":bool".into()), subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::and/or".into(),
                    param: format!("#{}", i + 1),
                    expected: ":bool".into(),
                    got: format_type(&apply_subst(&arg_ty, subst)),
                    span: arg.span().clone(),
                });
            }
        }
    }
    Some(TypeExpr::Path(":bool".into()))
}

// ─── Unification ────────────────────────────────────────────────────────

#[derive(Debug)]
struct UnifyError;

/// Attempt to unify two type expressions under the given substitution.
/// Extends `subst` on success; leaves it untouched on failure.
fn unify(
    a: &TypeExpr,
    b: &TypeExpr,
    subst: &mut Subst,
    types: &TypeEnv,
) -> Result<(), UnifyError> {
    // Reduce both sides to canonical shape before the structural
    // match — follow Var bindings AND expand typealiases at each
    // level. The recursive unify-on-children calls reduce at their
    // levels; combined, every position in both type trees is seen
    // post-alias. `:MyCache<K,V>` and its expansion
    // `:rust::lru::LruCache<K,V>` unify structurally as a result.
    let a = reduce(&walk(a, subst), subst, types);
    let b = reduce(&walk(b, subst), subst, types);
    match (&a, &b) {
        (TypeExpr::Var(x), TypeExpr::Var(y)) if x == y => Ok(()),
        (TypeExpr::Var(x), other) | (other, TypeExpr::Var(x)) => {
            if occurs(*x, other, subst) {
                return Err(UnifyError);
            }
            subst.insert(*x, other.clone());
            Ok(())
        }
        (TypeExpr::Path(p1), TypeExpr::Path(p2)) => {
            if p1 == p2 {
                Ok(())
            } else {
                Err(UnifyError)
            }
        }
        (
            TypeExpr::Parametric { head: h1, args: a1 },
            TypeExpr::Parametric { head: h2, args: a2 },
        ) => {
            if h1 != h2 || a1.len() != a2.len() {
                return Err(UnifyError);
            }
            for (x, y) in a1.iter().zip(a2.iter()) {
                unify(x, y, subst, types)?;
            }
            Ok(())
        }
        (TypeExpr::Fn { args: a1, ret: r1 }, TypeExpr::Fn { args: a2, ret: r2 }) => {
            if a1.len() != a2.len() {
                return Err(UnifyError);
            }
            for (x, y) in a1.iter().zip(a2.iter()) {
                unify(x, y, subst, types)?;
            }
            unify(r1, r2, subst, types)
        }
        (TypeExpr::Tuple(e1), TypeExpr::Tuple(e2)) => {
            if e1.len() != e2.len() {
                return Err(UnifyError);
            }
            for (x, y) in e1.iter().zip(e2.iter()) {
                unify(x, y, subst, types)?;
            }
            Ok(())
        }
        _ => Err(UnifyError),
    }
}

/// Chase a type through the substitution map until a non-bound root
/// is reached. Does not mutate the subst — callers use this to peek
/// at the current binding without path-compressing.
fn walk(ty: &TypeExpr, subst: &Subst) -> TypeExpr {
    let mut current = ty.clone();
    loop {
        match &current {
            TypeExpr::Var(id) => match subst.get(id) {
                Some(next) => current = next.clone(),
                None => return current,
            },
            _ => return current,
        }
    }
}

// ─── Rust-deps scheme dispatch ───────────────────────────────────────

/// Dispatch a `:rust::*` call to the shim's scheme function registered
/// in the rust-deps registry. Wraps the checker's internal state in a
/// [`CheckSchemeCtx`] that implements [`crate::rust_deps::SchemeCtx`],
/// giving the shim a narrow interface that doesn't depend on this
/// module's private types.
#[allow(clippy::too_many_arguments)]
fn dispatch_rust_scheme(
    keyword: &str,
    head_span: &Span,
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    let registry = crate::rust_deps::get();
    let sym_entry = match registry.get_symbol(keyword) {
        Some(s) => s,
        None => {
            errors.push(CheckError::UnknownCallee {
                callee: keyword.to_string(),
                span: head_span.clone(),
            });
            return None;
        }
    };
    let mut ctx = CheckSchemeCtx {
        env,
        locals,
        fresh,
        subst,
        errors,
    };
    (sym_entry.scheme)(args, &mut ctx)
}

/// Adapter that presents the checker's internal state (`env`, `locals`,
/// `fresh`, `subst`, `errors`) through the narrow
/// [`crate::rust_deps::SchemeCtx`] trait. Lets shim authors write their
/// scheme functions without depending on `check.rs`'s private types.
struct CheckSchemeCtx<'a> {
    env: &'a CheckEnv,
    locals: &'a HashMap<String, TypeExpr>,
    fresh: &'a mut InferCtx,
    subst: &'a mut Subst,
    errors: &'a mut Vec<CheckError>,
}

impl<'a> crate::rust_deps::SchemeCtx for CheckSchemeCtx<'a> {
    fn fresh_var(&mut self) -> TypeExpr {
        self.fresh.fresh()
    }

    fn infer(&mut self, ast: &WatAST) -> Option<TypeExpr> {
        infer(ast, self.env, self.locals, self.fresh, self.subst, self.errors)
    }

    fn unify_types(&mut self, a: &TypeExpr, b: &TypeExpr) -> bool {
        unify(a, b, self.subst, self.env.types()).is_ok()
    }

    fn apply_subst(&self, t: &TypeExpr) -> TypeExpr {
        apply_subst(t, self.subst)
    }

    fn push_type_mismatch(
        &mut self,
        callee: &str,
        param: &str,
        expected: String,
        got: String,
        span: Span,
    ) {
        self.errors.push(CheckError::TypeMismatch {
            callee: callee.into(),
            param: param.into(),
            expected,
            got,
            span, // arc 138 F2: real span threaded through
        });
    }

    fn push_arity_mismatch(&mut self, callee: &str, expected: usize, got: usize, span: Span) {
        self.errors.push(CheckError::ArityMismatch {
            callee: callee.into(),
            expected,
            got,
            span, // arc 138 F2: real span threaded through
        });
    }

    fn push_malformed(&mut self, head: &str, reason: String, span: Span) {
        self.errors.push(CheckError::MalformedForm {
            head: head.into(),
            reason,
            span, // arc 138 F2: real span threaded through
        });
    }

    fn parse_type_keyword(&self, keyword: &str) -> Result<TypeExpr, crate::types::TypeError> {
        crate::types::parse_type_expr(keyword)
    }
}

/// Apply the substitution map deeply — rewrites every `Var(id)` in
/// `ty` to its bound target (transitively). **Does NOT expand
/// typealiases.** `:MyAlias<i64>` stays `:MyAlias<i64>`.
///
/// Use this for **error display** — it preserves the surface name
/// the user wrote, so `TypeMismatch` reads "expected
/// `:wat::stream::Stream<i64>`", not the tuple expansion.
///
/// For **structural matching** against the canonical form of a type,
/// call [`reduce`] instead.
fn apply_subst(ty: &TypeExpr, subst: &Subst) -> TypeExpr {
    match ty {
        TypeExpr::Var(id) => match subst.get(id) {
            Some(inner) => apply_subst(inner, subst),
            None => TypeExpr::Var(*id),
        },
        TypeExpr::Path(_) => ty.clone(),
        TypeExpr::Parametric { head, args } => TypeExpr::Parametric {
            head: head.clone(),
            args: args.iter().map(|a| apply_subst(a, subst)).collect(),
        },
        TypeExpr::Fn { args, ret } => TypeExpr::Fn {
            args: args.iter().map(|a| apply_subst(a, subst)).collect(),
            ret: Box::new(apply_subst(ret, subst)),
        },
        TypeExpr::Tuple(elements) => TypeExpr::Tuple(
            elements.iter().map(|e| apply_subst(e, subst)).collect(),
        ),
    }
}

/// Fully reduce a type to its **canonical structural form** — follow
/// every Var substitution AND expand every typealias, at every level
/// of the tree. This is the single normalization pass: any
/// shape-inspection site (matching on `TypeExpr::Tuple`,
/// `TypeExpr::Parametric { head, ... }`, `TypeExpr::Fn`, etc.) should
/// call this before the match, so aliases never hide structure from
/// the check.
///
/// Relationship to the other passes:
///
/// - [`apply_subst`] is "walk Vars, preserve alias names." Right for
///   error messages (the surface name is what the user wrote).
/// - [`crate::types::expand_alias`] is "peel aliases at one level,
///   leave Vars." Right internally during unify to establish the
///   root shape before unifying children.
/// - `reduce` is both, recursively. Right for every shape-direct
///   inspection where the alias is incidental and the structural
///   root is what matters.
///
/// `unify`'s prologue also calls `reduce` — both sides see canonical
/// shapes before the structural match below runs, and the recursive
/// unify-on-children calls reduce at each level.
fn reduce(ty: &TypeExpr, subst: &Subst, types: &TypeEnv) -> TypeExpr {
    let expanded = crate::types::expand_alias(ty, types);
    match expanded {
        TypeExpr::Var(id) => match subst.get(&id) {
            Some(inner) => reduce(inner, subst, types),
            None => TypeExpr::Var(id),
        },
        TypeExpr::Path(_) => expanded,
        TypeExpr::Parametric { head, args } => TypeExpr::Parametric {
            head,
            args: args.iter().map(|a| reduce(a, subst, types)).collect(),
        },
        TypeExpr::Fn { args, ret } => TypeExpr::Fn {
            args: args.iter().map(|a| reduce(a, subst, types)).collect(),
            ret: Box::new(reduce(&ret, subst, types)),
        },
        TypeExpr::Tuple(elements) => TypeExpr::Tuple(
            elements.iter().map(|e| reduce(e, subst, types)).collect(),
        ),
    }
}

/// Occurs check — prevents binding `α := foo(α)`.
fn occurs(id: u64, ty: &TypeExpr, subst: &Subst) -> bool {
    let ty = walk(ty, subst);
    match &ty {
        TypeExpr::Var(other) => *other == id,
        TypeExpr::Path(_) => false,
        TypeExpr::Parametric { args, .. } => args.iter().any(|a| occurs(id, a, subst)),
        TypeExpr::Fn { args, ret } => {
            args.iter().any(|a| occurs(id, a, subst)) || occurs(id, ret, subst)
        }
        TypeExpr::Tuple(elements) => elements.iter().any(|e| occurs(id, e, subst)),
    }
}

/// Instantiate a scheme's universally-quantified type parameters with
/// fresh unification variables. Produces monomorphic `(params, ret)`.
fn instantiate(scheme: &TypeScheme, fresh: &mut InferCtx) -> (Vec<TypeExpr>, TypeExpr) {
    if scheme.type_params.is_empty() {
        return (scheme.params.clone(), scheme.ret.clone());
    }
    let mut mapping: HashMap<String, TypeExpr> = HashMap::new();
    for tp in &scheme.type_params {
        mapping.insert(tp.clone(), fresh.fresh());
    }
    let params = scheme
        .params
        .iter()
        .map(|p| rename(p, &mapping))
        .collect();
    let ret = rename(&scheme.ret, &mapping);
    (params, ret)
}

/// Replace `Path(":T")` occurrences where T is a key in `mapping`
/// with the mapping's value. Used by [`instantiate`] to convert a
/// rigid type variable name into a fresh unification var.
fn rename(ty: &TypeExpr, mapping: &HashMap<String, TypeExpr>) -> TypeExpr {
    match ty {
        TypeExpr::Path(p) => {
            let key = p.strip_prefix(':').unwrap_or(p);
            if let Some(replacement) = mapping.get(key) {
                replacement.clone()
            } else {
                ty.clone()
            }
        }
        TypeExpr::Parametric { head, args } => TypeExpr::Parametric {
            head: head.clone(),
            args: args.iter().map(|a| rename(a, mapping)).collect(),
        },
        TypeExpr::Fn { args, ret } => TypeExpr::Fn {
            args: args.iter().map(|a| rename(a, mapping)).collect(),
            ret: Box::new(rename(ret, mapping)),
        },
        TypeExpr::Tuple(elements) => {
            TypeExpr::Tuple(elements.iter().map(|e| rename(e, mapping)).collect())
        }
        TypeExpr::Var(_) => ty.clone(),
    }
}

// ─── Pretty printing ────────────────────────────────────────────────────

/// Arc 143 — exposed so `runtime.rs` helpers can render a `TypeExpr`
/// as a keyword string for AST reconstruction in the three introspection
/// primitives (`lookup-define`, `signature-of`, `body-of`).
pub fn format_type(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Path(p) => p.clone(),
        TypeExpr::Parametric { head, args } => {
            let inner: Vec<_> = args.iter().map(format_type_inner).collect();
            format!(":{}<{}>", head, inner.join(","))
        }
        TypeExpr::Fn { args, ret } => {
            let in_parts: Vec<_> = args.iter().map(format_type_inner).collect();
            format!(":fn({})->{}", in_parts.join(","), format_type_inner(ret))
        }
        TypeExpr::Tuple(elements) => {
            let inner: Vec<_> = elements.iter().map(format_type_inner).collect();
            if elements.len() == 1 {
                // 1-tuple requires trailing comma to disambiguate
                // from parenthesization.
                format!(":({},)", inner[0])
            } else {
                format!(":({})", inner.join(","))
            }
        }
        TypeExpr::Var(id) => format!(":?{}", id),
    }
}

/// Arc 143 — exposed as companion to `format_type` (used recursively
/// for inner type arguments where the leading `:` is omitted).
pub fn format_type_inner(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Path(p) => p.strip_prefix(':').unwrap_or(p).to_string(),
        TypeExpr::Parametric { head, args } => {
            let inner: Vec<_> = args.iter().map(format_type_inner).collect();
            format!("{}<{}>", head, inner.join(","))
        }
        TypeExpr::Fn { args, ret } => {
            let in_parts: Vec<_> = args.iter().map(format_type_inner).collect();
            format!("fn({})->{}", in_parts.join(","), format_type_inner(ret))
        }
        TypeExpr::Tuple(elements) => {
            let inner: Vec<_> = elements.iter().map(format_type_inner).collect();
            if elements.len() == 1 {
                format!("({},)", inner[0])
            } else {
                format!("({})", inner.join(","))
            }
        }
        TypeExpr::Var(id) => format!("?{}", id),
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────

fn build_locals(
    param_names: &[String],
    param_types: &[TypeExpr],
) -> HashMap<String, TypeExpr> {
    let mut locals = HashMap::new();
    for (name, ty) in param_names.iter().zip(param_types.iter()) {
        locals.insert(name.clone(), ty.clone());
    }
    locals
}

fn derive_scheme_from_function(func: &Function) -> Option<TypeScheme> {
    // `runtime::Function` carries declared type-parameters, parameter
    // types, and the return type since slice 7b. Lambdas (name = None)
    // leave param_types empty and aren't statically typed here.
    func.name.as_ref()?;
    Some(TypeScheme {
        type_params: func.type_params.clone(),
        params: func.param_types.clone(),
        ret: func.ret_type.clone(),
    })
}

// ─── Built-in schemes ───────────────────────────────────────────────────

fn register_builtins(env: &mut CheckEnv) {
    let i64_ty = || TypeExpr::Path(":i64".into());
    let u8_ty = || TypeExpr::Path(":u8".into());
    let f64_ty = || TypeExpr::Path(":f64".into());
    let bool_ty = || TypeExpr::Path(":bool".into());
    let holon_ty = || TypeExpr::Path(":wat::holon::HolonAST".into());
    let t_var = || TypeExpr::Path(":T".into());

    // :u8 range-checked cast from :i64. Arc 008 slice 1. Runtime
    // rejects out-of-range values (0..=255) with a MalformedForm.
    env.register(
        ":wat::core::u8".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty()],
            ret: u8_ty(),
        },
    );

    // :wat::io::IOReader + :wat::io::IOWriter abstract IO substrate.
    // Arc 008 slice 2. Two opaque wat types; multiple concrete
    // backings (real stdio, StringIo). Byte-oriented primitives with
    // char-level conveniences.
    let string_ty = || TypeExpr::Path(":String".into());
    let unit_ty = || TypeExpr::Tuple(vec![]);
    let vec_u8_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![u8_ty()],
    };
    let opt_vec_u8_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![vec_u8_ty()],
    };
    let opt_string_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![string_ty()],
    };
    let ioreader_ty = || TypeExpr::Path(":wat::io::IOReader".into());
    let iowriter_ty = || TypeExpr::Path(":wat::io::IOWriter".into());

    // IOReader — construction + ops.
    env.register(
        ":wat::io::IOReader/from-bytes".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![vec_u8_ty()],
            ret: ioreader_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/from-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: ioreader_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/read".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![ioreader_ty(), i64_ty()],
            ret: opt_vec_u8_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/read-all".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![ioreader_ty()],
            ret: vec_u8_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/read-line".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![ioreader_ty()],
            ret: opt_string_ty(),
        },
    );
    env.register(
        ":wat::io::IOReader/rewind".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![ioreader_ty()],
            ret: unit_ty(),
        },
    );

    // IOWriter — construction + ops + snapshot.
    env.register(
        ":wat::io::IOWriter/new".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: iowriter_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/open-file".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":String".into())],
            ret: iowriter_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/to-bytes".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty()],
            ret: vec_u8_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/to-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty()],
            ret: opt_string_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/write".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), vec_u8_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/write-all".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), vec_u8_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/write-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), string_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/print".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), string_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/println".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), string_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/writeln".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty(), string_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::io::IOWriter/flush".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty()],
            ret: unit_ty(),
        },
    );
    // Arc 103b — explicit close for pipe-backed writers. Idempotent.
    // For non-pipe backings (StringIoWriter, RealStdout, RealStderr)
    // close is a no-op — closing real OS stdio would break the
    // parent process.
    env.register(
        ":wat::io::IOWriter/close".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![iowriter_ty()],
            ret: unit_ty(),
        },
    );
    // Arc 093 — auto-deleting temp file / temp dir wrappers
    // around Rust's `tempfile` crate. Drop unlinks the file/dir
    // when the wat value's Arc-count reaches zero. Caller binds
    // the handle in let*, pulls the path string when needed,
    // lets Drop fire at scope exit.
    env.register(
        ":wat::io::TempFile/new".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: TypeExpr::Path(":wat::io::TempFile".into()),
        },
    );
    env.register(
        ":wat::io::TempFile/path".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::io::TempFile".into())],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::io::TempDir/new".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: TypeExpr::Path(":wat::io::TempDir".into()),
        },
    );
    env.register(
        ":wat::io::TempDir/path".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::io::TempDir".into())],
            ret: string_ty(),
        },
    );
    // `(:wat::io::read-file path) -> :String` — read file
    // content via the host-attached SourceLoader. Same
    // capability discipline as `:wat::load-file!` /
    // `:wat::eval-file!`. First consumer: dispatcher-style
    // scripts that read EDN from stdin and forward a query
    // program's source to `:wat::kernel::run-sandboxed`.
    env.register(
        ":wat::io::read-file".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: string_ty(),
        },
    );

    // :wat::kernel::run-sandboxed (string entry) and
    // :wat::kernel::run-sandboxed-ast (forms entry) — arc 007.
    // RETIRED at the substrate level by arc 105c. Both now live as
    // wat-level defines in `wat/std/sandbox.wat`, atop:
    //   - arc 105a's :wat::kernel::spawn-program (Result-returning)
    //   - arc 105b's :wat::kernel::ThreadDiedError/message accessor
    // Their schemes register from the wat-level (define ...) forms
    // at startup; this comment block stands as the substrate-side
    // grave marker so future readers can find the relocation.
    //
    // The string-entry hermetic primitive was retired in arc 012
    // slice 3; its AST-entry sibling lives in wat/std/hermetic.wat
    // atop fork-program-ast.

    // :wat::kernel::run-sandboxed-hermetic-ast — retired as a Rust
    // primitive in arc 012 slice 3. Shipped as wat stdlib in
    // wat/std/hermetic.wat on top of fork-program-ast + wait-child
    // + struct-new. The keyword path + signature + return type are
    // identical; only the implementation layer moved. See
    // docs/arc/2026/04/012-fork-and-pipes/ for the arc's record.

    // :wat::kernel::assertion-failed! — arc 007 slice 3. Raises via
    // panic_any(AssertionPayload) so run-sandboxed's catch_unwind can
    // downcast and populate Failure.actual / Failure.expected. The op
    // NEVER RETURNS at runtime — the panic unwinds the stack — so the
    // return type is polymorphic per arc 107: `∀T. ... -> :T`. T
    // unifies with whatever the caller's context demands, including
    // `:()` (existing test-stdlib call sites) AND non-`:()` types
    // (`:wat::std::option::expect<T>` and `:wat::std::result::expect<T,E>`
    // need `:T` arms). The previous declared `:()` was a lie since
    // wat has no `Never` type; T is the honest scheme.
    env.register(
        ":wat::kernel::assertion-failed!".to_string(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                string_ty(),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![string_ty()],
                },
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![string_ty()],
                },
            ],
            ret: TypeExpr::Path(":T".into()),
        },
    );

    // :wat::kernel::raise! — arc 113 closure. Panics with a
    // structured HolonAST payload riding the AssertionPayload's
    // `data` field. Catches in run-sandboxed populate
    // `Failure/data` with the original HolonAST — the receiver
    // gets the data Value back, not a stringified rendering.
    //
    // Sibling of `assertion-failed!`: same panic_any mechanism,
    // same catch path, same polymorphic return (`∀T. -> :T` —
    // never returns; T unifies with caller context). The
    // difference is what's emitted: `assertion-failed!` carries
    // (message, actual, expected) for assert-* failures;
    // `raise!` carries an arbitrary HolonAST under `data` for
    // user-defined structured errors. Once the chain is in EDN,
    // panic messages stop being strings — they become Values
    // with addressable fields.
    env.register(
        ":wat::kernel::raise!".to_string(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Path(":wat::holon::HolonAST".into())],
            ret: TypeExpr::Path(":T".into()),
        },
    );

    // Integer arithmetic — strict i64 × i64 → i64 under the
    // `:wat::core::i64::*` namespace.
    for op in &[
        ":wat::core::i64::+",
        ":wat::core::i64::-",
        ":wat::core::i64::*",
        ":wat::core::i64::/",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![i64_ty(), i64_ty()],
                ret: i64_ty(),
            },
        );
    }
    // Float arithmetic — strict f64 × f64 → f64 under the
    // `:wat::core::f64::*` namespace. Users commit to int or float at
    // the call site; no implicit promotion.
    for op in &[
        ":wat::core::f64::+",
        ":wat::core::f64::-",
        ":wat::core::f64::*",
        ":wat::core::f64::/",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![f64_ty(), f64_ty()],
                ret: f64_ty(),
            },
        );
    }
    // Arc 019 — f64 rounding primitive. `(round v digits) -> f64`
    // rounds `v` to `digits` decimal places using round-half-away-
    // from-zero. `digits=0` rounds to the nearest integer;
    // `digits=2` rounds to two decimals. Negative `digits` rounds
    // to tens / hundreds / etc. NaN and ±∞ pass through unchanged.
    env.register(
        ":wat::core::f64::round".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty(), i64_ty()],
            ret: f64_ty(),
        },
    );

    // Arc 046 — strict-f64 max / min / abs / clamp. Lab arc 015
    // surfaced these as substrate gaps while porting indicator
    // vocab; lifting them here means every wat consumer reaches
    // for the same names rather than reinventing in userland.
    for op in &[":wat::core::f64::max", ":wat::core::f64::min"] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![f64_ty(), f64_ty()],
                ret: f64_ty(),
            },
        );
    }
    env.register(
        ":wat::core::f64::abs".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty()],
            ret: f64_ty(),
        },
    );
    env.register(
        ":wat::core::f64::clamp".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty(), f64_ty(), f64_ty()],
            ret: f64_ty(),
        },
    );

    // Arc 047 — Vec aggregates and the `last` accessor return
    // Option to honestly signal empty/no-match. Same reasoning as
    // the polymorphism shift on first/second/third for Vec inputs.
    let opt = |inner: TypeExpr| TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![inner],
    };
    let vec_of = |inner: TypeExpr| TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![inner],
    };
    env.register(
        ":wat::core::last".to_string(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var())],
            ret: opt(t_var()),
        },
    );
    env.register(
        ":wat::core::find-last-index".to_string(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var()],
                    ret: Box::new(bool_ty()),
                },
            ],
            ret: opt(i64_ty()),
        },
    );
    env.register(
        ":wat::core::f64::max-of".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![vec_of(f64_ty())],
            ret: opt(f64_ty()),
        },
    );
    env.register(
        ":wat::core::f64::min-of".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![vec_of(f64_ty())],
            ret: opt(f64_ty()),
        },
    );

    // Scalar conversions — arc 014. :wat::core::<source>::to-<target>
    // between the four scalar tiers (i64, f64, bool, String).
    // Infallible ones return the target directly; fallible ones return
    // :Option<T>. No implicit coercion — every conversion is an
    // explicit named call at the call site.
    let opt_i64_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![i64_ty()],
    };
    let opt_f64_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![f64_ty()],
    };
    let opt_bool_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![bool_ty()],
    };
    env.register(
        ":wat::core::i64::to-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::core::i64::to-f64".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty()],
            ret: f64_ty(),
        },
    );
    env.register(
        ":wat::core::f64::to-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::core::f64::to-i64".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty()],
            ret: opt_i64_ty(),
        },
    );
    env.register(
        ":wat::core::string::to-i64".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: opt_i64_ty(),
        },
    );
    env.register(
        ":wat::core::string::to-f64".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: opt_f64_ty(),
        },
    );
    env.register(
        ":wat::core::bool::to-string".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![bool_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::core::string::to-bool".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: opt_bool_ty(),
        },
    );

    // String basics — :wat::core::string::*. Per-type ops, char-
    // oriented (length counts unicode scalars, not bytes). See
    // src/string_ops.rs for the handlers.
    for op in &[
        ":wat::core::string::contains?",
        ":wat::core::string::starts-with?",
        ":wat::core::string::ends-with?",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![string_ty(), string_ty()],
                ret: bool_ty(),
            },
        );
    }
    env.register(
        ":wat::core::string::length".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::core::string::trim".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::core::string::split".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty(), string_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![string_ty()],
            },
        },
    );
    env.register(
        ":wat::core::string::join".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(),
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![string_ty()],
                },
            ],
            ret: string_ty(),
        },
    );

    // Regex — :wat::core::regex::*. matches? is unanchored (pattern
    // match anywhere in haystack); wrap with ^...$ for full-string.
    env.register(
        ":wat::core::regex::matches?".to_string(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty(), string_ty()],
            ret: bool_ty(),
        },
    );

    // Comparison / equality — arc 050. The polymorphic forms
    // (`:wat::core::=`, `<`, `>`, `<=`, `>=`) are special-cased in
    // `infer_list` so they accept mixed numeric pairs (i64+f64) and
    // promote at runtime. For non-numeric types they still require
    // both operands to be the same type, same as the prior
    // `∀T. T → T → :bool` shape. No scheme registration here — the
    // special-case branch handles inference end-to-end.

    // Typed strict comparison/equality — arc 050. Power-user opt-in
    // for callers who want the type-guard behavior. Reject mixed
    // input at the checker; runtime delegates to the same eval_eq /
    // eval_compare paths.
    for op in &[
        ":wat::core::i64::=",
        ":wat::core::i64::<",
        ":wat::core::i64::>",
        ":wat::core::i64::<=",
        ":wat::core::i64::>=",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![i64_ty(), i64_ty()],
                ret: bool_ty(),
            },
        );
    }
    for op in &[
        ":wat::core::f64::=",
        ":wat::core::f64::<",
        ":wat::core::f64::>",
        ":wat::core::f64::<=",
        ":wat::core::f64::>=",
    ] {
        env.register(
            op.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![f64_ty(), f64_ty()],
                ret: bool_ty(),
            },
        );
    }
    // Polymorphic arithmetic — arc 050. Special-cased in `infer_list`
    // for the cross-numeric promotion rule (i64+f64→f64). No scheme
    // registration here — the special-case branch handles inference
    // end-to-end.

    // Boolean negation.
    env.register(
        ":wat::core::not".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![bool_ty()],
            ret: bool_ty(),
        },
    );

    // Algebra-core UpperCalls.
    // Atom — ∀T. T → :wat::holon::HolonAST. Polymorphic per arc 057
    // (primitive → leaf; HolonAST → opaque-wrap; quoted form →
    // structural lower). Arc 065 introduced named siblings (`leaf`
    // for primitives, `from-watast` for quoted forms) so consumers
    // can pick the verb that names the move; the polymorphism
    // stays for back-compat across ~960 existing call sites.
    env.register(
        ":wat::holon::Atom".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![t_var()],
            ret: holon_ty(),
        },
    );
    // Arc 065 named siblings of polymorphic Atom — one verb per
    // move. Polymorphism for back-compat; named ops for new code.
    env.register(
        ":wat::holon::leaf".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![t_var()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::holon::from-watast".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::WatAST".into())],
            ret: holon_ty(),
        },
    );
    // atom-value — ∀T. :wat::holon::HolonAST → :T. Dual of Atom. The caller's
    // let-binding type ascription (or surrounding context) pins T; the
    // runtime dispatches on the holon's variant and errors when the
    // variant doesn't match the expected return type.
    env.register(
        ":wat::core::atom-value".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![holon_ty()],
            ret: t_var(),
        },
    );

    // to-watast — :wat::holon::HolonAST → :wat::WatAST. Story-2 escape
    // hatch per arc 057: structural inverse of Atom's quote-lowering.
    // Pair with :wat::eval-ast! when you want the value, not the
    // coordinate.
    env.register(
        ":wat::holon::to-watast".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: TypeExpr::Path(":wat::WatAST".into()),
        },
    );

    // Term decomposition (arc 073). The Prolog/population-code primitive:
    // any HolonAST decomposes into (template, slots, ranges). Templates
    // compare exactly; slots compare via tolerance; ranges parameterize
    // the tolerance window. Three substrate functions; three pure shapes.
    env.register(
        ":wat::holon::term::template".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::holon::term::slots".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Path(":f64".into())],
            },
        },
    );
    env.register(
        ":wat::holon::term::ranges".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Tuple(vec![
                    TypeExpr::Path(":f64".into()),
                    TypeExpr::Path(":f64".into()),
                ])],
            },
        },
    );
    // term::matches? — composes template + slots + ranges + sigma. The
    // population-code unification primitive: same cell type AND every
    // slot within the substrate's coincident floor. Cheaper than
    // `coincident?` for forms that share a template (no encoding pass).
    env.register(
        ":wat::holon::term::matches?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: TypeExpr::Path(":bool".into()),
        },
    );

    // Arc 074 — substrate floor accessors. Read the substrate's
    // presence/coincident floor at d. Users compose these into filter
    // funcs they pass to `Hologram/get`.
    env.register(
        ":wat::holon::presence-floor".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":i64".into())],
            ret: TypeExpr::Path(":f64".into()),
        },
    );
    env.register(
        ":wat::holon::coincident-floor".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":i64".into())],
            ret: TypeExpr::Path(":f64".into()),
        },
    );

    // Arc 076 — therm-routed Hologram. Slot derives from the form's
    // structure (first Thermometer leaf's normalized floor, or slot 0
    // for non-therm). Filter is bound at construction; get is filtered-
    // argmax. HolonAST → HolonAST.
    let hologram_ty = || TypeExpr::Path(":wat::holon::Hologram".into());
    let f64_ty = || TypeExpr::Path(":f64".into());
    let bool_ty = || TypeExpr::Path(":bool".into());
    let filter_ty = || TypeExpr::Fn {
        args: vec![f64_ty()],
        ret: Box::new(bool_ty()),
    };
    env.register(
        ":wat::holon::Hologram/make".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![filter_ty()],
            ret: hologram_ty(),
        },
    );
    env.register(
        ":wat::holon::Hologram/put".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![hologram_ty(), holon_ty(), holon_ty()],
            ret: TypeExpr::Tuple(vec![]),
        },
    );
    env.register(
        ":wat::holon::Hologram/get".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![hologram_ty(), holon_ty()],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![holon_ty()],
            },
        },
    );
    env.register(
        ":wat::holon::Hologram/find".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![hologram_ty(), holon_ty()],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![TypeExpr::Tuple(vec![holon_ty(), holon_ty()])],
            },
        },
    );
    env.register(
        ":wat::holon::Hologram/remove".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![hologram_ty(), holon_ty()],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![holon_ty()],
            },
        },
    );
    env.register(
        ":wat::holon::Hologram/len".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![hologram_ty()],
            ret: TypeExpr::Path(":i64".into()),
        },
    );
    env.register(
        ":wat::holon::Hologram/capacity".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![hologram_ty()],
            ret: TypeExpr::Path(":i64".into()),
        },
    );

    // Arc 076 slice 2 — therm-form constructor. Carries the user's
    // natural domain on the form; Hologram applies its own capacity at
    // slot routing time. No capacity arg.
    env.register(
        ":wat::holon::therm-form".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );

    // The eval-family forms — per the 2026-04-20 INSCRIPTION adding
    // :Result<wat::holon::HolonAST, :wat::core::EvalError> as the uniform
    // return type. Every dynamic evaluation failure (verification,
    // parse, mutation-form refused, unknown function, type mismatch,
    // etc.) becomes an Err value in the Result rather than an
    // unwinding RuntimeError. `:wat::core::try` inside eval'd code
    // continues to propagate as before — the TryPropagate signal
    // passes through the dispatcher's wrap.
    //
    // Arg types keep the pre-inscription looseness (the structural
    // keywords and payload strings aren't type-validated in fine
    // detail) — the purpose of adding these schemes is to enforce
    // the return shape at every call site, not to tighten arg
    // checking. A future pass may narrow arg types as real misuse
    // surfaces.
    let eval_result_ty = || TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            holon_ty(),
            TypeExpr::Path(":wat::core::EvalError".into()),
        ],
    };
    let wat_ast_ty = || TypeExpr::Path(":wat::WatAST".into());
    let keyword_ty = || TypeExpr::Path(":wat::core::keyword".into());
    let string_ty = || TypeExpr::Path(":String".into());

    // Arc 028 slice 3 — eval family iface drop. Each form takes its
    // source/path directly as the first arg; no interface keyword.
    // eval-edn! narrowed to string-only (one source shape per form,
    // like load! / load-string!).
    // Arc 102 — `:wat::eval-ast!` returns `Result<:T, :EvalError>`
    // polymorphic. Same trust-the-caller discipline as
    // `:wat::edn::read` / `:wat::eval-edn!`: the caller annotates
    // T with the type they expect the inner eval to produce; the
    // runtime returns whatever it actually produces (bare Value,
    // not the arc-066 `value_to_holon` wrap that arc 102 reverts).
    // Type-mismatched downstream ops fail at runtime in the same
    // way they would for any `read-edn`-typed binding.
    env.register(
        ":wat::eval-ast!".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![wat_ast_ty()],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    TypeExpr::Path("T".into()),
                    TypeExpr::Path(":wat::core::EvalError".into()),
                ],
            },
        },
    );
    // :wat::eval-step! (arc 068) — one CBV reduction at the leftmost-
    // outermost redex. Returns Ok(StepResult) on progress (StepNext,
    // StepTerminal, or AlreadyTerminal — arc 070); Err(EvalError) for
    // malformed forms, effectful ops in step mode, or shapes the
    // stepper hasn't been taught yet.
    env.register(
        ":wat::eval-step!".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![wat_ast_ty()],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    TypeExpr::Path(":wat::eval::StepResult".into()),
                    TypeExpr::Path(":wat::core::EvalError".into()),
                ],
            },
        },
    );
    // :wat::eval::walk<A> (arc 070) — fold over the eval-step! chain.
    // Visitor sees every coordinate exactly once, in order, with the
    // step-result the substrate produced at that coordinate. Returns
    // (terminal-HolonAST, final-acc) on Ok; the chain's terminal +
    // the visitor's accumulated state.
    env.register(
        ":wat::eval::walk".into(),
        TypeScheme {
            type_params: vec!["A".into()],
            params: vec![
                wat_ast_ty(),
                TypeExpr::Path("A".into()),
                TypeExpr::Fn {
                    args: vec![
                        TypeExpr::Path("A".into()),
                        wat_ast_ty(),
                        TypeExpr::Path(":wat::eval::StepResult".into()),
                    ],
                    ret: Box::new(TypeExpr::Parametric {
                        head: "wat::eval::WalkStep".into(),
                        args: vec![TypeExpr::Path("A".into())],
                    }),
                },
            ],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    TypeExpr::Tuple(vec![
                        TypeExpr::Path(":wat::holon::HolonAST".into()),
                        TypeExpr::Path("A".into()),
                    ]),
                    TypeExpr::Path(":wat::core::EvalError".into()),
                ],
            },
        },
    );
    env.register(
        ":wat::eval-edn!".into(),
        TypeScheme {
            type_params: vec![],
            // <source-string>
            params: vec![string_ty()],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-file!".into(),
        TypeScheme {
            type_params: vec![],
            // <path>
            params: vec![string_ty()],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-digest!".into(),
        TypeScheme {
            type_params: vec![],
            // <path>, :wat::verify::digest-<algo>, :wat::verify::<iface>, <hex>
            params: vec![string_ty(), keyword_ty(), keyword_ty(), string_ty()],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-digest-string!".into(),
        TypeScheme {
            type_params: vec![],
            // <source>, :wat::verify::digest-<algo>, :wat::verify::<iface>, <hex>
            params: vec![string_ty(), keyword_ty(), keyword_ty(), string_ty()],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-signed!".into(),
        TypeScheme {
            type_params: vec![],
            // <path>, :wat::verify::signed-<algo>,
            // :wat::verify::<iface>, <sig>, :wat::verify::<iface>, <pubkey>
            params: vec![
                string_ty(),
                keyword_ty(),
                keyword_ty(),
                string_ty(),
                keyword_ty(),
                string_ty(),
            ],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::eval-signed-string!".into(),
        TypeScheme {
            type_params: vec![],
            // <source>, :wat::verify::signed-<algo>,
            // :wat::verify::<iface>, <sig>, :wat::verify::<iface>, <pubkey>
            params: vec![
                string_ty(),
                keyword_ty(),
                keyword_ty(),
                string_ty(),
                keyword_ty(),
                string_ty(),
            ],
            ret: eval_result_ty(),
        },
    );
    env.register(
        ":wat::holon::Bind".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: holon_ty(),
        },
    );
    // Bundle takes :wat::holon::Holons and returns
    // :Result<wat::holon::HolonAST, :wat::holon::CapacityExceeded>.
    // The Result wrap is the forcing function for the capacity guard:
    // authors are required by the type system to acknowledge the
    // failure case — either matching explicitly or propagating via
    // `:wat::core::try`. Under `:error` the Err arm fires with the
    // cost/budget struct; under `:panic` the process panics before
    // returning.
    env.register(
        ":wat::holon::Bundle".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![holon_ty()],
            }],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    holon_ty(),
                    TypeExpr::Path(":wat::holon::CapacityExceeded".into()),
                ],
            },
        },
    );
    env.register(
        ":wat::holon::Permute".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), i64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::holon::Thermometer".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![f64_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::holon::Blend".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty(), f64_ty(), f64_ty()],
            ret: holon_ty(),
        },
    );

    // Cosine measurement — the retrieval scalar (FOUNDATION 1718 +
    // OPEN-QUESTIONS line 419). Algebra-substrate operation (input is
    // holons, not raw numbers).
    //   (:wat::holon::cosine      target ref) -> :f64
    //   (:wat::holon::presence?   target ref) -> :bool (cosine > noise-floor)
    //   (:wat::holon::coincident? a      b  ) -> :bool ((1 - cosine) < noise-floor)
    //     dual to presence? — same threshold, equivalence direction. Arc 023.
    //
    // Arc 052: cosine and dot are special-cased in `infer_list` to
    // accept HolonAST OR Vector inputs (polymorphic). No scheme
    // registration here for those two — their inference branches in
    // infer_list cover both AST-AST and Vector-Vector and mixed cases.
    // presence? and coincident? remain HolonAST-only and keep their
    // scheme registrations.
    env.register(
        ":wat::holon::presence?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), holon_ty()],
            ret: bool_ty(),
        },
    );
    // :wat::holon::coincident? scheme retired in arc 061; polymorphic
    // under `infer_polymorphic_holon_pair_to_bool` (HolonAST | Vector
    // in either position; returns :bool). Same shape as cosine's
    // arc-052 polymorphism.

    // eval-coincident? family — arc 026. Each variant mirrors its
    // eval-*! parent's arg shape, applied per-side (2 sides per
    // variant). Return is uniform Result<bool, EvalError> — any
    // failure on either side arrives as Err<EvalError>.
    let eval_coincident_ret = || TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            bool_ty(),
            TypeExpr::Path(":wat::core::EvalError".into()),
        ],
    };
    // slice 1 — base (AST). Takes two WatAST args (quote-captured).
    env.register(
        ":wat::holon::eval-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![wat_ast_ty(), wat_ast_ty()],
            ret: eval_coincident_ret(),
        },
    );
    // Arc 028 slice 3 — eval-coincident family arities updated to
    // match new eval-*! shapes (iface keyword dropped).
    // EDN variant — 2 source strings.
    env.register(
        ":wat::holon::eval-edn-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty(), string_ty()],
            ret: eval_coincident_ret(),
        },
    );
    // digest variant — 2 × (path, algo, payload-iface, hex) = 8 args.
    env.register(
        ":wat::holon::eval-digest-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(), keyword_ty(), keyword_ty(), string_ty(),
                string_ty(), keyword_ty(), keyword_ty(), string_ty(),
            ],
            ret: eval_coincident_ret(),
        },
    );
    // digest-string variant — same arity, inline sources.
    env.register(
        ":wat::holon::eval-digest-string-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(), keyword_ty(), keyword_ty(), string_ty(),
                string_ty(), keyword_ty(), keyword_ty(), string_ty(),
            ],
            ret: eval_coincident_ret(),
        },
    );
    // signed variant — 2 × (path, algo, sig-iface, sig, pk-iface, pk) = 12 args.
    env.register(
        ":wat::holon::eval-signed-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(), keyword_ty(), keyword_ty(), string_ty(), keyword_ty(), string_ty(),
                string_ty(), keyword_ty(), keyword_ty(), string_ty(), keyword_ty(), string_ty(),
            ],
            ret: eval_coincident_ret(),
        },
    );
    // signed-string variant — same arity, inline sources.
    env.register(
        ":wat::holon::eval-signed-string-coincident?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(), keyword_ty(), keyword_ty(), string_ty(), keyword_ty(), string_ty(),
                string_ty(), keyword_ty(), keyword_ty(), string_ty(), keyword_ty(), string_ty(),
            ],
            ret: eval_coincident_ret(),
        },
    );

    // Config accessors — nullary, read committed startup values.
    // Arc 077: dim-count + dim-capacity are the program-d surfaces.
    // (Pre-arc-077 alias `:wat::config::dims` removed; use dim-count.)
    env.register(
        ":wat::config::dim-count".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::config::dim-capacity".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::config::global-seed".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::config::noise-floor".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: f64_ty(),
        },
    );

    // Kernel primitives.
    // (:wat::kernel::stopped) → :bool.
    env.register(
        ":wat::kernel::stopped?".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: bool_ty(),
        },
    );
    // (:wat::kernel::pipe) → :(wat::io::IOWriter, wat::io::IOReader).
    // Arc 012 slice 1b. Writer first (producer), reader second.
    env.register(
        ":wat::kernel::pipe".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: TypeExpr::Tuple(vec![
                TypeExpr::Path(":wat::io::IOWriter".into()),
                TypeExpr::Path(":wat::io::IOReader".into()),
            ]),
        },
    );
    // (:wat::kernel::fork-program-ast forms) → :wat::kernel::Process<I, O>.
    // Arc 012 slice 2 + arc 112 unification. Forks the current
    // wat process (COW-inheriting the loaded substrate), runs the
    // caller's forms as a fresh :user::main in the child, returns the
    // unified Process<I,O> struct (same shape spawn-program returns;
    // only the internal join handle's variant differs — Forked vs
    // InThread). Pre-arc-112 returned a separate :wat::kernel::ForkedChild
    // type; that type retired in arc 112 because the only difference
    // from Process was the wait mechanism, which lives inside
    // ProgramHandle's enum variant.
    let process_ty = || TypeExpr::Parametric {
        head: "wat::kernel::Process".into(),
        args: vec![
            TypeExpr::Path(":I".into()),
            TypeExpr::Path(":O".into()),
        ],
    };
    env.register(
        ":wat::kernel::fork-program-ast".into(),
        TypeScheme {
            type_params: vec!["I".into(), "O".into()],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Path(":wat::WatAST".into())],
            }],
            ret: process_ty(),
        },
    );
    // (:wat::kernel::fork-program src scope) → :wat::kernel::Process<I, O>.
    // Arc 104b + arc 112 unification.
    env.register(
        ":wat::kernel::fork-program".into(),
        TypeScheme {
            type_params: vec!["I".into(), "O".into()],
            params: vec![
                TypeExpr::Path(":String".into()),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":String".into())],
                },
            ],
            ret: process_ty(),
        },
    );
    // (:wat::kernel::spawn-program src scope) →
    //   :Result<:wat::kernel::Process, :wat::kernel::StartupError>.
    // (:wat::kernel::spawn-program-ast forms scope) → same.
    //
    // Arc 103. The in-thread sibling of `fork-program-ast` — same
    // `(IOWriter, IOReader, IOReader, ProgramHandle<()>)` shape, but
    // the inner program runs on a `std::thread` instead of a forked
    // OS process. Caller writes EDN+newline to `proc.stdin`, blocks
    // on `read-line` from `proc.stdout` — mini-TCP via kernel pipes.
    // See `docs/ZERO-MUTEX.md` §"Mini-TCP via paired channels".
    //
    // Arc 105a: failures during freeze (parse / type-check / config)
    // or `:user::main` signature validation surface as
    // `(Err startup-error)` data instead of raising — wat-level
    // callers pattern-match. A successful spawn yields `(Ok proc)`.
    //
    // No substrate `spawn-program-hermetic-ast`. Today's hermetic
    // distinction means real fork (`fork-program-ast`); in-thread
    // "hermetic" reduces to "inner program declares its own Config
    // preamble," which is a wat-level discipline.
    // Arc 112 — Process<I, O> phantom-param lift; uses the
    // `process_ty` closure declared above for fork-program-ast.
    let process_or_startup_error = || TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            process_ty(),
            TypeExpr::Path(":wat::kernel::StartupError".into()),
        ],
    };
    env.register(
        ":wat::kernel::spawn-program".into(),
        TypeScheme {
            type_params: vec!["I".into(), "O".into()],
            params: vec![
                TypeExpr::Path(":String".into()),
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":String".into())],
                },
            ],
            ret: process_or_startup_error(),
        },
    );
    env.register(
        ":wat::kernel::spawn-program-ast".into(),
        TypeScheme {
            type_params: vec!["I".into(), "O".into()],
            params: vec![
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":wat::WatAST".into())],
                },
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![TypeExpr::Path(":String".into())],
                },
            ],
            ret: process_or_startup_error(),
        },
    );
    // (:wat::kernel::Process/join-result proc) →
    //   :Result<:(), :wat::kernel::ProcessDiedError>.
    // Arc 112 — the canonical death-as-data wait verb on a unified
    // Process<I,O>. Internally dispatches on the Process's join-field
    // ProgramHandle variant: InThread (spawn-program origin) does
    // arc 060's recv-on-channel; Forked (fork-program origin) does
    // waitpid. Both arms synthesize the outcome as ProcessDiedError
    // so the receiver matches one shape regardless of how the Program
    // was spawned. Symmetric with arc 060's bare
    // `:wat::kernel::join-result handle :ProgramHandle<R>`, which
    // continues to return `Result<R, ThreadDiedError>` on the bare
    // Thread side. Pre-arc-112 callers used the now-retired
    // `:wat::kernel::wait-child` returning :i64; the migration is
    // `(wait-child (ForkedChild/handle child))` →
    // `(:wat::kernel::Process/join-result proc)`.
    // Arc 113 — Err arm widens to Vec<ProcessDiedError> chain.
    let process_died_chain_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![TypeExpr::Path(":wat::kernel::ProcessDiedError".into())],
    };
    env.register(
        ":wat::kernel::Process/join-result".into(),
        TypeScheme {
            type_params: vec!["I".into(), "O".into()],
            params: vec![process_ty()],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    TypeExpr::Tuple(vec![]),
                    process_died_chain_ty(),
                ],
            },
        },
    );
    // (:wat::kernel::spawn-thread body) →
    //   :wat::kernel::Thread<I,O>.
    //
    // Arc 114 slice 1. The in-thread sibling of `:wat::kernel::fork-program`,
    // satisfying the same Program contract — input channel, output
    // channel, error mechanism via join. Body is a function whose
    // signature MUST be
    //   :Fn(:Receiver<I>, :Sender<O>) -> :wat::core::unit
    // (the body reads from the input half, writes to the output half;
    // values flow only through channels — never via a return).
    //
    // Returns Thread<I,O>. The parent gets the OUTSIDE ends:
    //   `Thread/input`   :Sender<I>     (parent writes; thread reads)
    //   `Thread/output`  :Receiver<O>   (thread writes; parent reads)
    //   `Thread/join`    :ProgramHandle (panic surfaces on Thread/join-result)
    //
    // Arc 114 names the meta-principle this verb expresses: hosting
    // is a user choice (thread vs forked process); the protocol is
    // fixed (typed channels in / out, panic via join). Code that
    // talks to a Program does not know or care which host backs it.
    let thread_ty = || TypeExpr::Parametric {
        head: "wat::kernel::Thread".into(),
        args: vec![
            TypeExpr::Path(":I".into()),
            TypeExpr::Path(":O".into()),
        ],
    };
    let thread_body_fn_ty = || TypeExpr::Fn {
        args: vec![
            TypeExpr::Parametric {
                head: "rust::crossbeam_channel::Receiver".into(),
                args: vec![TypeExpr::Path(":I".into())],
            },
            TypeExpr::Parametric {
                head: "rust::crossbeam_channel::Sender".into(),
                args: vec![TypeExpr::Path(":O".into())],
            },
        ],
        ret: Box::new(TypeExpr::Tuple(vec![])),
    };
    env.register(
        ":wat::kernel::spawn-thread".into(),
        TypeScheme {
            type_params: vec!["I".into(), "O".into()],
            params: vec![thread_body_fn_ty()],
            ret: thread_ty(),
        },
    );
    // (:wat::kernel::Thread/join-result thr) →
    //   :Result<:(), :Vec<:wat::kernel::ThreadDiedError>>.
    //
    // Arc 114 slice 1. Symmetric with `Process/join-result` (arc 112)
    // but for the in-thread satisfier. Threads share memory with the
    // parent; panic info travels through the spawn driver's
    // `catch_unwind` channel and surfaces on this verb's Err arm —
    // there is no separate stderr stream (that's how processes
    // recover panic; threads don't need to). Arc 113's
    // `Vec<ThreadDiedError>` chain shape applies — head is the
    // immediate thread that died; tail captures whatever upstream
    // chain its panic carried.
    let thread_died_chain_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![TypeExpr::Path(":wat::kernel::ThreadDiedError".into())],
    };
    env.register(
        ":wat::kernel::Thread/join-result".into(),
        TypeScheme {
            type_params: vec!["I".into(), "O".into()],
            params: vec![thread_ty()],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    TypeExpr::Tuple(vec![]),
                    thread_died_chain_ty(),
                ],
            },
        },
    );
    // (:wat::kernel::process-send proc :I) →
    //   :Result<:(), :wat::kernel::ProcessDiedError>
    // Arc 112 slice 2b — typed value send to a Process's stdin.
    // Renders the value via :wat::edn::write (arc 092 EDN v4),
    // appends a newline, writes to proc.stdin via
    // IOWriter/write-string. Returns Ok(()) on landed write;
    // Err(ProcessDiedError::ChannelDisconnected) when the pipe is
    // closed (peer Process exited or panicked before reading).
    //
    // Pre-§J spelling. Post-arc-109 § J slice 10f this verb
    // becomes :wat::kernel::Process/send under the typed-method
    // naming convention; same shape, just renamed.
    env.register(
        ":wat::kernel::process-send".into(),
        TypeScheme {
            type_params: vec!["I".into(), "O".into()],
            params: vec![process_ty(), TypeExpr::Path(":I".into())],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    TypeExpr::Tuple(vec![]),
                    process_died_chain_ty(),
                ],
            },
        },
    );
    // (:wat::kernel::process-recv proc) →
    //   :Result<:Option<:O>, :wat::kernel::ProcessDiedError>
    // Arc 112 slice 2b — typed value recv from a Process's stdout.
    // Reads one line from stdout via IOReader/read-line, parses
    // via :wat::edn::read (arc 092). Three-state shape mirrors
    // arc 111's intra-process recv:
    //
    //   Ok(Some v)  — child wrote one EDN-framed O; parsed; here.
    //   Ok(:None)   — child stdout EOF + clean exit (no stderr,
    //                 exit code 0).
    //   Err(died)   — child stdout EOF + non-zero exit OR stderr
    //                 lines populated. died.message carries the
    //                 stderr contents joined; arc 113 widens this
    //                 to a Vec<ProgramDiedError> chain.
    //
    // Slice-2b limitation (matches arc 105c hermetic.wat's pattern):
    // reads stdout primarily; stderr drained only on stdout EOF.
    // Children that write to stderr WHILE stdout is being read
    // surface stderr only after stdout EOFs. Multiplex-during-stream
    // is follow-up substrate work when a caller needs it.
    //
    // Pre-§J spelling. Post-arc-109 § J slice 10f this verb
    // becomes :wat::kernel::Process/recv under the typed-method
    // naming convention; same shape, just renamed.
    env.register(
        ":wat::kernel::process-recv".into(),
        TypeScheme {
            type_params: vec!["I".into(), "O".into()],
            params: vec![process_ty()],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    TypeExpr::Parametric {
                        head: "Option".into(),
                        args: vec![TypeExpr::Path(":O".into())],
                    },
                    process_died_chain_ty(),
                ],
            },
        },
    );
    // User-signal surface — 2026-04-19 stance: kernel measures, userland
    // owns transitions. Six nullary primitives: three pollers return
    // :bool; three resetters return :(). SIGINT / SIGTERM stay on the
    // `stopped` flag above.
    for path in [
        ":wat::kernel::sigusr1?",
        ":wat::kernel::sigusr2?",
        ":wat::kernel::sighup?",
    ] {
        env.register(
            path.into(),
            TypeScheme {
                type_params: vec![],
                params: vec![],
                ret: bool_ty(),
            },
        );
    }
    for path in [
        ":wat::kernel::reset-sigusr1!",
        ":wat::kernel::reset-sigusr2!",
        ":wat::kernel::reset-sighup!",
    ] {
        env.register(
            path.into(),
            TypeScheme {
                type_params: vec![],
                params: vec![],
                ret: TypeExpr::Tuple(vec![]),
            },
        );
    }
    // Arc 111 — three states surfaced at the type level:
    //   recv: Result<Option<T>, ThreadDiedError>
    //         Ok(Some v)      — value flowed
    //         Ok(:None)       — clean shutdown (every sender dropped via scope)
    //         Err(ThreadDied) — sender thread panicked
    //   send: Result<(), ThreadDiedError>
    //         Ok(())          — delivered
    //         Err(ThreadDied) — receiver gone (clean vs panic in the
    //                           ThreadDiedError variants per arc 060)
    // Slice 2 wires the OnceLock plumbing so Err carries the rich panic
    // message; slice 1 ships the type shape with Err unreachable from
    // the runtime path (still always ChannelDisconnected as a stand-in).
    // Arc 113 — Err arm widens from a single ThreadDiedError to a
    // `Vec<ThreadDiedError>` (chained-cause backtrace). Vec is the
    // chain: head = the immediate peer that died; tail = whatever
    // killed it, transitively. Slice 1 ships the wire shape; slice 2
    // wires auto-conj at every cross-thread hand-off boundary so the
    // substrate produces real chains. Pre-arc-113 consumers matching
    // `((Err died) ...)` against a single ThreadDiedError now match
    // against a Vec<ThreadDiedError>; common shape `((Err chain)
    // (handle (:wat::core::Vector/first chain)))` to recover head.
    let died_chain_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![TypeExpr::Path(":wat::kernel::ThreadDiedError".into())],
    };
    let comm_ok_option_t = || TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![t_var()],
            },
            died_chain_ty(),
        ],
    };
    let comm_send_ret = || TypeExpr::Parametric {
        head: "Result".into(),
        args: vec![
            TypeExpr::Tuple(vec![]),
            died_chain_ty(),
        ],
    };
    // (:wat::kernel::send sender value) —
    //   ∀T. Sender<T> × T -> Result<(), ThreadDiedError>.
    env.register(
        ":wat::kernel::send".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                TypeExpr::Parametric {
                    head: "rust::crossbeam_channel::Sender".into(),
                    args: vec![t_var()],
                },
                t_var(),
            ],
            ret: comm_send_ret(),
        },
    );
    // (:wat::kernel::try-recv receiver) —
    //   ∀T. Receiver<T> -> Result<Option<T>, ThreadDiedError>.
    // Ok(:None) covers both empty and clean-disconnected (try-recv
    // doesn't block; Err only fires for sender-thread panic).
    env.register(
        ":wat::kernel::try-recv".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "rust::crossbeam_channel::Receiver".into(),
                args: vec![t_var()],
            }],
            ret: comm_ok_option_t(),
        },
    );
    // (:wat::kernel::recv receiver) —
    //   ∀T. Receiver<T> -> Result<Option<T>, ThreadDiedError>.
    env.register(
        ":wat::kernel::recv".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "rust::crossbeam_channel::Receiver".into(),
                args: vec![t_var()],
            }],
            ret: comm_ok_option_t(),
        },
    );
    // (:wat::kernel::join handle) — ∀R. ProgramHandle<R> -> R.
    let r_var = || TypeExpr::Path(":R".into());
    env.register(
        ":wat::kernel::join".into(),
        TypeScheme {
            type_params: vec!["R".into()],
            params: vec![TypeExpr::Parametric {
                head: "wat::kernel::ProgramHandle".into(),
                args: vec![r_var()],
            }],
            ret: r_var(),
        },
    );
    // (:wat::kernel::join-result handle) — arc 060 + arc 113.
    //   ∀R. ProgramHandle<R> -> Result<R, Vec<wat::kernel::ThreadDiedError>>
    // Sibling to join: same blocking recv on the spawn channel; differs
    // in failure handling (data-as-Result instead of panic-the-caller).
    // Arc 113 widened the Err arm to a Vec — head = the spawned thread's
    // death; tail = whatever killed it transitively.
    env.register(
        ":wat::kernel::join-result".into(),
        TypeScheme {
            type_params: vec!["R".into()],
            params: vec![TypeExpr::Parametric {
                head: "wat::kernel::ProgramHandle".into(),
                args: vec![r_var()],
            }],
            ret: TypeExpr::Parametric {
                head: "Result".into(),
                args: vec![
                    r_var(),
                    died_chain_ty(),
                ],
            },
        },
    );
    // (:wat::kernel::ThreadDiedError/message err) -> :String — arc 105b.
    // Extracts the carried message from any ThreadDiedError variant;
    // returns "channel disconnected" for the unit variant. Routes
    // around the wat-side enum-pattern type-checker gap that arc 103b
    // surfaced. Wat callers (wat/std/sandbox.wat) use this to build
    // RunResult.failure.message without variant discrimination.
    env.register(
        ":wat::kernel::ThreadDiedError/message".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::kernel::ThreadDiedError".into())],
            ret: TypeExpr::Path(":String".into()),
        },
    );
    // (:wat::kernel::ThreadDiedError/to-failure err) -> :wat::kernel::Failure
    // — arc 105c. Always returns a structured Failure, preserving
    // arc 064's actual/expected/location/frames through run-sandboxed
    // when the panic carried an AssertionPayload. Plain panics and
    // non-panic variants get a message-only Failure. wat/std/
    // sandbox.wat's failure-from-thread-died routes through this.
    env.register(
        ":wat::kernel::ThreadDiedError/to-failure".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::kernel::ThreadDiedError".into())],
            ret: TypeExpr::Path(":wat::kernel::Failure".into()),
        },
    );
    // (:wat::kernel::ProcessDiedError/message err) -> :String — arc 112.
    // Sibling of ThreadDiedError/message for the Process<I,O> subject.
    env.register(
        ":wat::kernel::ProcessDiedError/message".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::kernel::ProcessDiedError".into())],
            ret: TypeExpr::Path(":String".into()),
        },
    );
    // (:wat::kernel::ProcessDiedError/to-failure err) -> :wat::kernel::Failure
    // — arc 112. Sibling of ThreadDiedError/to-failure. Builds a
    // structured Failure regardless of variant; preserves arc-064
    // assertion-payload structure when present.
    env.register(
        ":wat::kernel::ProcessDiedError/to-failure".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::kernel::ProcessDiedError".into())],
            ret: TypeExpr::Path(":wat::kernel::Failure".into()),
        },
    );
    // (:wat::kernel::extract-panics (lines :Vec<String>))
    //   -> :Option<Vec<wat::kernel::ProcessDiedError>>
    //
    // Arc 113 slice 3 — process side of the cascade. Stderr is the
    // diagnostic side channel; the child writes a tagged EDN line
    // `#wat.kernel/Panics [...]` on AssertionPayload panic. This verb
    // walks the captured stderr-lines from end to start, locates
    // the marker, parses the body via the type registry, returns
    // the chain. The wat-side `drive-sandbox` prefers the parsed
    // chain when present; otherwise falls back to the singleton
    // shape from `Process/join-result`.
    //
    // Symmetry note: threads pass DiedError values directly through
    // crossbeam (zero-copy); processes pass them as EDN over kernel
    // pipes. The CHAIN SHAPE at the caller surface is identical —
    // `Result<R, Vec<*DiedError>>`. Only the transport differs;
    // `extract-panics` is the EDN side of the same coin.
    env.register(
        ":wat::kernel::extract-panics".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Path(":String".into())],
            }],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":wat::kernel::ProcessDiedError".into())],
                }],
            },
        },
    );
    // HandlePool — claim-or-panic discipline.
    //   new    : ∀T. :String -> :Vec<T> -> :HandlePool<T>
    //   pop    : ∀T. :HandlePool<T> -> :T
    //   finish : ∀T. :HandlePool<T> -> :()
    env.register(
        ":wat::kernel::HandlePool::new".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                TypeExpr::Path(":String".into()),
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![t_var()],
                },
            ],
            ret: TypeExpr::Parametric {
                head: "wat::kernel::HandlePool".into(),
                args: vec![t_var()],
            },
        },
    );
    env.register(
        ":wat::kernel::HandlePool::pop".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "wat::kernel::HandlePool".into(),
                args: vec![t_var()],
            }],
            ret: t_var(),
        },
    );
    env.register(
        ":wat::kernel::HandlePool::finish".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "wat::kernel::HandlePool".into(),
                args: vec![t_var()],
            }],
            ret: TypeExpr::Tuple(vec![]),
        },
    );
    // (:wat::kernel::select receivers) —
    //   ∀T. Vec<Receiver<T>> -> :(i64, Result<Option<T>, ThreadDiedError>).
    // Arc 111 — second tuple element grows from :Option<T> to
    // :Result<:Option<T>, :ThreadDiedError> for symmetry with recv.
    env.register(
        ":wat::kernel::select".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Parametric {
                    head: "rust::crossbeam_channel::Receiver".into(),
                    args: vec![t_var()],
                }],
            }],
            ret: TypeExpr::Tuple(vec![
                TypeExpr::Path(":i64".into()),
                comm_ok_option_t(),
            ]),
        },
    );
    // Algebra measurement: dot product. Per 058-005 new measurement
    // primitive. Scalar-returning sibling of cosine; used by the
    // Gram-Schmidt stdlib macros (Reject, Project).
    //
    // Arc 052: polymorphic via `infer_list` special-case branch (see
    // cosine note above); no scheme registration here.
    // Arc 052: Vector as first-class wat-tier value.
    // `:wat::holon::encode` materializes a HolonAST into a Vector at
    // the ambient d. The encoding context (vm/scalar/registry) is
    // ambient on the SymbolTable, same as cosine/dot/simhash; user
    // surface is one-arg.
    env.register(
        ":wat::holon::encode".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: TypeExpr::Path(":wat::holon::Vector".into()),
        },
    );
    // Arc 061 — vector portability. Serialize / deserialize the
    // wire format the cryptographic-substrate protocol uses to
    // transmit V (the encoded vector) between users. 4-byte dim
    // header + 2-bit-per-cell ternary packing; see
    // `eval_holon_vector_bytes` for the format. Arc 062 swaps the
    // verbose `:Vec<u8>` for `:wat::core::Bytes` (substrate-general
    // alias); both forms work at call sites because alias resolution
    // is structural.
    env.register(
        ":wat::holon::vector-bytes".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::holon::Vector".into())],
            ret: TypeExpr::Path(":wat::core::Bytes".into()),
        },
    );
    env.register(
        ":wat::holon::bytes-vector".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::core::Bytes".into())],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![TypeExpr::Path(":wat::holon::Vector".into())],
            },
        },
    );
    // Arc 063 — Bytes ↔ hex text bridge. to-hex emits lowercase
    // no-separator; from-hex accepts mixed case, returns :None on
    // odd length / non-hex character. Empty string round-trips to
    // empty Bytes.
    env.register(
        ":wat::core::Bytes::to-hex".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":wat::core::Bytes".into())],
            ret: TypeExpr::Path(":String".into()),
        },
    );
    env.register(
        ":wat::core::Bytes::from-hex".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Path(":String".into())],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![TypeExpr::Path(":wat::core::Bytes".into())],
            },
        },
    );
    // Arc 064 — polymorphic value rendering. `:wat::core::show<T>`
    // takes any value and returns a debug-friendly String. Used
    // internally by `:wat::test::assert-eq` to populate the failure
    // payload's actual/expected fields; exposed publicly so test
    // code and future assertions reuse it.
    env.register(
        ":wat::core::show".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![t_var()],
            ret: TypeExpr::Path(":String".into()),
        },
    );
    // Arc 079 — polymorphic value-to-EDN rendering. Three primitives
    // wrap wat-edn's writer: `write` (compact single-line), `write-pretty`
    // (multi-line indented), `write-json` (sentinel-tagged JSON).
    // Each takes any wat value and returns a String; consumers compose
    // them to render structured records on stdout (telemetry::Console)
    // or for cross-process IPC.
    for op in [
        ":wat::edn::write",
        ":wat::edn::write-pretty",
        ":wat::edn::write-json",
        ":wat::edn::write-notag",
        ":wat::edn::write-json-natural",
    ] {
        env.register(
            op.into(),
            TypeScheme {
                type_params: vec!["T".into()],
                params: vec![t_var()],
                ret: TypeExpr::Path(":String".into()),
            },
        );
    }
    // `(:wat::edn::read s)` → `:T`. Inverse of write — parses an
    // EDN string into a wat runtime Value. Polymorphic-fresh-var
    // return so the caller's binding context unifies with whatever
    // shape the parsed value takes; runtime mismatches surface as
    // pattern-match / accessor errors at the use site.
    env.register(
        ":wat::edn::read".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![TypeExpr::Path(":String".into())],
            ret: t_var(),
        },
    );
    // Arc 053: Vector-tier algebra primitives. Operate on raw
    // materialized Vectors without round-tripping through HolonAST.
    // Used by Phase 4 learning code that holds emergent vectors.
    let vector_ty = || TypeExpr::Path(":wat::holon::Vector".into());
    env.register(
        ":wat::holon::vector-bind".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![vector_ty(), vector_ty()],
            ret: vector_ty(),
        },
    );
    env.register(
        ":wat::holon::vector-bundle".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![vector_ty()],
            }],
            ret: vector_ty(),
        },
    );
    env.register(
        ":wat::holon::vector-blend".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![vector_ty(), vector_ty(), f64_ty(), f64_ty()],
            ret: vector_ty(),
        },
    );
    env.register(
        ":wat::holon::vector-permute".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![vector_ty(), i64_ty()],
            ret: vector_ty(),
        },
    );
    // Arc 053: OnlineSubspace native value + 10 core methods.
    let subspace_ty = || TypeExpr::Path(":wat::holon::OnlineSubspace".into());
    let vec_f64_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![f64_ty()],
    };
    env.register(
        ":wat::holon::OnlineSubspace/new".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty(), i64_ty()],
            ret: subspace_ty(),
        },
    );
    for unary_to_i64 in &[
        ":wat::holon::OnlineSubspace/dim",
        ":wat::holon::OnlineSubspace/k",
        ":wat::holon::OnlineSubspace/n",
    ] {
        env.register(
            unary_to_i64.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![subspace_ty()],
                ret: i64_ty(),
            },
        );
    }
    env.register(
        ":wat::holon::OnlineSubspace/threshold".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![subspace_ty()],
            ret: f64_ty(),
        },
    );
    env.register(
        ":wat::holon::OnlineSubspace/eigenvalues".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![subspace_ty()],
            ret: vec_f64_ty(),
        },
    );
    env.register(
        ":wat::holon::OnlineSubspace/update".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![subspace_ty(), vector_ty()],
            ret: f64_ty(),
        },
    );
    env.register(
        ":wat::holon::OnlineSubspace/residual".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![subspace_ty(), vector_ty()],
            ret: f64_ty(),
        },
    );
    for unary_to_vec in &[
        ":wat::holon::OnlineSubspace/project",
        ":wat::holon::OnlineSubspace/reconstruct",
    ] {
        env.register(
            unary_to_vec.to_string(),
            TypeScheme {
                type_params: vec![],
                params: vec![subspace_ty(), vector_ty()],
                ret: vec_f64_ty(),
            },
        );
    }

    // Arc 053: Reckoner native value + 8 core methods. Label is :i64;
    // Prediction is a wat tuple :(Vec<(i64,f64)>, Option<i64>, f64,
    // f64). ReckConfig is encoded in the constructor name (Discrete
    // vs Continuous).
    let reckoner_ty = || TypeExpr::Path(":wat::holon::Reckoner".into());
    let unit_ty = || TypeExpr::Tuple(vec![]);
    env.register(
        ":wat::holon::Reckoner/new-discrete".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![
                string_ty(),
                i64_ty(),
                i64_ty(),
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Path(":wat::holon::HolonAST".into())],
                },
            ],
            ret: reckoner_ty(),
        },
    );
    env.register(
        ":wat::holon::Reckoner/new-continuous".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty(), i64_ty(), i64_ty(), f64_ty(), i64_ty()],
            ret: reckoner_ty(),
        },
    );
    env.register(
        ":wat::holon::Reckoner/observe".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty(), vector_ty(), i64_ty(), f64_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::holon::Reckoner/predict".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty(), vector_ty()],
            ret: TypeExpr::Tuple(vec![
                TypeExpr::Parametric {
                    head: "Vec".into(),
                    args: vec![TypeExpr::Tuple(vec![i64_ty(), f64_ty()])],
                },
                TypeExpr::Parametric {
                    head: "Option".into(),
                    args: vec![i64_ty()],
                },
                f64_ty(),
                f64_ty(),
            ]),
        },
    );
    env.register(
        ":wat::holon::Reckoner/resolve".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty(), f64_ty(), bool_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::holon::Reckoner/curve".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty()],
            ret: TypeExpr::Parametric {
                head: "Option".into(),
                args: vec![TypeExpr::Tuple(vec![f64_ty(), f64_ty()])],
            },
        },
    );
    env.register(
        ":wat::holon::Reckoner/labels".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![i64_ty()],
            },
        },
    );
    env.register(
        ":wat::holon::Reckoner/dims".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![reckoner_ty()],
            ret: i64_ty(),
        },
    );

    // Arc 053: Engram native value + 4 read methods.
    let engram_ty = || TypeExpr::Path(":wat::holon::Engram".into());
    env.register(
        ":wat::holon::Engram/name".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![engram_ty()],
            ret: string_ty(),
        },
    );
    env.register(
        ":wat::holon::Engram/eigenvalue-signature".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![engram_ty()],
            ret: vec_f64_ty(),
        },
    );
    env.register(
        ":wat::holon::Engram/n".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![engram_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::holon::Engram/residual".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![engram_ty(), vector_ty()],
            ret: f64_ty(),
        },
    );

    // Arc 053: EngramLibrary native value + 6 core methods.
    let library_ty = || TypeExpr::Path(":wat::holon::EngramLibrary".into());
    env.register(
        ":wat::holon::EngramLibrary/new".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty()],
            ret: library_ty(),
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/add".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty(), string_ty(), subspace_ty()],
            ret: unit_ty(),
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/match-vec".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty(), vector_ty(), i64_ty(), i64_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![TypeExpr::Tuple(vec![string_ty(), f64_ty()])],
            },
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/len".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty()],
            ret: i64_ty(),
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/contains".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty(), string_ty()],
            ret: bool_ty(),
        },
    );
    env.register(
        ":wat::holon::EngramLibrary/names".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![library_ty()],
            ret: TypeExpr::Parametric {
                head: "Vec".into(),
                args: vec![string_ty()],
            },
        },
    );
    // Arc 051: SimHash — direction-space lattice position. Charikar's
    // hyperplane SimHash via the canonical Atom(0)..Atom(63) basis.
    // Maps an input holon to a 64-bit i64 key; cosine-similar inputs
    // share the same key (or near-same in hamming distance). Used as
    // the key-derivation function for bidirectional engram caches and
    // any content-addressed retrieval over the holon algebra.
    //
    // Arc 052: polymorphic via `infer_list` special-case branch —
    // accepts HolonAST or Vector input. No scheme registration here.
    // HolonAST → immediate surface arity. Returns the top-level
    // cardinality: 1 for leaf / Atom / Permute / Thermometer,
    // 2 for Bind / Blend, children.len() for Bundle. Useful for
    // user code that wants to introspect the shape of a form
    // (e.g., capacity-aware Bundle construction).
    env.register(
        ":wat::holon::statement-length".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: i64_ty(),
        },
    );

    // Arc 143 slice 1 — three runtime introspection primitives. Each
    // takes a :Symbol (keyword name) and returns :Option<HolonAST>:
    //   lookup-define  — full (:define <head> <body>) AST
    //   signature-of   — head only
    //   body-of        — body only (:None for substrate primitives)
    let symbol_ty = || TypeExpr::Path(":wat::core::keyword".into());
    let opt_holon_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![TypeExpr::Path(":wat::holon::HolonAST".into())],
    };
    env.register(
        ":wat::runtime::lookup-define".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![symbol_ty()],
            ret: opt_holon_ty(),
        },
    );
    env.register(
        ":wat::runtime::signature-of".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![symbol_ty()],
            ret: opt_holon_ty(),
        },
    );
    env.register(
        ":wat::runtime::body-of".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![symbol_ty()],
            ret: opt_holon_ty(),
        },
    );

    // Arc 143 slice 3 — HolonAST manipulation primitives.
    //
    // rename-callable-name (head :HolonAST) (from :keyword) (to :keyword) -> :HolonAST
    // extract-arg-names    (head :HolonAST)                               -> :Vec<keyword>
    //
    // The type-checker special-case in `infer_list` (check.rs:3126+) bypasses
    // normal type-unification for these primitives because the first argument
    // is a HolonAST value (not a plain keyword), which interacts with the arc-009
    // "names are values" dispatch the same way as the slice-1 introspection
    // primitives.
    let holon_ty = || TypeExpr::Path(":wat::holon::HolonAST".into());
    let vec_kw_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![TypeExpr::Path(":wat::core::keyword".into())],
    };
    env.register(
        ":wat::runtime::rename-callable-name".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty(), symbol_ty(), symbol_ty()],
            ret: holon_ty(),
        },
    );
    env.register(
        ":wat::runtime::extract-arg-names".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![holon_ty()],
            ret: vec_kw_ty(),
        },
    );

    // IO primitives — see `:wat::io::IOReader/*` + `:wat::io::IOWriter/*`
    // registered above. Arc 008 retired the earlier `:wat::io::write`
    // and `:wat::io::read-line` primitives (which dispatched on
    // `Value::io__Stdin/Stdout/Stderr` directly) in favour of the
    // abstract IOReader/IOWriter surface.

    // Stdlib math — single-method Rust calls per FOUNDATION-CHANGELOG
    // 2026-04-18. All unary :f64 -> :f64 except pi which is :() -> :f64.
    // Packaged here so Log / Circular expansions get proper checking.
    for name in ["ln", "log", "exp", "sin", "cos", "sqrt"] {
        env.register(
            format!(":wat::std::math::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![f64_ty()],
                ret: f64_ty(),
            },
        );
    }
    env.register(
        ":wat::std::math::pi".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: f64_ty(),
        },
    );

    // Stat reductions over Vec<f64> — population variance/stddev
    // (matches numpy default ddof=0); all return :Option<f64> with
    // None on empty input (matches f64::min-of/max-of convention).
    let opt_f64_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![f64_ty()],
    };
    let vec_f64_ty = || TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![f64_ty()],
    };
    for name in ["mean", "variance", "stddev"] {
        env.register(
            format!(":wat::std::stat::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![vec_f64_ty()],
                ret: opt_f64_ty(),
            },
        );
    }

    // Arc 056 — :wat::time::* surface. Sibling of :wat::io::* at the
    // same nesting depth (world-observing primitives, not pure
    // stdlib). Single Instant value type backs all 9 primitives;
    // duration measurement is two `now` calls + integer-accessor
    // subtract (no separate Duration type).
    let instant_ty = || TypeExpr::Path(":wat::time::Instant".into());
    let string_ty = || TypeExpr::Path(":String".into());
    let opt_instant_ty = || TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![instant_ty()],
    };
    env.register(
        ":wat::time::now".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![],
            ret: instant_ty(),
        },
    );
    for name in ["at", "at-millis", "at-nanos"] {
        env.register(
            format!(":wat::time::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![i64_ty()],
                ret: instant_ty(),
            },
        );
    }
    env.register(
        ":wat::time::from-iso8601".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![string_ty()],
            ret: opt_instant_ty(),
        },
    );
    env.register(
        ":wat::time::to-iso8601".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![instant_ty(), i64_ty()],
            ret: string_ty(),
        },
    );
    for name in ["epoch-seconds", "epoch-millis", "epoch-nanos"] {
        env.register(
            format!(":wat::time::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![instant_ty()],
                ret: i64_ty(),
            },
        );
    }

    // Arc 097 — Duration constructors. Seven unit constructors at
    // :wat::time::* (Nanosecond, Microsecond, Millisecond, Second,
    // Minute, Hour, Day). Each :i64 -> :wat::time::Duration.
    let duration_ty = || TypeExpr::Path(":wat::time::Duration".into());
    for name in [
        "Nanosecond",
        "Microsecond",
        "Millisecond",
        "Second",
        "Minute",
        "Hour",
        "Day",
    ] {
        env.register(
            format!(":wat::time::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![i64_ty()],
                ret: duration_ty(),
            },
        );
    }

    // Arc 097 slice 3 — `ago` / `from-now` composers. Each takes a
    // Duration and returns Instant (relative to wall-clock now).
    for name in ["ago", "from-now"] {
        env.register(
            format!(":wat::time::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![duration_ty()],
                ret: instant_ty(),
            },
        );
    }

    // Arc 097 slice 4 — pre-composed unit-ago / unit-from-now sugars.
    // 14 helpers (7 units × {ago, from-now}). Each takes :i64 and
    // returns Instant relative to wall-clock now.
    for name in [
        "nanoseconds-ago",
        "microseconds-ago",
        "milliseconds-ago",
        "seconds-ago",
        "minutes-ago",
        "hours-ago",
        "days-ago",
        "nanoseconds-from-now",
        "microseconds-from-now",
        "milliseconds-from-now",
        "seconds-from-now",
        "minutes-from-now",
        "hours-from-now",
        "days-from-now",
    ] {
        env.register(
            format!(":wat::time::{}", name),
            TypeScheme {
                type_params: vec![],
                params: vec![i64_ty()],
                ret: instant_ty(),
            },
        );
    }
    // List/Vec primitives — Round 4a, per docs/058-backlog.md.
    //
    //   length   : ∀T. Vec<T> -> :i64
    //   empty?   : ∀T. Vec<T> -> :bool
    //   reverse  : ∀T. Vec<T> -> Vec<T>
    //   range    : :i64 × :i64 -> Vec<i64>   (two-arg; no overload)
    //   take     : ∀T. Vec<T> × :i64 -> Vec<T>
    //   drop     : ∀T. Vec<T> × :i64 -> Vec<T>
    //   map      : ∀T,U. Vec<T> × fn(T)->U -> Vec<U>
    //   foldl    : ∀T,Acc. Vec<T> × Acc × fn(Acc,T)->Acc -> Acc
    //   window   : ∀T. Vec<T> × :i64 -> Vec<Vec<T>>   (at :wat::std::list::)
    let u_var = || TypeExpr::Path(":U".into());
    let acc_var = || TypeExpr::Path(":Acc".into());
    let vec_of = |inner: TypeExpr| TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![inner],
    };
    // :wat::core::length scheme retired; polymorphic under
    // `infer_length` (arc 035). Dispatched in `infer_list`.
    // :wat::core::empty? scheme retired (arc 058); polymorphic under
    // `infer_empty_q`. Same shape as `length` — Vec<T> | HashMap<K,V>
    // | HashSet<T> → bool.
    env.register(
        ":wat::core::reverse".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var())],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::core::range".into(),
        TypeScheme {
            type_params: vec![],
            params: vec![i64_ty(), i64_ty()],
            ret: vec_of(i64_ty()),
        },
    );
    env.register(
        ":wat::core::take".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var()), i64_ty()],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::core::drop".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var()), i64_ty()],
            ret: vec_of(t_var()),
        },
    );
    // Arc 056 — sort-by with user-supplied less-than predicate.
    // `(sort-by xs less?) -> Vec<T>` where `less? : :fn(T,T) -> :bool`.
    // The user owns asc vs desc via the predicate; key-extraction is
    // the predicate composing inner accessors. Common Lisp tradition.
    env.register(
        ":wat::core::sort-by".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var(), t_var()],
                    ret: Box::new(TypeExpr::Path(":bool".into())),
                },
            ],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::core::map".into(),
        TypeScheme {
            type_params: vec!["T".into(), "U".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var()],
                    ret: Box::new(u_var()),
                },
            ],
            ret: vec_of(u_var()),
        },
    );
    env.register(
        ":wat::core::foldl".into(),
        TypeScheme {
            type_params: vec!["T".into(), "Acc".into()],
            params: vec![
                vec_of(t_var()),
                acc_var(),
                TypeExpr::Fn {
                    args: vec![acc_var(), t_var()],
                    ret: Box::new(acc_var()),
                },
            ],
            ret: acc_var(),
        },
    );
    env.register(
        ":wat::core::foldr".into(),
        TypeScheme {
            type_params: vec!["T".into(), "Acc".into()],
            params: vec![
                vec_of(t_var()),
                acc_var(),
                TypeExpr::Fn {
                    args: vec![t_var(), acc_var()],
                    ret: Box::new(acc_var()),
                },
            ],
            ret: acc_var(),
        },
    );
    env.register(
        ":wat::core::filter".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var()],
                    ret: Box::new(bool_ty()),
                },
            ],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::std::list::zip".into(),
        TypeScheme {
            type_params: vec!["T".into(), "U".into()],
            params: vec![vec_of(t_var()), vec_of(u_var())],
            ret: vec_of(TypeExpr::Tuple(vec![t_var(), u_var()])),
        },
    );
    // get, assoc, conj, and contains? are all polymorphic over
    // container type — dispatched by the infer_* arms above. No
    // narrow schemes registered here.
    // :wat::std::member? RETIRED in arc 025. Use `:wat::core::contains?`
    // instead — now polymorphic over HashMap / HashSet / Vec.
    env.register(
        ":wat::std::list::remove-at".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var()), i64_ty()],
            ret: vec_of(t_var()),
        },
    );
    env.register(
        ":wat::std::list::window".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var()), i64_ty()],
            ret: vec_of(vec_of(t_var())),
        },
    );

    // first/second/third are special-cased (polymorphic over Vec + tuple;
    // see infer_positional_accessor). rest is simple:
    env.register(
        ":wat::core::rest".into(),
        TypeScheme {
            type_params: vec!["T".into()],
            params: vec![vec_of(t_var())],
            ret: vec_of(t_var()),
        },
    );
    // :wat::core::conj — polymorphic add-to-growing-collection.
    //   ∀T. Vec<T>     × T -> Vec<T>
    //   ∀T. HashSet<T> × T -> HashSet<T>
    // Illegal on HashMap (use assoc instead — HashMap needs key+value
    // pairing). Dispatched by `infer_conj` at check.rs arm above.
    //
    // No narrow scheme registered; handled entirely by infer_conj.
    // :wat::std::list::map-with-index — needed by Sequential for
    // indexed fold.
    env.register(
        ":wat::std::list::map-with-index".into(),
        TypeScheme {
            type_params: vec!["T".into(), "U".into()],
            params: vec![
                vec_of(t_var()),
                TypeExpr::Fn {
                    args: vec![t_var(), i64_ty()],
                    ret: Box::new(u_var()),
                },
            ],
            ret: vec_of(u_var()),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::{
        expand_all, register_defmacros, register_stdlib_defmacros, MacroRegistry,
    };
    use crate::runtime::{
        register_defines, register_stdlib_defines, register_struct_methods, Environment,
        SymbolTable,
    };
    use crate::types::{parse_type_expr, register_stdlib_types, register_types, TypeEnv};
    use std::sync::OnceLock;

    /// The stdlib is always part of the language. Test harnesses
    /// preload it once per process via `OnceLock`, clone the resulting
    /// state per test. This mirrors `startup_from_source`'s stdlib
    /// passes without running user-source phases, so every check()
    /// call sees `:wat::std::*` names, macros, and typealiases.
    fn stdlib_loaded() -> &'static (SymbolTable, MacroRegistry, TypeEnv) {
        static LOADED: OnceLock<(SymbolTable, MacroRegistry, TypeEnv)> = OnceLock::new();
        LOADED.get_or_init(|| {
            let stdlib = crate::stdlib::stdlib_forms().expect("stdlib parses");
            let mut macros = MacroRegistry::new();
            let stdlib_post_macros =
                register_stdlib_defmacros(stdlib, &mut macros).expect("stdlib defmacros");
            let expanded_stdlib = expand_all(
                stdlib_post_macros,
                &mut macros,
                &Environment::default(),
                &SymbolTable::default(),
            )
            .expect("stdlib macro expansion");
            let mut types = TypeEnv::with_builtins();
            let stdlib_post_types =
                register_stdlib_types(expanded_stdlib, &mut types).expect("stdlib types");
            let mut symbols = SymbolTable::new();
            let _ = register_stdlib_defines(stdlib_post_types, &mut symbols)
                .expect("stdlib defines");
            register_struct_methods(&types, &mut symbols)
                .expect("built-in struct methods");
            (symbols, macros, types)
        })
    }

    fn check(src: &str) -> Result<(), CheckErrors> {
        let (stdlib_sym, stdlib_macros, stdlib_types) = stdlib_loaded();
        let forms = crate::parse_all!(src).expect("parse ok");
        let mut macros = stdlib_macros.clone();
        let rest = register_defmacros(forms, &mut macros).expect("register macros");
        let expanded = expand_all(
            rest,
            &mut macros,
            &Environment::new(),
            stdlib_sym,
        )
        .expect("expand");
        // Register user-defined type decls (struct / enum / newtype /
        // typealias) into a fresh clone of the stdlib type env. Mirrors
        // production startup: register_stdlib_types for stdlib, then
        // register_types for user source.
        let mut types = stdlib_types.clone();
        let rest_post_types =
            register_types(expanded, &mut types).expect("register user types");
        let mut sym = stdlib_sym.clone();
        let rest = register_defines(rest_post_types, &mut sym).expect("register defines");
        check_program(&rest, &sym, &types)
    }

    // ─── Arity checking ─────────────────────────────────────────────────

    #[test]
    fn correct_arity_passes() {
        assert!(check("(:wat::core::i64::+ 1 2)").is_ok());
        assert!(check("(:wat::core::not true)").is_ok());
        assert!(check("(:wat::holon::Bind (:wat::holon::Atom 1) (:wat::holon::Atom 2))").is_ok());
    }

    #[test]
    fn too_few_args_rejected() {
        let err = check("(:wat::core::i64::+ 1)").unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, CheckError::ArityMismatch { expected: 2, got: 1, .. })));
    }

    #[test]
    fn too_many_args_rejected() {
        let err = check("(:wat::core::not true false)").unwrap_err();
        assert!(err
            .0
            .iter()
            .any(|e| matches!(e, CheckError::ArityMismatch { expected: 1, got: 2, .. })));
    }

    // ─── Monomorphic type mismatch ──────────────────────────────────────

    #[test]
    fn string_to_add_rejected() {
        let err = check(r#"(:wat::core::i64::+ "hello" 3)"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    /// Arc 138 slice 1 — every CheckError surfaced on user source carries
    /// a span. The rendered Display output names file:line:col so a
    /// human or agent reading the error can navigate to the offending
    /// form without grepping. Canary for the project-wide "errors
    /// carry coordinates" doctrine.
    #[test]
    fn type_mismatch_message_carries_span() {
        let err = check(r#"(:wat::core::i64::+ "hello" 3)"#).unwrap_err();
        let rendered = format!("{}", err);
        assert!(
            rendered.contains("src/") || rendered.contains(".rs:"),
            "TypeMismatch Display must include real source coordinates; rendered:\n{}",
            rendered
        );
    }

    /// Arc 140 slice 2 — sandbox-scope leak fires when a deftest body
    /// invokes a name registered at the OUTER test-file scope but NOT
    /// in the deftest's prelude. The diagnostic carries two spans —
    /// the offending invocation AND the outer-scope define — so users
    /// navigate to both sites without grepping. Substrate-as-teacher
    /// pattern; the failure is the user's nudge.
    #[test]
    fn sandbox_scope_leak_fires_with_diagnostic() {
        // Top-level define :my::helper. Deftest body invokes it
        // without including it in the prelude (the empty `()` second
        // argument). The sub-program's scope at runtime would NOT
        // contain :my::helper — sandbox isolation. Arc 140's check
        // catches this at outer freeze.
        let src = r#"
            (:wat::core::define
              (:my::helper (x :wat::core::i64) -> :wat::core::i64)
              (:wat::core::i64::* x 2))

            (:wat::test::deftest :test::leaky
              ()
              (:wat::test::assert-eq (:my::helper 21) 42))
        "#;
        let err = check(src).unwrap_err();
        let rendered = format!("{}", err);
        // Hard guarantees the diagnostic must satisfy:
        assert!(
            err.0.iter().any(|e| matches!(e, CheckError::SandboxScopeLeak { .. })),
            "expected SandboxScopeLeak; rendered:\n{}", rendered
        );
        assert!(
            rendered.contains("sandbox-scope leak"),
            "rendered must contain 'sandbox-scope leak'; got:\n{}", rendered
        );
        assert!(
            rendered.contains(":my::helper"),
            "rendered must name the offending function; got:\n{}", rendered
        );
        assert!(
            rendered.contains("src/") || rendered.contains(".rs:"),
            "rendered must include file:line:col coordinates (arc 138); got:\n{}", rendered
        );
        assert!(
            rendered.contains("prelude"),
            "rendered must teach the user to move the define into the prelude; got:\n{}", rendered
        );
    }

    /// Arc 140 slice 2 — confirm the leak rule does NOT misfire when
    /// the helper IS properly placed in the deftest's prelude. Same
    /// shape as the leak test; helper moved into prelude position; no
    /// SandboxScopeLeak fires.
    #[test]
    fn sandbox_scope_no_leak_when_in_prelude() {
        let src = r#"
            (:wat::test::deftest :test::clean
              ((:wat::core::define
                 (:my::helper (x :wat::core::i64) -> :wat::core::i64)
                 (:wat::core::i64::* x 2)))
              (:wat::test::assert-eq (:my::helper 21) 42))
        "#;
        // No outer-scope :my::helper define exists; the only define
        // for it lives inside the deftest's prelude. The walker must
        // NOT fire SandboxScopeLeak.
        let result = check(src);
        if let Err(errs) = &result {
            assert!(
                !errs.0.iter().any(|e| matches!(e, CheckError::SandboxScopeLeak { .. })),
                "SandboxScopeLeak misfired; rendered:\n{}", errs
            );
        }
        // (Non-leak errors may still appear from other check rules
        // here — the test only asserts SandboxScopeLeak doesn't fire.)
    }

    #[test]
    fn bool_to_add_rejected() {
        let err = check("(:wat::core::i64::+ true 3)").unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn bind_non_holon_rejected() {
        let err = check("(:wat::holon::Bind 42 (:wat::holon::Atom 1))").unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── Polymorphic comparison (T -> T -> bool) ────────────────────────

    #[test]
    fn equality_same_type_passes() {
        assert!(check("(:wat::core::= 1 2)").is_ok());
        assert!(check(r#"(:wat::core::= "a" "b")"#).is_ok());
        assert!(check("(:wat::core::= true false)").is_ok());
    }

    #[test]
    fn equality_mixed_types_rejected() {
        let err = check(r#"(:wat::core::= 1 "x")"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn less_than_mixed_types_rejected() {
        let err = check(r#"(:wat::core::< 1 "x")"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── Polymorphic list (T* -> List<T>) ───────────────────────────────

    #[test]
    fn list_same_type_passes() {
        assert!(check("(:wat::core::Vector :wat::core::i64 1 2 3)").is_ok());
        assert!(check(r#"(:wat::core::Vector :wat::core::String "a" "b")"#).is_ok());
    }

    #[test]
    fn list_mixed_types_rejected() {
        let err = check(r#"(:wat::core::Vector :wat::core::i64 1 "two" 3)"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn bundle_of_list_of_holons_passes() {
        // Bundle takes :wat::holon::Holons. A list of (Atom ...) calls
        // returns :wat::holon::Holons, so Bundle(list(Atoms...)) type-checks.
        assert!(check(
            r#"(:wat::holon::Bundle (:wat::core::Vector :wat::holon::HolonAST
                 (:wat::holon::Atom 1)
                 (:wat::holon::Atom 2)))"#
        )
        .is_ok());
    }

    #[test]
    fn bundle_of_list_of_ints_rejected() {
        // Bundle wants :wat::holon::Holons, but this is :wat::core::Vector<wat::core::i64>.
        let err = check(r#"(:wat::holon::Bundle (:wat::core::Vector :wat::core::i64 1 2 3))"#).unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // LocalCache / :rust::lru::LruCache check tests retired in
    // arc 013 slice 4b — the wat-lru crate owns that surface now.
    // Equivalent check coverage lives in
    // crates/wat-lru/tests/wat_lru_tests.rs, exercised end-to-end
    // via wat::Harness::from_source_with_deps with the dep wiring.

    // Wrong-key-type rejection was enforced by the hand-written lru
    // shim's scheme via unification of call-site K with the cache's
    // declared K. The macro-regenerated shim's Rust signature uses
    // `Value` (not K) for the key arg — the scheme sees Value and
    // unifies trivially. Lands when the macro gets a per-arg type
    // hint (e.g. `#[wat_param = "K"]`). Tracked informally; not
    // blocking lru regeneration correctness because runtime
    // canonicalization still enforces primitive-key at dispatch time.

    #[test]
    fn rust_unknown_symbol_rejected() {
        let err = check("(:rust::imaginary::Crate::method 1 2)").unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::UnknownCallee { .. })));
    }

    // ─── User define signature checks ───────────────────────────────────

    #[test]
    fn user_define_body_matches_signature() {
        assert!(check(
            r#"(:wat::core::define (:my::app::add (x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64)
                 (:wat::core::i64::+ x y))"#
        )
        .is_ok());
    }

    #[test]
    fn user_define_body_wrong_return_rejected() {
        let err = check(
            r#"(:wat::core::define (:my::app::add (x :i64) (y :i64) -> :bool)
                 (:wat::core::i64::+ x y))"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::ReturnTypeMismatch { .. })));
    }

    #[test]
    fn user_parametric_define_passes() {
        // Identity: ∀T. T -> T. Body returns x, which has type T.
        // With rigid type variables, x: T unifies with ret: T.
        assert!(check(
            r#"(:wat::core::define (:my::app::id<T> (x :T) -> :T) x)"#
        )
        .is_ok());
    }

    #[test]
    fn user_parametric_wrong_return_rejected() {
        // Declared ret T; body returns an :i64 constant. Rigid T
        // doesn't unify with :i64.
        let err = check(
            r#"(:wat::core::define (:my::app::bad<T> (x :T) -> :T) 42)"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::ReturnTypeMismatch { .. })));
    }

    // ─── Typed-let discipline ───────────────────────────────────────────

    #[test]
    fn typed_let_binding_matches_rhs() {
        assert!(check(
            r#"(:wat::core::let (((x :wat::core::i64) 42)) (:wat::core::i64::+ x 1))"#
        )
        .is_ok());
    }

    #[test]
    fn typed_let_binding_wrong_type_rejected() {
        // Declared :i64 but RHS is :String — unification fails.
        let err = check(
            r#"(:wat::core::let (((x :i64) "hello")) x)"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    #[test]
    fn typed_let_binding_multiple() {
        assert!(check(
            r#"(:wat::core::let
                 (((x :wat::core::i64) 1)
                  ((y :wat::core::i64) 2)
                  ((z :wat::core::i64) 3))
                 (:wat::core::i64::+ (:wat::core::i64::+ x y) z))"#
        )
        .is_ok());
    }

    #[test]
    fn typed_let_binding_with_lambda_value() {
        // A lambda bound to a let with :fn(wat::core::i64)->wat::core::i64
        // declaration. Declared type matches lambda's own signature, so it
        // passes.
        assert!(check(
            r#"(:wat::core::let
                 (((doubler :fn(wat::core::i64)->wat::core::i64)
                   (:wat::core::lambda ((x :wat::core::i64) -> :wat::core::i64)
                     (:wat::core::i64::+ x x))))
                 true)"#
        )
        .is_ok());
    }

    #[test]
    fn typed_let_binding_lambda_declared_wrong_rejected() {
        // Declared :fn(i64)->bool but lambda produces :fn(i64)->i64.
        let err = check(
            r#"(:wat::core::let
                 (((f :fn(i64)->bool)
                   (:wat::core::lambda ((x :i64) -> :i64) x)))
                 true)"#,
        )
        .unwrap_err();
        assert!(err.0.iter().any(|e| matches!(e, CheckError::TypeMismatch { .. })));
    }

    // ─── :Any ban ───────────────────────────────────────────────────────

    #[test]
    fn any_as_param_type_rejected_at_parse() {
        // Parsing `:Any` in any position is an error.
        let err = parse_type_expr(":Any").unwrap_err();
        assert!(matches!(err, crate::types::TypeError::AnyBanned { .. }));
    }

    #[test]
    fn any_as_parametric_head_rejected_at_parse() {
        let err = parse_type_expr(":Any<i64>").unwrap_err();
        assert!(matches!(err, crate::types::TypeError::AnyBanned { .. }));
    }

    #[test]
    fn any_as_nested_arg_rejected_at_parse() {
        let err = parse_type_expr(":Vec<Any>").unwrap_err();
        assert!(matches!(err, crate::types::TypeError::AnyBanned { .. }));
    }

    #[test]
    fn any_in_fn_rejected_at_parse() {
        let err = parse_type_expr(":fn(Any)->i64").unwrap_err();
        assert!(matches!(err, crate::types::TypeError::AnyBanned { .. }));
    }


    // ─── Multiple errors reported together ──────────────────────────────

    #[test]
    fn multiple_errors_reported() {
        let err = check(r#"(:wat::core::i64::+ "s" 1) (:wat::core::not 42)"#).unwrap_err();
        assert!(err.0.len() >= 2, "expected >=2 errors, got {}", err.0.len());
    }

    // ─── Unification directly ───────────────────────────────────────────

    #[test]
    fn unify_identical_paths() {
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":i64".into()),
            &TypeExpr::Path(":i64".into()),
            &mut s,
            &TypeEnv::with_builtins(),
        )
        .is_ok());
    }

    #[test]
    fn unify_distinct_paths_fails() {
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":i64".into()),
            &TypeExpr::Path(":f64".into()),
            &mut s,
            &TypeEnv::with_builtins(),
        )
        .is_err());
    }

    #[test]
    fn unify_rigid_vars_require_same_name() {
        // Rigid Path(":T") only unifies with Path(":T").
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":T".into()),
            &TypeExpr::Path(":T".into()),
            &mut s,
            &TypeEnv::with_builtins(),
        )
        .is_ok());
        let mut s = Subst::new();
        assert!(unify(
            &TypeExpr::Path(":T".into()),
            &TypeExpr::Path(":U".into()),
            &mut s,
            &TypeEnv::with_builtins(),
        )
        .is_err());
    }

    #[test]
    fn unify_fresh_var_binds_to_concrete() {
        let mut s = Subst::new();
        let var = TypeExpr::Var(0);
        let concrete = TypeExpr::Path(":i64".into());
        unify(&var, &concrete, &mut s, &TypeEnv::with_builtins()).expect("unify");
        assert_eq!(apply_subst(&var, &s), concrete);
    }

    #[test]
    fn unify_parametric_head_must_match() {
        // Different parametric heads must NOT unify: Vec<i64> vs Option<i64>.
        let mut s = Subst::new();
        let vec_int = TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![TypeExpr::Path(":i64".into())],
        };
        let option_int = TypeExpr::Parametric {
            head: "Option".into(),
            args: vec![TypeExpr::Path(":i64".into())],
        };
        assert!(unify(&vec_int, &option_int, &mut s, &TypeEnv::with_builtins()).is_err());
    }

    #[test]
    fn unify_fn_types() {
        let mut s = Subst::new();
        let f1 = TypeExpr::Fn {
            args: vec![TypeExpr::Path(":i64".into())],
            ret: Box::new(TypeExpr::Path(":bool".into())),
        };
        let f2 = TypeExpr::Fn {
            args: vec![TypeExpr::Path(":i64".into())],
            ret: Box::new(TypeExpr::Path(":bool".into())),
        };
        assert!(unify(&f1, &f2, &mut s, &TypeEnv::with_builtins()).is_ok());
    }

    #[test]
    fn occurs_check_rejects_cycle() {
        let mut s = Subst::new();
        // α = List<α>  — would produce an infinite type.
        let cyclic = TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![TypeExpr::Var(0)],
        };
        assert!(unify(&TypeExpr::Var(0), &cyclic, &mut s, &TypeEnv::with_builtins()).is_err());
    }

    // ─── Parse + unify round-trip ───────────────────────────────────────

    #[test]
    fn type_expr_parse_and_unify() {
        let mut s = Subst::new();
        let a = parse_type_expr(":wat::holon::HolonAST").unwrap();
        let b = parse_type_expr(":wat::holon::HolonAST").unwrap();
        assert!(unify(&a, &b, &mut s, &TypeEnv::with_builtins()).is_ok());
    }

    // ─── Parametric user-defined enum match patterns ────────────────────
    //
    // First parametric user-defined enum surfaced by arc 119
    // (`:wat::lru::Request<K,V>`). Coverage gap before arc 119: zero
    // parametric user enums existed in the codebase, so the
    // match-pattern resolver's path that drops type params
    // (`MatchShape::Enum(name)` → `TypeExpr::Path(name)`) was never
    // exercised. `:wat::core::Option<T>` and `:wat::core::Result<T,E>`
    // bypass the gap via dedicated MatchShape variants
    // (`Option(t)`, `Result(t,e)`) that carry their type args directly.
    //
    // These tests cover the gap: a parametric user enum with tagged
    // and unit variants must type-check when matched against a
    // scrutinee whose declared type carries the type arguments.

    #[test]
    fn parametric_user_enum_tagged_variant_match() {
        // Single type param, tagged variants. Mirrors the minimal
        // bug repro: enum decl carries `<T>`, function param carries
        // `:my::Box<T>`, match patterns reference variants under the
        // bare `:my::Box::*` prefix.
        let src = r#"
            (:wat::core::enum :my::Box<T>
              (Empty)
              (Filled (value :T)))

            (:wat::core::define
              (:my::is-empty<T> (b :my::Box<T>) -> :wat::core::bool)
              (:wat::core::match b -> :wat::core::bool
                ((:my::Box::Empty) true)
                ((:my::Box::Filled _v) false)))
        "#;
        let result = check(src);
        assert!(
            result.is_ok(),
            "parametric enum tagged-variant match should type-check, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn parametric_user_enum_two_type_args_match() {
        // Two type params, tagged variants. Mirrors arc 119's
        // `:wat::lru::Request<K,V>` shape directly.
        let src = r#"
            (:wat::core::enum :my::Either<L,R>
              (Left (value :L))
              (Right (value :R)))

            (:wat::core::define
              (:my::is-left<L,R> (e :my::Either<L,R>) -> :wat::core::bool)
              (:wat::core::match e -> :wat::core::bool
                ((:my::Either::Left _v) true)
                ((:my::Either::Right _v) false)))
        "#;
        let result = check(src);
        assert!(
            result.is_ok(),
            "parametric enum two-type-args match should type-check, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn parametric_user_enum_extracts_typed_field() {
        // Tagged variant binder must inherit the parametric type's
        // instantiation. If the scrutinee is :my::Box<i64>, then
        // (Filled v) must bind v as :i64.
        let src = r#"
            (:wat::core::enum :my::Box<T>
              (Empty)
              (Filled (value :T)))

            (:wat::core::define
              (:my::default-or<T> (b :my::Box<T>) (d :T) -> :T)
              (:wat::core::match b -> :T
                ((:my::Box::Empty) d)
                ((:my::Box::Filled v) v)))
        "#;
        let result = check(src);
        assert!(
            result.is_ok(),
            "parametric enum extracts-typed-field should type-check, got: {:?}",
            result.err()
        );
    }

    // ─── Arc 128 — check walker respects sandbox boundary ────────────

    /// Arc 128 — the scope-deadlock anti-pattern at the OUTER scope
    /// (no surrounding sandbox call) MUST still fire `ScopeDeadlock`.
    /// Verifies arc 117's check survives arc 128's boundary addition.
    #[test]
    fn arc_128_outer_scope_deadlock_still_fires() {
        let src = r#"
            (:wat::core::define
              (:my::deadlock-at-outer -> :wat::core::unit)
              (:wat::core::let*
                (((pair :wat::kernel::Channel<wat::core::i64>)
                  (:wat::kernel::make-bounded-channel :wat::core::i64 1))
                 ((rx :wat::kernel::Receiver<wat::core::i64>)
                  (:wat::core::second pair))
                 ((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
                  (:wat::kernel::spawn-thread
                    (:wat::core::lambda
                      ((_in :wat::kernel::Receiver<wat::core::unit>)
                       (_out :wat::kernel::Sender<wat::core::i64>)
                       -> :wat::core::unit)
                      (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::unit
                        ((:wat::core::Ok _) ())
                        ((:wat::core::Err _) ()))))))
                (:wat::core::match
                  (:wat::kernel::Thread/join-result thr)
                  -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))
        "#;
        let err = check(src).expect_err(
            "outer-scope deadlock pattern must still fire ScopeDeadlock post-arc-128",
        );
        assert!(
            err.0
                .iter()
                .any(|e| matches!(e, CheckError::ScopeDeadlock { .. })),
            "expected ScopeDeadlock at outer scope (arc 117 still active), got: {:?}",
            err.0
        );
    }

    /// Arc 128 — the SAME scope-deadlock anti-pattern, when nested
    /// inside the forms-block of a `run-sandboxed-hermetic-ast` call,
    /// MUST NOT fire `ScopeDeadlock` at outer freeze. The inner program
    /// has its own freeze cycle at runtime; outer walker stops at the
    /// sandbox boundary.
    #[test]
    fn arc_128_inner_scope_deadlock_skipped_in_sandboxed_forms() {
        let src = r#"
            (:wat::core::define
              (:my::deftest-style -> :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-hermetic-ast
                (:wat::core::forms
                  (:wat::core::define
                    (:user::main
                      (_stdin  :wat::io::IOReader)
                      (_stdout :wat::io::IOWriter)
                      (_stderr :wat::io::IOWriter)
                      -> :wat::core::unit)
                    (:wat::core::let*
                      (((pair :wat::kernel::Channel<wat::core::i64>)
                        (:wat::kernel::make-bounded-channel :wat::core::i64 1))
                       ((rx :wat::kernel::Receiver<wat::core::i64>)
                        (:wat::core::second pair))
                       ((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
                        (:wat::kernel::spawn-thread
                          (:wat::core::lambda
                            ((_in :wat::kernel::Receiver<wat::core::unit>)
                             (_out :wat::kernel::Sender<wat::core::i64>)
                             -> :wat::core::unit)
                            (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::unit
                              ((:wat::core::Ok _) ())
                              ((:wat::core::Err _) ()))))))
                      (:wat::core::match
                        (:wat::kernel::Thread/join-result thr)
                        -> :wat::core::unit
                        ((:wat::core::Ok _) ())
                        ((:wat::core::Err _) ())))))
                (:wat::core::Vector :wat::core::String)
                :wat::core::None))
        "#;
        let result = check(src);
        // The outer freeze must NOT see the inner anti-pattern.
        // ScopeDeadlock errors firing here would mean the walker is
        // still descending into the forms block — arc 128 not in
        // effect.
        match result {
            Ok(_) => {}
            Err(errors) => {
                let scope_deadlocks: Vec<_> = errors
                    .0
                    .iter()
                    .filter(|e| matches!(e, CheckError::ScopeDeadlock { .. }))
                    .collect();
                assert!(
                    scope_deadlocks.is_empty(),
                    "arc 128: walker must skip sandboxed forms block; ScopeDeadlock fired at outer freeze: {:?}",
                    scope_deadlocks
                );
            }
        }
    }

    // ─── Arc 126 — channel-pair-deadlock prevention ─────────────────

    #[test]
    fn channel_pair_deadlock_fires_on_canonical_anti_pattern() {
        // The arc 119 minimal repro shape: one make-bounded-channel
        // pair, both halves bound, both passed to one helper-verb.
        // Structural truth — same pair-anchor IS same channel — so
        // the rule fires regardless of helper-verb's body.
        let src = r#"
            (:wat::core::define
              (:my::helper-verb
                (tx :wat::kernel::Sender<wat::core::unit>)
                (rx :wat::kernel::Receiver<wat::core::unit>)
                -> :wat::core::unit)
              ())

            (:wat::core::define
              (:my::caller (_d :wat::core::unit) -> :wat::core::unit)
              (:wat::core::let*
                (((pair :wat::kernel::Channel<wat::core::unit>)
                  (:wat::kernel::make-bounded-channel :wat::core::unit 1))
                 ((tx :wat::kernel::Sender<wat::core::unit>)
                  (:wat::core::first pair))
                 ((rx :wat::kernel::Receiver<wat::core::unit>)
                  (:wat::core::second pair)))
                (:my::helper-verb tx rx)))
        "#;
        let err = check(src).unwrap_err();
        assert!(
            err.0.iter().any(|e| matches!(
                e,
                CheckError::ChannelPairDeadlock { callee, sender_arg, receiver_arg, pair_anchor, .. }
                    if callee == ":my::helper-verb"
                        && sender_arg == "tx"
                        && receiver_arg == "rx"
                        && pair_anchor == "pair"
            )),
            "expected ChannelPairDeadlock on canonical anti-pattern, got: {:?}",
            err.0
        );
    }

    #[test]
    fn channel_pair_deadlock_silent_on_two_different_pairs() {
        // Two SEPARATE make-bounded-channel calls — each end traces
        // to a distinct pair-anchor. Different anchors = different
        // channels = no deadlock. The rule must NOT fire.
        let src = r#"
            (:wat::core::define
              (:my::helper-verb
                (tx :wat::kernel::Sender<wat::core::unit>)
                (rx :wat::kernel::Receiver<wat::core::unit>)
                -> :wat::core::unit)
              ())

            (:wat::core::define
              (:my::caller (_d :wat::core::unit) -> :wat::core::unit)
              (:wat::core::let*
                (((pair-a :wat::kernel::Channel<wat::core::unit>)
                  (:wat::kernel::make-bounded-channel :wat::core::unit 1))
                 ((pair-b :wat::kernel::Channel<wat::core::unit>)
                  (:wat::kernel::make-bounded-channel :wat::core::unit 1))
                 ((tx :wat::kernel::Sender<wat::core::unit>)
                  (:wat::core::first pair-a))
                 ((rx :wat::kernel::Receiver<wat::core::unit>)
                  (:wat::core::second pair-b)))
                (:my::helper-verb tx rx)))
        "#;
        let result = check(src);
        // Two-different-pairs is a legitimate shape (the arc 126
        // canonical fix); no ChannelPairDeadlock should fire.
        assert!(
            result.as_ref().is_ok()
                || result
                    .as_ref()
                    .err()
                    .map(|e| {
                        !e.0.iter()
                            .any(|err| matches!(err, CheckError::ChannelPairDeadlock { .. }))
                    })
                    .unwrap_or(true),
            "two different pairs should not fire ChannelPairDeadlock, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn channel_pair_deadlock_silent_on_canonical_handle_pop() {
        // The substrate's canonical pair-by-index pattern: a Handle
        // is `(ReqTx, AckRx)` where ReqTx is one end of the request
        // channel and AckRx is one end of the SEPARATE ack channel.
        // The trace gives up at the Handle (its RHS isn't a
        // make-channel call); no anchor resolves; no fire. This is
        // the false-negative-by-design that ZERO-MUTEX.md § "Routing
        // acks" relies on — the pair-by-index pattern routes both
        // ends but holds them at separate-anchor positions.
        let src = r#"
            (:wat::core::typealias :my::Handle
              :(wat::kernel::Sender<wat::core::unit>,wat::kernel::Receiver<wat::core::unit>))

            (:wat::core::define
              (:my::pop-handle (_d :wat::core::unit) -> :my::Handle)
              (:wat::core::let*
                (((p :wat::kernel::Channel<wat::core::unit>)
                  (:wat::kernel::make-bounded-channel :wat::core::unit 1))
                 ((q :wat::kernel::Channel<wat::core::unit>)
                  (:wat::kernel::make-bounded-channel :wat::core::unit 1))
                 ((req-tx :wat::kernel::Sender<wat::core::unit>)
                  (:wat::core::first p))
                 ((ack-rx :wat::kernel::Receiver<wat::core::unit>)
                  (:wat::core::second q)))
                (:wat::core::Tuple req-tx ack-rx)))

            (:wat::core::define
              (:my::helper-verb
                (tx :wat::kernel::Sender<wat::core::unit>)
                (rx :wat::kernel::Receiver<wat::core::unit>)
                -> :wat::core::unit)
              ())

            (:wat::core::define
              (:my::caller (_d :wat::core::unit) -> :wat::core::unit)
              (:wat::core::let*
                (((handle :my::Handle) (:my::pop-handle))
                 ((req-tx :wat::kernel::Sender<wat::core::unit>)
                  (:wat::core::first handle))
                 ((ack-rx :wat::kernel::Receiver<wat::core::unit>)
                  (:wat::core::second handle)))
                (:my::helper-verb req-tx ack-rx)))
        "#;
        let result = check(src);
        assert!(
            result.as_ref().is_ok()
                || result
                    .as_ref()
                    .err()
                    .map(|e| {
                        !e.0.iter()
                            .any(|err| matches!(err, CheckError::ChannelPairDeadlock { .. }))
                    })
                    .unwrap_or(true),
            "HandlePool-pop pattern should not fire ChannelPairDeadlock (trace gives up at user fn boundary), got: {:?}",
            result.err()
        );
    }

    #[test]
    fn channel_pair_deadlock_diagnostic_substring() {
        // Slice 2 of arc 126 converts 6 :ignore'd test sites to
        // :should-panic with a substring match. This test locks the
        // contract: the Display impl MUST emit "channel-pair-deadlock"
        // verbatim. Divergent phrasing breaks the verification chain.
        let src = r#"
            (:wat::core::define
              (:my::helper-verb
                (tx :wat::kernel::Sender<wat::core::unit>)
                (rx :wat::kernel::Receiver<wat::core::unit>)
                -> :wat::core::unit)
              ())

            (:wat::core::define
              (:my::caller (_d :wat::core::unit) -> :wat::core::unit)
              (:wat::core::let*
                (((pair :wat::kernel::Channel<wat::core::unit>)
                  (:wat::kernel::make-bounded-channel :wat::core::unit 1))
                 ((tx :wat::kernel::Sender<wat::core::unit>)
                  (:wat::core::first pair))
                 ((rx :wat::kernel::Receiver<wat::core::unit>)
                  (:wat::core::second pair)))
                (:my::helper-verb tx rx)))
        "#;
        let err = check(src).unwrap_err();
        let pair_err = err
            .0
            .iter()
            .find(|e| matches!(e, CheckError::ChannelPairDeadlock { .. }))
            .expect("ChannelPairDeadlock variant present");
        let display = format!("{}", pair_err);
        assert!(
            display.contains("channel-pair-deadlock"),
            "Display impl missing load-bearing 'channel-pair-deadlock' substring; got: {}",
            display
        );
    }

    /// Arc 126 RELAND — the channel-pair-deadlock anti-pattern, when
    /// nested inside the forms-block of a `run-sandboxed-hermetic-ast`
    /// call, MUST NOT fire `ChannelPairDeadlock` at outer freeze. The
    /// inner program has its own freeze cycle at runtime; outer walker
    /// stops at the sandbox boundary. Mirrors arc 128's
    /// `arc_128_inner_scope_deadlock_skipped_in_sandboxed_forms` test
    /// for the new arc-126 check.
    #[test]
    fn channel_pair_deadlock_skipped_in_sandboxed_forms() {
        let src = r#"
            (:wat::core::define
              (:my::deftest-style -> :wat::kernel::RunResult)
              (:wat::kernel::run-sandboxed-hermetic-ast
                (:wat::core::forms
                  (:wat::core::define
                    (:my::helper-verb
                      (tx :wat::kernel::Sender<wat::core::unit>)
                      (rx :wat::kernel::Receiver<wat::core::unit>)
                      -> :wat::core::unit)
                    ())
                  (:wat::core::define
                    (:user::main
                      (_stdin  :wat::io::IOReader)
                      (_stdout :wat::io::IOWriter)
                      (_stderr :wat::io::IOWriter)
                      -> :wat::core::unit)
                    (:wat::core::let*
                      (((pair :wat::kernel::Channel<wat::core::unit>)
                        (:wat::kernel::make-bounded-channel :wat::core::unit 1))
                       ((tx :wat::kernel::Sender<wat::core::unit>)
                        (:wat::core::first pair))
                       ((rx :wat::kernel::Receiver<wat::core::unit>)
                        (:wat::core::second pair)))
                      (:my::helper-verb tx rx))))
                (:wat::core::Vector :wat::core::String)
                :wat::core::None))
        "#;
        let result = check(src);
        // The outer freeze must NOT see the inner anti-pattern.
        // ChannelPairDeadlock errors firing here would mean the
        // walker is descending into the forms block — arc 128
        // boundary not inherited by arc 126's walker.
        match result {
            Ok(_) => {}
            Err(errors) => {
                let pair_deadlocks: Vec<_> = errors
                    .0
                    .iter()
                    .filter(|e| matches!(e, CheckError::ChannelPairDeadlock { .. }))
                    .collect();
                assert!(
                    pair_deadlocks.is_empty(),
                    "arc 126 reland: walker must skip sandboxed forms block; ChannelPairDeadlock fired at outer freeze: {:?}",
                    pair_deadlocks
                );
            }
        }
    }

    // ─── Arc 131 — HandlePool counts as Sender-bearing ──────────────

    /// Arc 131 — a `let*` binding-block containing a HandlePool
    /// whose T (after alias resolution) carries a Sender, sibling
    /// to a Thread that gets `Thread/join-result`'d in body
    /// position, MUST fire `ScopeDeadlock` with offending_kind
    /// "HandlePool". The canonical service-test mistake: the
    /// pool's internal entries hold Sender clones that outlive
    /// the worker's recv loop. Closes the "future arc" caveat
    /// arc 117's source comment named.
    #[test]
    fn arc_131_handlepool_with_sender_fires() {
        // Direct shape: `HandlePool<Sender<i64>>` — no user typealias
        // needed. The new arm recurses into args; finds a Sender
        // structurally; returns Some("HandlePool"). Models the
        // canonical service-test mistake without the syntactic noise
        // of a parametric typealias declaration.
        //
        // The lambda body has a `(recv _in)` so arc 134's body-form
        // narrowing (no-recv → exempt) does NOT apply — the canonical
        // deadlock-prone shape requires the thread to actually have a
        // recv on its input. Without recv there is no recv-loop to
        // hang, no deadlock, and the rule should not fire.
        let src = r#"
            (:wat::core::define
              (:my::deadlock-via-handlepool -> :wat::core::unit)
              (:wat::core::let*
                (((pool :wat::kernel::HandlePool<wat::kernel::Sender<wat::core::i64>>)
                  (:wat::kernel::HandlePool::new
                    "pool"
                    (:wat::core::Vector :wat::kernel::Sender<wat::core::i64>)))
                 ((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
                  (:wat::kernel::spawn-thread
                    (:wat::core::lambda
                      ((_in :wat::kernel::Receiver<wat::core::unit>)
                       (_out :wat::kernel::Sender<wat::core::i64>)
                       -> :wat::core::unit)
                      (:wat::core::match (:wat::kernel::recv _in)
                        -> :wat::core::unit
                        ((:wat::core::Ok _) ())
                        ((:wat::core::Err _) ()))))))
                (:wat::core::match
                  (:wat::kernel::Thread/join-result thr)
                  -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))
        "#;
        let err = check(src).expect_err(
            "HandlePool<HandleAlias-with-Sender> sibling to Thread/join-result must fire ScopeDeadlock",
        );
        let scope_deadlocks: Vec<_> = err
            .0
            .iter()
            .filter_map(|e| match e {
                CheckError::ScopeDeadlock { offending_kind, .. } => Some(*offending_kind),
                _ => None,
            })
            .collect();
        assert!(
            scope_deadlocks.iter().any(|k| *k == "HandlePool"),
            "expected ScopeDeadlock with offending_kind=\"HandlePool\", got kinds: {:?}; full errors: {:?}",
            scope_deadlocks,
            err.0
        );
    }

    /// Arc 131 — `HandlePool<T>` where T does NOT contain a Sender
    /// (e.g. `HandlePool<i64>`) is not deadlock-prone; the new
    /// surface arm passes through silently. Confirms the rule
    /// fires structurally on Sender-presence, not on HandlePool
    /// per se.
    #[test]
    fn arc_131_handlepool_without_sender_silent() {
        let src = r#"
            (:wat::core::define
              (:my::no-deadlock-on-bare-handlepool -> :wat::core::unit)
              (:wat::core::let*
                (((pool :wat::kernel::HandlePool<wat::core::i64>)
                  (:wat::kernel::HandlePool::new
                    "pool"
                    (:wat::core::Vector :wat::core::i64)))
                 ((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
                  (:wat::kernel::spawn-thread
                    (:wat::core::lambda
                      ((_in :wat::kernel::Receiver<wat::core::unit>)
                       (_out :wat::kernel::Sender<wat::core::i64>)
                       -> :wat::core::unit)
                      ()))))
                (:wat::core::match
                  (:wat::kernel::Thread/join-result thr)
                  -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))
        "#;
        let result = check(src);
        // HandlePool<i64> has no embedded Sender — the rule must
        // pass through. ScopeDeadlock firing here would mean the
        // new arm flagged HandlePool unconditionally.
        match result {
            Ok(_) => {}
            Err(errors) => {
                let scope_deadlocks: Vec<_> = errors
                    .0
                    .iter()
                    .filter(|e| matches!(e, CheckError::ScopeDeadlock { .. }))
                    .collect();
                assert!(
                    scope_deadlocks.is_empty(),
                    "arc 131: HandlePool<i64> must not fire ScopeDeadlock (T contains no Sender); got: {:?}",
                    scope_deadlocks
                );
            }
        }
    }

    // ─── Arc 133 — tuple-destructure bindings honor scope-deadlock checks ───

    /// Arc 133 — regression guard: the existing typed-name binding shape
    /// (arc 117 / arc 131 canonical form) MUST continue to fire
    /// `ScopeDeadlock` after the structural walker is retired and the
    /// inference-time check takes over. The inference-time path uses
    /// `extended` which is populated by `process_let_binding` — for
    /// typed-name bindings, the declared type annotation is used.
    ///
    /// Shape: `((pool :HandlePool<Sender<i64>>) ...)` sibling to
    /// `((thr :Thread<...>) ...)` + `Thread/join-result thr` in body.
    /// Expected: `ScopeDeadlock` fires; `offending_kind = "HandlePool"`.
    #[test]
    fn arc_133_typed_name_binding_still_fires() {
        // Body has `(recv _in)` — the canonical deadlock-prone shape
        // requires the thread to actually call recv. Arc 134's body-
        // form narrowing only exempts no-recv lambda bodies; this
        // test's body is recv-bearing so the rule still fires.
        let src = r#"
            (:wat::core::define
              (:my::typed-name-still-fires -> :wat::core::unit)
              (:wat::core::let*
                (((pool :wat::kernel::HandlePool<wat::kernel::Sender<wat::core::i64>>)
                  (:wat::kernel::HandlePool::new
                    "pool"
                    (:wat::core::Vector :wat::kernel::Sender<wat::core::i64>)))
                 ((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
                  (:wat::kernel::spawn-thread
                    (:wat::core::lambda
                      ((_in :wat::kernel::Receiver<wat::core::unit>)
                       (_out :wat::kernel::Sender<wat::core::i64>)
                       -> :wat::core::unit)
                      (:wat::core::match (:wat::kernel::recv _in)
                        -> :wat::core::unit
                        ((:wat::core::Ok _) ())
                        ((:wat::core::Err _) ()))))))
                (:wat::core::match
                  (:wat::kernel::Thread/join-result thr)
                  -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))
        "#;
        let err = check(src).expect_err(
            "arc 133: typed-name HandlePool sibling to Thread/join-result must still fire ScopeDeadlock after structural walker retirement",
        );
        let scope_deadlocks: Vec<_> = err
            .0
            .iter()
            .filter_map(|e| match e {
                CheckError::ScopeDeadlock { offending_kind, .. } => Some(*offending_kind),
                _ => None,
            })
            .collect();
        assert!(
            scope_deadlocks.iter().any(|k| *k == "HandlePool"),
            "arc 133 regression: expected ScopeDeadlock offending_kind=\"HandlePool\" from typed-name binding; got kinds: {:?}; full errors: {:?}",
            scope_deadlocks,
            err.0
        );
    }

    /// Arc 133 — the new path: tuple-destructure binding
    /// `((pool driver) (some-spawn-fn))` where the spawn returns
    /// `(HandlePool<Sender<i64>>, Thread<unit,i64>)` and
    /// `Thread/join-result driver` appears in the body.
    ///
    /// The inference-time check reads from `extended`, which contains
    /// both `pool → HandlePool<Sender<i64>>` and
    /// `driver → Thread<unit,i64>` after `process_let_binding`
    /// destructures the tuple. Both names are visible to the
    /// classifier; the Sender-bearing HandlePool fires the deadlock.
    ///
    /// The typealias `SpawnResult` is declared to give the RHS a
    /// named return type of tuple shape; `infer_let_star` resolves it
    /// via `process_let_binding`'s destructure path (fresh vars +
    /// unify against tuple).
    ///
    /// Expected: `ScopeDeadlock` fires;
    ///   offending_binding = "pool", offending_kind = "HandlePool".
    #[test]
    fn arc_133_tuple_destructure_with_handlepool_fires() {
        // Define a helper that returns the (HandlePool, Thread) tuple
        // so the destructure binding shape `((pool driver) (spawn-svc))` is
        // exercised. The helper's return type is declared as a tuple
        // so that `process_let_binding`'s destructure arm fires.
        let src = r#"
            (:wat::core::define
              (:my::spawn-svc -> :(wat::kernel::HandlePool<wat::kernel::Sender<wat::core::i64>>,wat::kernel::Thread<wat::core::unit,wat::core::i64>))
              (:wat::core::let*
                (((pool :wat::kernel::HandlePool<wat::kernel::Sender<wat::core::i64>>)
                  (:wat::kernel::HandlePool::new
                    "pool"
                    (:wat::core::Vector :wat::kernel::Sender<wat::core::i64>)))
                 ((driver :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
                  (:wat::kernel::spawn-thread
                    (:wat::core::lambda
                      ((_in :wat::kernel::Receiver<wat::core::unit>)
                       (_out :wat::kernel::Sender<wat::core::i64>)
                       -> :wat::core::unit)
                      ()))))
                (:wat::core::Tuple pool driver)))

            (:wat::core::define
              (:my::caller-via-destructure -> :wat::core::unit)
              (:wat::core::let*
                (((pool driver)
                  (:my::spawn-svc)))
                (:wat::core::match
                  (:wat::kernel::Thread/join-result driver)
                  -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))
        "#;
        let err = check(src).expect_err(
            "arc 133: tuple-destructure of (HandlePool, Thread) with Thread/join-result in body must fire ScopeDeadlock",
        );
        let scope_deadlocks: Vec<_> = err
            .0
            .iter()
            .filter_map(|e| match e {
                CheckError::ScopeDeadlock { offending_binding, offending_kind, .. } => {
                    Some((offending_binding.clone(), *offending_kind))
                }
                _ => None,
            })
            .collect();
        assert!(
            scope_deadlocks
                .iter()
                .any(|(name, kind)| name == "pool" && *kind == "HandlePool"),
            "arc 133: expected ScopeDeadlock offending_binding=\"pool\" offending_kind=\"HandlePool\"; got: {:?}; full errors: {:?}",
            scope_deadlocks,
            err.0
        );
    }

    /// Arc 133 — negative path: tuple-destructure where the RHS
    /// returns `(wat::core::i64, wat::kernel::Thread<unit,unit>)` —
    /// no Sender-bearing element. The inference-time check must pass
    /// through silently.
    ///
    /// Confirms that `check_let_star_for_scope_deadlock_inferred`
    /// classifies on INFERRED TYPE STRUCTURE, not on the presence of
    /// a tuple-destructure pattern per se.
    #[test]
    fn arc_133_tuple_destructure_silent_when_clean() {
        let src = r#"
            (:wat::core::define
              (:my::spawn-clean -> :(wat::core::i64,wat::kernel::Thread<wat::core::unit,wat::core::unit>))
              (:wat::core::let*
                (((counter :wat::core::i64) 42)
                 ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                  (:wat::kernel::spawn-thread
                    (:wat::core::lambda
                      ((_in :wat::kernel::Receiver<wat::core::unit>)
                       (_out :wat::kernel::Sender<wat::core::unit>)
                       -> :wat::core::unit)
                      ()))))
                (:wat::core::tuple counter driver)))

            (:wat::core::define
              (:my::clean-caller -> :wat::core::unit)
              (:wat::core::let*
                (((counter driver)
                  (:my::spawn-clean)))
                (:wat::core::match
                  (:wat::kernel::Thread/join-result driver)
                  -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))
        "#;
        let result = check(src);
        match result {
            Ok(_) => {}
            Err(errors) => {
                let scope_deadlocks: Vec<_> = errors
                    .0
                    .iter()
                    .filter(|e| matches!(e, CheckError::ScopeDeadlock { .. }))
                    .collect();
                assert!(
                    scope_deadlocks.is_empty(),
                    "arc 133: tuple-destructure of (i64, Thread) must not fire ScopeDeadlock (no Sender-bearing element); got: {:?}",
                    scope_deadlocks
                );
            }
        }
    }

    /// Arc 133 — sibling check: the `ChannelPairDeadlock` rule also
    /// fires when both halves of a channel pair are bound via
    /// tuple-destructure `((tx rx) (:wat::kernel::make-bounded-channel :i64 1))`
    /// and then both are passed to one helper function.
    ///
    /// The extension to `walk_for_pair_deadlock` adds synthetic
    /// pair-scope entries for the tuple-destructure binding, with
    /// a shared anchor name. When the helper call receives both `tx`
    /// and `rx`, the trace resolves them to the same anchor →
    /// `ChannelPairDeadlock` fires.
    #[test]
    fn arc_133_tuple_destructure_pair_check_fires() {
        let src = r#"
            (:wat::core::define
              (:my::helper-pair
                (tx :wat::kernel::Sender<wat::core::i64>)
                (rx :wat::kernel::Receiver<wat::core::i64>)
                -> :wat::core::unit)
              ())

            (:wat::core::define
              (:my::caller-destructure (_d :wat::core::unit) -> :wat::core::unit)
              (:wat::core::let*
                (((tx rx)
                  (:wat::kernel::make-bounded-channel :wat::core::i64 1)))
                (:my::helper-pair tx rx)))
        "#;
        let err = check(src).expect_err(
            "arc 133: tuple-destructure of make-bounded-channel with both halves passed to one helper must fire ChannelPairDeadlock",
        );
        let pair_deadlocks: Vec<_> = err
            .0
            .iter()
            .filter(|e| matches!(e, CheckError::ChannelPairDeadlock { .. }))
            .collect();
        assert!(
            !pair_deadlocks.is_empty(),
            "arc 133: expected ChannelPairDeadlock from tuple-destructure of make-bounded-channel; got: {:?}",
            err.0
        );
    }

    /// Arc 134 — origin-trace narrowing. The canonical Thread<I,O>
    /// usage pattern binds a Sender via `(:wat::kernel::Thread/input
    /// thr)` sibling to the Thread itself, then calls
    /// `Thread/join-result thr` in the body. The Sender's
    /// pair-Receiver is the spawned function's `in` parameter, owned
    /// by the Thread struct itself — its lifetime is coupled to the
    /// Thread, not to parent scope. The rule MUST NOT fire on this
    /// shape.
    ///
    /// This test mirrors the canonical Thread/input/output pattern
    /// from `tests/wat_spawn_lambda.rs` (the integration tests that
    /// surfaced the pre-arc-134 false positive) and locks it in as a
    /// regression guard.
    #[test]
    fn arc_134_thread_input_output_does_not_fire() {
        let src = r#"
            (:wat::core::define
              (:my::worker
                (in :wat::kernel::Receiver<wat::core::i64>)
                (out :wat::kernel::Sender<wat::core::i64>)
                -> :wat::core::unit)
              ())

            (:wat::core::define (:my::caller -> :wat::core::unit)
              (:wat::core::let*
                (((thr :wat::kernel::Thread<wat::core::i64,wat::core::i64>)
                  (:wat::kernel::spawn-thread :my::worker))
                 ((tx :wat::kernel::Sender<wat::core::i64>)
                  (:wat::kernel::Thread/input thr))
                 ((rx :wat::kernel::Receiver<wat::core::i64>)
                  (:wat::kernel::Thread/output thr)))
                (:wat::core::match
                  (:wat::kernel::Thread/join-result thr)
                  -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))
        "#;
        let result = check(src);
        match result {
            Ok(_) => {}
            Err(errors) => {
                let scope_deadlocks: Vec<_> = errors
                    .0
                    .iter()
                    .filter(|e| matches!(e, CheckError::ScopeDeadlock { .. }))
                    .collect();
                assert!(
                    scope_deadlocks.is_empty(),
                    "arc 134: Sender from Thread/input <thr> sibling to thr with Thread/join-result thr in body must NOT fire ScopeDeadlock; got: {:?}",
                    scope_deadlocks
                );
            }
        }
    }

    /// Arc 134 — guard rail. The exemption is targeted at Senders
    /// originating from `(:wat::kernel::Thread/input <_>)`. A Sender
    /// from a parent-allocated `(:wat::kernel::make-bounded-channel
    /// ...)` (the canonical deadlock anchor) sibling to a Thread with
    /// `Thread/join-result` in body MUST still fire — that's the
    /// exact shape arc 117 was designed to catch.
    ///
    /// This locks in that arc 134's narrowing does NOT silently weaken
    /// the rule's coverage of the canonical deadlock pattern.
    #[test]
    fn arc_134_parent_allocated_channel_still_fires() {
        let src = r#"
            (:wat::core::define
              (:my::worker
                (in :wat::kernel::Receiver<wat::core::i64>)
                -> :wat::core::unit)
              ())

            (:wat::core::define (:my::caller -> :wat::core::unit)
              (:wat::core::let*
                (((pair :wat::kernel::Channel<wat::core::i64>)
                  (:wat::kernel::make-bounded-channel :wat::core::i64 1))
                 ((tx :wat::kernel::Sender<wat::core::i64>)
                  (:wat::core::first pair))
                 ((rx :wat::kernel::Receiver<wat::core::i64>)
                  (:wat::core::second pair))
                 ((thr :wat::kernel::Thread<wat::core::i64,wat::core::unit>)
                  (:wat::kernel::spawn-thread :my::worker)))
                (:wat::core::match
                  (:wat::kernel::Thread/join-result thr)
                  -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))
        "#;
        let err = check(src).expect_err(
            "arc 134: parent-allocated Sender (from make-bounded-channel) sibling to Thread with join-result MUST still fire ScopeDeadlock — arc 117's canonical anchor",
        );
        let scope_deadlocks: Vec<_> = err
            .0
            .iter()
            .filter(|e| matches!(e, CheckError::ScopeDeadlock { .. }))
            .collect();
        assert!(
            !scope_deadlocks.is_empty(),
            "arc 134: expected ScopeDeadlock from parent-allocated channel sibling to Thread; got: {:?}",
            err.0
        );
    }

    /// Arc 134 — body-form narrowing. The pattern from
    /// `tests/wat_typealias.rs::alias_over_fn_type_works_at_spawn`:
    /// parent allocates a channel via `(make-bounded-channel)` and
    /// captures `tx` (Sender) in the spawn-thread closure. The
    /// closure's body calls a helper that sends, never recvs. The
    /// thread cannot have a recv-loop, so no Sender lifetime can
    /// deadlock it.
    ///
    /// Pre-arc-134 (post-arc-133) this fired ScopeDeadlock on `pair`
    /// + `tx` because the rule only checked type-coexistence with
    /// the Thread, ignoring whether the spawn body actually had a
    /// recv. Arc 134's body-form narrowing walks the inline lambda
    /// body looking for `(:wat::kernel::recv ...)`; absent → exempt.
    ///
    /// Regression guard for the wat_typealias false positive.
    #[test]
    fn arc_134_no_recv_in_lambda_body_does_not_fire() {
        let src = r#"
            (:wat::core::define
              (:my::sender-helper
                (tx :wat::kernel::Sender<wat::core::i64>)
                -> :wat::core::unit)
              (:wat::core::match (:wat::kernel::send tx 7)
                -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) ())))

            (:wat::core::define (:my::caller -> :wat::core::unit)
              (:wat::core::let*
                (((pair :wat::kernel::Channel<wat::core::i64>)
                  (:wat::kernel::make-bounded-channel :wat::core::i64 1))
                 ((tx :wat::kernel::Sender<wat::core::i64>)
                  (:wat::core::first pair))
                 ((rx :wat::kernel::Receiver<wat::core::i64>)
                  (:wat::core::second pair))
                 ((thr :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                  (:wat::kernel::spawn-thread
                    (:wat::core::lambda
                      ((_in :wat::kernel::Receiver<wat::core::unit>)
                       (_out :wat::kernel::Sender<wat::core::unit>)
                       -> :wat::core::unit)
                      (:my::sender-helper tx)))))
                (:wat::core::match
                  (:wat::kernel::Thread/join-result thr)
                  -> :wat::core::unit
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) ()))))
        "#;
        let result = check(src);
        match result {
            Ok(_) => {}
            Err(errors) => {
                let scope_deadlocks: Vec<_> = errors
                    .0
                    .iter()
                    .filter(|e| matches!(e, CheckError::ScopeDeadlock { .. }))
                    .collect();
                assert!(
                    scope_deadlocks.is_empty(),
                    "arc 134: spawn-thread inline lambda with no recv in body must NOT fire ScopeDeadlock (no recv-loop possible); got: {:?}",
                    scope_deadlocks
                );
            }
        }
    }
}
