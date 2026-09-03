//! Eval-loop signal types — EvalSignal, EvalBreak, RuntimeError, RuntimeErrorKind.
//!
//! Moved from `src/runtime.rs` (block ~2024–2597) in Stone 251.2b.
//! Function and Value stay in runtime.rs until later stones (transitional imports).

use std::fmt;
use std::sync::Arc;
use crate::value::{ClauseAttempt, ClauseFailureReason, Value};
use crate::value::Function;
use crate::span::Span;
use crate::value::observe::ValueSnapshot;

/// Eval-loop control signals — NOT diagnostics. Raised and caught at function
/// boundaries (the TCO trampoline; the `?`/option propagation handler). If one
/// reaches user code, that is an interpreter bug (see the Display messages).
///
/// Arc 243 Stone 243.7b: split from `RuntimeError` so the diagnostic enum
/// becomes signal-free (prerequisite for the Pattern A retrofit in 243.7c).
#[derive(Debug)]
pub enum EvalSignal {
    /// Internal tail-call signal raised by `eval_tail` when it recognizes
    /// a user-defined function call in tail position. Carries the next
    /// function and its already-evaluated args up to the enclosing
    /// `apply_function` trampoline loop. Reaching user code is a bug.
    TailCall {
        func: Arc<Function>,
        args: Vec<Value>,
        /// Where in the caller this tail call was invoked — the List
        /// AST node's span. Arc 016 slice 2.
        call_span: Span,
    },
    /// Internal control-flow signal raised by `:wat::core::Result/try`
    /// on an `Err` value. Carries the `Err` payload up to the innermost
    /// enclosing function boundary; `apply_function` catches it and
    /// converts it into the function's own `Err(e)` return. Reaching
    /// user code is a checker invariant violation.
    TryPropagate(Box<Value>),
    /// Option-side propagation signal raised by `:wat::core::Option/try`
    /// on a `:None` value. Mirror of `TryPropagate` for the Option-returning
    /// function family. `apply_function` catches and converts to `Value::Option(None)`.
    OptionPropagate,
}

impl fmt::Display for EvalSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalSignal::TryPropagate(_) => write!(
                f,
                ":wat::core::Result/try: internal error — an Err propagation escaped its enclosing Result-returning function. The type checker should prevent this; reaching it indicates a checker gap or a try used in a context without a Result return type.",
            ),
            EvalSignal::OptionPropagate => write!(
                f,
                ":wat::core::Option/try: internal error — a :None propagation escaped its enclosing Option-returning function. The type checker should prevent this; reaching it indicates a checker gap or an Option/try used in a context without an Option return type.",
            ),
            EvalSignal::TailCall { .. } => write!(
                f,
                "TCO: internal error — a tail-call signal escaped its enclosing apply_function. The evaluator should catch TailCall at every function boundary; reaching the user with one unwound indicates an interpreter bug.",
            ),
        }
    }
}

/// The eval loop's `Err` type: an evaluation breaks either with a located
/// diagnostic (user-directed) or a control signal (evaluator-directed).
///
/// Arc 243 Stone 243.7b: `RuntimeError` becomes diagnostic-only; signals
/// live here so they can never masquerade as located diagnostics.
#[derive(Debug)]
pub enum EvalBreak {
    /// A genuine runtime diagnostic — carries a source location and
    /// surfaces to user code as an error.
    ///
    /// Boxed (arc 109, BRIEF-evalbreak-width): an inline RuntimeError made
    /// EvalBreak 128 bytes — exactly clippy's result_large_err threshold —
    /// earning 979 warnings. Boxed, EvalBreak is 80 (set by EvalSignal), so
    /// its width no longer tracks RuntimeErrorKind's widest variant.
    Diagnostic(Box<RuntimeError>),
    /// An eval-loop control signal — TCO / Result/try / Option/try
    /// propagation. Caught at function boundaries; never surfaces to
    /// user code.
    Signal(EvalSignal),
}

impl From<RuntimeError> for EvalBreak {
    fn from(e: RuntimeError) -> Self {
        EvalBreak::Diagnostic(Box::new(e))
    }
}

impl fmt::Display for EvalBreak {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalBreak::Diagnostic(e) => fmt::Display::fmt(e, f),
            EvalBreak::Signal(s) => fmt::Display::fmt(s, f),
        }
    }
}

/// Runtime error. Pattern A (Stone 243.7c): span at the outer struct level;
/// variant data in [`RuntimeErrorKind`].
///
/// The `span` field is mandatory at construction — Rust's struct-literal rule
/// makes a span-less `RuntimeError` uncompilable. `crate::rust_caller_span!()` is the
/// explicit sentinel for the rare site with no recoverable source location
/// (freeze-pair variants `UserMainMissing` / `EvalVerificationFailed`);
/// `Display` / EDN elide unknown spans.
pub struct RuntimeError {
    span: Span,
    /// Boxed (arc 109 stone B2). Inline, this field made `RuntimeError` 128 bytes —
    /// exactly clippy's `result_large_err` threshold — earning 482 warnings across
    /// every `Result<_, RuntimeError>` signature, because the struct's width tracked
    /// `RuntimeErrorKind`'s widest variant. Boxed, `RuntimeError` is 56 (48 span + 8
    /// pointer), so its width no longer tracks the kind enum at all and no future
    /// variant can re-breach the threshold.
    ///
    /// This is invisible to callers **by construction**: the field is private and
    /// reached only through `new` / `kind` / `into_kind` (stone B1), so the box is an
    /// implementation detail rather than a shape every call site has to know. That is
    /// why B1 came first — it is what made B2 a three-line change instead of a
    /// ~1438-site sweep, and what keeps the *next* width change three lines too.
    ///
    /// EDN is byte-identical, proven not assumed:
    /// `tests/value/probe_runtime_error_boxed_kind_edn.rs` pins that
    /// `Box<RuntimeErrorKind>::to_edn()` == `RuntimeErrorKind::to_edn()` via the
    /// blanket `impl<T: ToEdn> ToEdn for Box<T>` (`crates/wat-edn/src/lib.rs:217`),
    /// which the hand-written wrapper in `crate::edn::error` reaches by an
    /// auto-deref'd method call.
    kind: Box<RuntimeErrorKind>,
}

impl RuntimeError {
    /// The ONE door for construction.
    pub fn new(span: Span, kind: RuntimeErrorKind) -> Self {
        Self {
            span,
            kind: Box::new(kind),
        }
    }
    /// The ONE door for reading the kind. Returns `&RuntimeErrorKind` whether the
    /// storage is boxed or not — which is precisely why boxing cost no call site.
    pub fn kind(&self) -> &RuntimeErrorKind {
        &self.kind
    }
    /// The ONE door for taking the kind by value.
    pub fn into_kind(self) -> RuntimeErrorKind {
        *self.kind
    }
    /// Span stays inline — it is not what stone B2 boxes.
    pub fn span(&self) -> &Span {
        &self.span
    }
}

/// Arc 296 stone I — the taxonomy conversion `resolve::register`'s `?` performs at every
/// runtime-registration call site. `Rejection::verdict` is never `Insert`/`NoOp` (see its
/// doc), so those two arms are unreachable by construction.
impl From<crate::resolve::Rejection> for RuntimeError {
    fn from(r: crate::resolve::Rejection) -> Self {
        use crate::resolve::Registration;
        let kind = match r.verdict {
            Registration::Duplicate => RuntimeErrorKind::DuplicateDefine(r.name),
            Registration::Reserved => RuntimeErrorKind::ReservedPrefix(r.name),
            Registration::Unnamespaced => RuntimeErrorKind::UnnamespacedName(r.name),
            Registration::DottedName => RuntimeErrorKind::DottedName(r.name),
            Registration::Insert | Registration::NoOp => {
                unreachable!("resolve::register never rejects with Insert/NoOp")
            }
        };
        RuntimeError::new(r.span, kind)
    }
}

/// Variant data for [`RuntimeError`]. Spans live in the outer struct; variants
/// carry ONLY data unique to each failure kind.
///
/// **Multi-span variants** keep their SECONDARY spans as domain-named kind
/// fields per CONFORMARE.md § Multi-span. The outer `span` is the
/// most-actionable location (the site the user edits to fix):
/// - `SandboxScopeLeak`: outer = `call_span`, secondary = `outer_define_span`
/// - `PostconditionFailed`: outer = `body_span`, secondary = `ensure_span`
///
/// **Freeze pair** (`UserMainMissing`, `EvalVerificationFailed`): no span on
/// the kind; construct with outer `crate::rust_caller_span!()`, honestly elided by Display.
///
/// Arc 298.3: `#[derive(wat_edn::ToEdn)]` generates the kind enum's
/// `impl ToEdn`. The outer `RuntimeError::to_edn()` wraps it with
/// `splice_span(self.kind.to_edn(), &self.span)`. Replaces the deleted
/// hand-written `runtime_error_to_edn` match in `edn/error.rs`.
#[derive(Debug, wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::RUNTIME)]
pub enum RuntimeErrorKind {
    #[to_edn(key = "name")]
    UnboundSymbol(String),
    #[to_edn(key = "path")]
    UnknownFunction(String),
    /// Arc 255 Stone O-iv-a — registered in the intrinsic registry (name, arity, doc,
    /// examples all present; it works when called directly), but with no value-level
    /// door: it is a BINDING handler that takes `&[WatAST]`/`env`/`sym` and evaluates
    /// its own arguments, and `:wat::core::apply` has already evaluated its arguments
    /// into `&[Value]` — there is no AST left to hand it. PERMANENT, not transitional:
    /// no amount of sweeping the BINDING population (Stones O-iv-b/c/d) empties this,
    /// because a handler that needs `env`/`sym` can never be splatted. Deliberately its
    /// own variant rather than a reuse of `MalformedForm` (the call is well-formed) or
    /// a widened `UnknownFunction` (that tuple variant is pinned narrow — see its own
    /// comment at the `UnknownFunction` Display arm below; this name plainly IS known).
    NotValueDispatchable { name: String },
    NotCallable { got: Box<ValueSnapshot> },
    TypeMismatch {
        op: String,
        expected: &'static str,
        got: Box<ValueSnapshot>,
    },
    ArityMismatch {
        op: String,
        expected: usize,
        got: usize,
    },
    BadCondition { got: Box<ValueSnapshot> },
    MalformedForm { head: String, reason: String },
    #[to_edn(key = "name")]
    ParamShadowsBuiltin(String),
    DivisionByZero,
    /// `i64 + - *` (`checked_*`) overflow — Arc 300 stone C3. Distinct from
    /// `DivisionByZero` (conflating "doesn't fit in 64 bits" with "can't
    /// divide by zero" would be dishonest). Carries the op + both operands
    /// so the message names the exact overflowing expression (mirrors
    /// `TypeMismatch`/`ArityMismatch`'s op-carrying style). No auto-promotion
    /// to bigint — the caller chooses the wider type explicitly.
    IntegerOverflow {
        op: String,
        a: i64,
        b: i64,
    },
    #[to_edn(key = "name")]
    DuplicateDefine(String),
    #[to_edn(key = "prefix")]
    ReservedPrefix(String),

    /// Stone 118.B2c strike 1 — a `defclause` arm that can NEVER be selected, because an
    /// EARLIER arm accepts every value it accepts. Refused at registration.
    ///
    /// ## Why unreachability and not "overlap"
    ///
    /// Dispatch is first-match-wins in declaration order. Two arms whose domains merely
    /// INTERSECT are a legitimate FALLBACK — the later arm still fires for the rest of its
    /// domain, deterministically (`wat/bracket.wat:314-316` documents exactly this shape and
    /// depends on it). An arm whose domain is CONTAINED in an earlier one is dead code: no
    /// input can ever reach it, and nothing said so.
    ///
    /// ## Why this is the redef rule
    ///
    /// Arc 054 made `typealias`/`define`/`defmacro` "if byte-equivalent, no-op", else
    /// `DuplicateDefine`. Clause ARMS were never covered — an arm is not a definition BY NAME —
    /// so the one registry that dispatches on TYPES had no define-once rule. An arm that can
    /// never fire is a definition with no effect. Builder, 2026-08-18: *"you may only express
    /// something's def once and all other attempts must be identical."*
    ///
    /// 054's idempotent escape hatch is deliberately NOT carried over: it exists because a FILE
    /// can legitimately load twice (in-crate shims). Arms inside one form are hand-written
    /// adjacently and cannot arrive that way, so a byte-identical duplicate arm has no
    /// legitimate source.
    UnreachableClause {
        name: String,
        /// 0-based index of the arm that can never be selected.
        clause_index: usize,
        /// 0-based index of the EARLIER arm that subsumes it.
        subsumed_by: usize,
        /// Formatted declared types of the unreachable arm.
        declared_arg_types: Vec<String>,
    },
    /// A top-level name reached a registration gate with no namespace. Only
    /// fn arguments and `let` bindings may be bare — those are lexical and
    /// never reach a gate. Held against `Privilege::Stdlib` too; there is no
    /// privilege escape from the namespacing wall.
    #[to_edn(key = "name")]
    UnnamespacedName(String),
    /// Arc 296 stone H-1 — a name reached the registration gate with a `.` in its name
    /// segment (the part after the last `::`). Same door as `UnnamespacedName` /
    /// `ReservedPrefix` above — second taxonomy entry for `Registration::DottedName`
    /// (`TypeErrorKind`, `RuntimeErrorKind`, `MacroErrorKind`, `CheckErrorKind` all got
    /// theirs). Reserved because a dotted NAME is the wire discriminator for a
    /// tagged-enum variant (`#ns/Enum.Variant`); a record whose name contained a dot
    /// could forge it.
    #[to_edn(key = "name")]
    DottedName(String),
    /// A declaration form (`:wat::core::def`, `:wat::core::define`, etc.)
    /// found in expression position at runtime. Declaration forms are
    /// top-level registration forms; calling one at expression position
    /// means the caller confused the two phases. `head` names the
    /// specific declaration form that was misplaced.
    ///
    /// Arc 170 Gap I-B — minted to replace `DefineInExpressionPosition`,
    /// which only named `define`. Now covers all 8 declaration forms
    /// with a single symmetric variant carrying the offending `head`.
    #[to_edn(key = "head")]
    DeclarationInExpressionPosition(String),
    /// A constrained `eval` (`eval_in_frozen`) found a mutation-inducing
    /// form inside the AST it was asked to evaluate. Per FOUNDATION
    /// (§ constrained eval, line 663): "If the submitted AST contains a
    /// `define`, `defmacro`, `struct`, `enum`, `newtype`, `typealias`,
    /// or `load` form — eval refuses. This is not a mode; it is an
    /// invariant." Also covers `set-*!` config setters.
    EvalForbidsMutationForm { head: String },
    /// `:user::main` was not registered at startup. FOUNDATION requires
    /// exactly one `:user::main` declaration; zero halts.
    /// Freeze pair — no span; construct with outer `crate::rust_caller_span!()`.
    UserMainMissing,
    /// Verification failed for a `:wat::eval-digest!` /
    /// `:wat::eval-signed!` call. The wrapped [`HashError`]
    /// names the specific failure (mismatched digest, invalid
    /// signature, unsupported algorithm, malformed payload).
    /// Freeze pair — no span; construct with outer `crate::rust_caller_span!()`.
    EvalVerificationFailed {
        #[to_edn(key = "error")]
        err: crate::hash::HashError,
    },
    /// Raised when `:wat::kernel::join` reaps a spawned program
    /// whose thread panicked before yielding a result — the internal
    /// handle channel's Sender was dropped without sending, so the
    /// join's `recv` sees disconnected.
    ///
    /// User channels (`:wat::kernel::send` / `recv`)
    /// are symmetric on disconnect — both endpoints report it via
    /// `:Option` rather than via this error, so no call path in the
    /// user-level channel primitives produces this variant. It
    /// remains only for the join-on-panic case.
    ChannelDisconnected { op: String },
    /// A vector-level primitive (`:wat::holon::cosine`,
    /// `:wat::config::noise-floor`, etc.) was invoked but the
    /// [`SymbolTable`] has no attached [`EncodingCtx`]. Reachable from
    /// test harnesses that don't go through freeze; the frozen startup
    /// pipeline always installs one.
    NoEncodingCtx { op: String },
    /// A file-reading primitive (`:wat::eval-file!`, file-path
    /// variants of the verified eval/load forms, `:wat::verify::file-path`
    /// payloads) was invoked but the [`SymbolTable`] has no attached
    /// source loader. The frozen startup pipeline attaches the loader
    /// handed to `startup_from_source`; test harnesses that build a
    /// SymbolTable directly must call [`SymbolTable::set_source_loader`]
    /// to grant file-I/O capability.
    NoSourceLoader { op: String },
    /// `:wat::core::macroexpand` / `macroexpand-1` was invoked but the
    /// [`SymbolTable`] has no attached macro registry. The frozen
    /// startup pipeline attaches the registry; test harnesses that
    /// build a SymbolTable directly must call
    /// [`SymbolTable::set_macro_registry`] to grant macro-expansion
    /// capability. Arc 030.
    NoMacroRegistry { op: String },
    /// `:wat::core::macroexpand` / `macroexpand-1` surfaced a macro-
    /// expansion error (malformed template, arity mismatch in the
    /// expanded call, expansion-depth cycle, etc.). Carries the
    /// wrapped [`crate::macros::MacroError`] description. Arc 030.
    MacroExpansionFailed {
        op: String,
        #[to_edn(via = crate::edn::contract::error_edn_of_boxed)]
        cause: Box<crate::macros::MacroError>,
    },
    /// A `(:wat::core::match scrutinee ...)` ran with no arm whose
    /// pattern matches the scrutinee's shape. Exhaustiveness is the
    /// type checker's job; this variant fires only when the check was
    /// bypassed or hasn't caught up with a new pattern form.
    PatternMatchFailed { value_type: &'static str },
    /// Arc 068 — `:wat::eval-step!` saw a form whose head is an
    /// effectful op (kernel sends/recvs, IO writes, channel-construction
    /// primitives, `:wat::eval-ast!` itself, etc.). The stepwise
    /// evaluator deliberately rejects effects so the BOOK Chapter 59
    /// dual-LRU cache's "form IS its return value" invariant holds —
    /// the caller falls back to `:wat::eval-ast!` for sub-forms with
    /// effects.
    EffectfulInStep { op: String },
    /// Arc 068 — `:wat::eval-step!` saw a form whose shape isn't yet
    /// covered by a step rule (a future stdlib op, an unrecognized
    /// keyword head). Caller falls back to `:wat::eval-ast!` for
    /// the affected sub-form. Distinct from `EffectfulInStep` so
    /// consumers can distinguish "out of scope by design" from "not
    /// taught yet."
    NoStepRule { op: String },
    /// Raised by `:wat::kernel::assertion-failed!` when an assertion in
    /// a `:wat::test::*` form (or any user code that calls the primitive
    /// directly) fails. Intended to travel as a panic payload via the
    /// [`crate::assertion::AssertionPayload`] struct and be caught by
    /// `run-sandboxed`'s `catch_unwind`, where actual/expected land in
    /// the `:wat::kernel::Failure`'s slots. Outside a sandbox, this
    /// variant surfaces as an ordinary RuntimeError — reporting that
    /// an assertion fired without a test harness to catch it.
    AssertionFailed {
        message: String,
        actual: Option<String>,
        expected: Option<String>,
    },
    /// Arc 140 slice 1 — runtime panic enrichment. Fires when a
    /// sub-program (`run-sandboxed-ast` / `run-sandboxed-hermetic-ast`
    /// / `spawn-process`) hits an
    /// `UnknownFunction` AND the offending name (canonical form,
    /// stripping `<T,...>`) IS registered in the OUTER scope's
    /// `SymbolTable`. The substrate teaches: *"sandbox-scope leak —
    /// you defined this at outer scope but deftest sandboxes don't
    /// capture; move it into the prelude."* Both spans land so users
    /// click the call site AND the outer-scope define.
    ///
    /// The runtime backstop for scope leaks on dynamic / `eval-ast!` /
    /// otherwise check-walker-bypassing call paths. (The arc-140 static
    /// check-time twin `CheckError::SandboxScopeLeak` was annihilated with
    /// the arc-170 `*-program-ast` retirement — it fired only on those
    /// now-deleted forms-block heads. This runtime variant is a distinct,
    /// live feature over the `outer_symbols` sub-program mechanism.)
    ///
    /// Multi-span: outer `span` = `call_span` (most-actionable).
    /// Secondary: `outer_define_span` (the outer-scope define).
    SandboxScopeLeak {
        offending_name: String,
        /// Source location of the outer-scope define. May be `crate::rust_caller_span!()`.
        outer_define_span: crate::span::Span,
    },
    /// Arc 170 slice 1f-α — a thread-aware stdio helper
    /// (`:wat::kernel::println` / `eprintln` / `readln`) was
    /// invoked on a thread whose [`crate::services::ThreadIO`]
    /// cell is empty. The runtime spawns the three substrate
    /// stdio services at process start (slice 1f-δ) and the
    /// orchestrator (slice 1f-γ) populates ThreadIO from
    /// `:wat::kernel::spawn-thread`; reaching this variant means
    /// either the helper was called pre-orchestrator-bootstrap
    /// (a test harness that constructed a SymbolTable directly)
    /// or the calling thread was started outside the
    /// `:wat::kernel::spawn-thread` path (e.g., a hand-rolled
    /// `std::thread::spawn`). Tests populate ThreadIO via
    /// [`crate::services::install_thread_io`] before invoking
    /// the primitive.
    ServiceNotRunning {
        op: String,
    },
    /// Arc 170 slice 1f-ι — `:wat::kernel::readln`'s
    /// EDN→typed-`T` coercion (the `edn_to_typed_value` walker
    /// in `crate::edn::render`) found a shape mismatch between the
    /// caller's declared `-> :T` annotation and the EDN form on
    /// the wire. `expected` is the wat type the caller asked for;
    /// `got` is the EDN shape that actually arrived; `path`
    /// names the sub-field of the recursive coercion that
    /// failed (`""` for a top-level mismatch; `".name"`,
    /// `".[0]"`, etc. for nested cases).
    ///
    /// The diagnostic surface intentionally mirrors `EdnReadError`
    /// (the inverse direction — `wat_edn::OwnedValue` → wat `Value`
    /// without a target-T annotation); see `crate::edn::render`.
    EdnCoerceMismatch {
        op: String,
        // Arc 109 kill-std stone (BRIEF-runtime-error-width): boxed to bring the
        // variant from 96 to 64 bytes. `Box<String>` keeps `ToEdn` automatic via
        // the blanket `impl<T: ToEdn> ToEdn for Box<T>` (wat-edn/src/lib.rs) —
        // no derive/via changes, byte-identical EDN (`(**self).to_edn()` ==
        // `self.to_edn()`). `op` stays a bare `String` (matches the op-field
        // convention on every other op-carrying variant); `path` keeps its
        // existing `via` and stays unboxed.
        expected: Box<String>,
        got: Box<String>,
        #[to_edn(via = crate::edn::error::edn_path_segments)]
        path: String,
    },
    /// Arc 234 Stone 234.3b.fix — `:wat::core::Record/assoc` was invoked with
    /// a field key that does not exist on the record's class. Carries the
    /// bare class FQDN (no leading colon), the attempted field name, the
    /// list of actually-available field names, and the call site span.
    ///
    /// Minted to replace the previous `MalformedForm` catch-all stuffing
    /// in `eval_record_assoc`; every distinct error semantics gets its
    /// own variant (no reason-string stuffing into existing variants).
    UnknownField {
        record_class: String,   // bare FQDN, e.g. "myapp::Voltage" (no leading colon)
        field: String,          // bare field-name attempted, e.g. "nonexistent"
        available: Vec<String>, // known field names on the record
    },
    /// Stone 237.4 — runtime fall-through for a defclause call where no
    /// clause matches the actual argument values. Carries a structured
    /// `Vec<ClauseAttempt>` so the diagnostic surface teaches WHY each
    /// clause was skipped (arity / type / guard-false), not just THAT none
    /// matched (arc 233 errors-as-teaching-values doctrine).
    ///
    /// The type-checker should have caught the mismatch at check time via
    /// `CheckError::NoMatchingClauseAtCallSite`; this is the runtime
    /// defensive guard for cases that slip through (dynamic dispatch,
    /// incomplete type coverage, or a checker gap).
    ///
    /// Per arc 233 discipline: `called_args` uses `ValueSnapshot` (carries
    /// type_name + rendered representation).
    NoMatchingClause {
        name: String,
        called_arity: usize,
        called_args: Vec<ValueSnapshot>,
        /// Structured per-clause failure reasons (Stone 237.4 promotion).
        ///
        /// Arc 109 kill-std stone (BRIEF-runtime-error-width): boxed to bring
        /// the variant from 80 to 64 bytes (`Vec<ClauseAttempt>` 24 -> `Box<Vec<
        /// ClauseAttempt>>` 8), via the same blanket `Box<T: ToEdn>` forwarding
        /// as `EdnCoerceMismatch`'s boxed strings — byte-identical EDN. Boxed
        /// (not `called_args`) because it is the rich, secondary diagnostic
        /// payload — the same role `MacroExpansionFailed.cause` plays.
        attempted_clauses: Box<Vec<ClauseAttempt>>,
    },

    /// Stone 237.4 — postcondition failure for a defclause clause whose
    /// `:ensure` `:fn` returned `false`. The body executed successfully but
    /// the postcondition rejected the result. Carries the ensure expression
    /// snapshot so the diagnostic surface shows WHICH postcondition failed
    /// and dual spans (body site vs ensure-declaration site).
    ///
    /// Per arc 233 discipline: `returned_value` uses `ValueSnapshot`.
    ///
    /// Multi-span: outer `span` = `body_span` (most-actionable).
    /// Secondary: `ensure_span` (the `:ensure :fn` declaration).
    PostconditionFailed {
        defclause_name: String,
        clause_index: usize,
        /// Rendered form of the `:ensure :fn` expression (captured at dispatch time).
        ensure_expr_snapshot: String,
        returned_value: Box<ValueSnapshot>,
        /// Span of the `:ensure :fn` declaration.
        ///
        /// Arc 109 kill-std stone (BRIEF-runtime-error-width): boxed
        /// (48 -> 8 bytes) — this single field alone brings the variant from
        /// 112 to 72 bytes (measured). `Span: ToEdn` (wat-reader derive), so
        /// `Box<Span>` gets `ToEdn` for free via the same blanket forwarding
        /// impl; byte-identical EDN.
        ensure_span: Box<Span>,
    },
    /// Arc 258 Stone 258.2b — a `(:wat::core::macro-error "msg")` call aborting
    /// macro expansion with a user diagnostic. Returned as `Err` (not panic) so
    /// the macro engine (`macro_eval_pre_validated`) can wrap it into a clean
    /// `MacroError` — surfaced without "runtime::eval failed:" prefix noise.
    /// Macro-body-only: evaluated at expand time (step 4), never post-expansion.
    MacroAbort { message: String },
    /// Arc 170 closure #5 "the writer joins the lock-step" — `PipeWriter::write`
    /// polled `[fd, SHUTDOWN_BROADCAST_READ_FD]` before a write attempt (mirroring
    /// `channel/transfer.rs`'s read-side `read_one_line`) and the broadcast fired
    /// first. NOT an I/O error — the peer/pipe may be perfectly healthy; the write
    /// was preempted by a process-wide stop request. Kept distinct from the
    /// `MalformedForm` a genuine `write(2)` failure (EPIPE, etc.) produces so a
    /// caller CAN tell "stopped" apart from "broken" once something reads this
    /// field. Today `channel/transfer.rs`'s `typed_send` PipeFd arm still folds
    /// this into `SendOutcome::Disconnected` (unchanged from every other
    /// pipe-write failure) — distinguishing it there needs a `SendOutcome::
    /// Shutdown` variant, which cascades to `kernel/address.rs`'s
    /// `ThreadAddress::connect` and has no honest landing spot in `ConnectFail`
    /// without extending the arc 278 "connect' OUTCOME WALL"; a design call
    /// left to a follow-up (see `typed_send`'s PipeFd match arm for the detail).
    WriteStopped,
    /// Arc 278 #88 — a `(:wat::rete::core::defn …)` declaration's body failed one of the four
    /// axes the fence measures (Pure / Deterministic / Total / Law A —
    /// `crate::rete::purity::Axis`), checked ONCE at the definition site
    /// (`crate::rete::purity::apply_rete_defn_contracts`) instead of being re-derived per call
    /// site. `name` is the DECLARED helper's own FQDN — the inversion the whole stone exists
    /// for: an ordinary `defn` used from a `where` fails naming the calling RULE, with not one
    /// frame naming the helper; this error names the helper directly, at the point the author
    /// edited. `axis` is the failing axis's `Axis::variant_name()`; `head` is the specific
    /// violating sub-expression `find_axis_violation` located (may equal `name` itself, for a
    /// single-form body, or a deeper call inside it).
    ReteDefnAxisViolation {
        name: String,
        axis: &'static str,
        head: String,
    },
    /// Arc 278 #87 — a `(:wat::rete::core::defn …)` declaration's body (transitively)
    /// calls itself, or participates in a cycle of rete-defns. eBPF-shaped: a static
    /// refusal at LOAD, never a runtime budget. Once the fn is in the network every
    /// fire must complete; a user expression may not hang or blow the fire loop.
    /// `name` is the declared helper; `head` is the callee that closed the cycle
    /// (equal to `name` for self-recursion). Not an axis — a cycle is still pure.
    ReteDefnRecursive {
        name: String,
        head: String,
    },
}

/// Arc 138 slice 3a — render the file:line:col prefix for a RuntimeError.
/// Arc 298.2: every span is real; always emit. Mirrors `src/check.rs::span_prefix`.
fn span_prefix(span: &Span) -> String {
    format!("{}: ", span)
}

impl RuntimeErrorKind {
    /// Render this error kind's human-facing message with the span woven in.
    ///
    /// `span` is `Some(&outer_span)` when rendering a full [`RuntimeError`]
    /// (span-bearing form) and `None` when rendering the kind alone (span-free
    /// form). Message text lives here exactly once; both `Display` impls
    /// delegate here.
    fn fmt_with_span(
        &self,
        span: Option<&Span>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let prefix = span.map(span_prefix).unwrap_or_default();
        // Arc 298.2: every span is real; always emit in prose where needed.
        let prose_span: Option<&Span> = span;
        match self {
            RuntimeErrorKind::UnboundSymbol(s) => {
                write!(f, "{}unbound symbol: {}", prefix, s)
            }
            RuntimeErrorKind::UnknownFunction(p) => {
                // Arc 255 STONE-retirement-table-becomes-mechanism — DOOR 2: a
                // dynamically-built head (`eval-ast!`, `keyword/from-string`) never
                // passes the checker, so door 1 (src/check.rs) alone leaves a hole —
                // this is the runtime-only path. `RuntimeErrorKind::UnknownFunction`
                // is a tuple variant carrying only the path (pinned: do NOT widen it
                // to carry a structured remedies list — that is a different stone), so
                // the retirement replacement is folded directly into the message text.
                // `remedies_for` runs retirement lookup FIRST and an exact table hit
                // always scores 0 (a typo is never < 1), so this fires ONLY on a
                // genuine retirement.
                let remedies = crate::remedy::remedies_for(p, std::iter::empty());
                match remedies.first() {
                    Some(r) if r.score() == 0 => write!(
                        f,
                        "{}unknown function: {} — '{}' is retired; use '{}' instead",
                        prefix, p, p, r.form
                    ),
                    _ => write!(f, "{}unknown function: {}", prefix, p),
                }
            }
            // ⛔ THIS MESSAGE STATES AN ABSENCE, NEVER A REASON — corrected 2026-08-28, hours
            // after it shipped, at the builder's question: *"what prevents application?… is
            // max-of written wrong?"* It WAS written wrong. The first draft said "it takes its
            // arguments unevaluated", asserting an essential property of the verb. For
            // `:wat::f64::max-of` that is FALSE: `f64_variadic_reduce` uses `env`/`sym` for
            // exactly one thing — `eval_inner` on its own arguments — and is a pure fold after
            // that. It is ALGEBRA wearing a BINDING signature, and so are most of the 331 this
            // fires for. Until O-iii landed, `&[WatAST]` was the ONLY signature `#[wat_intrinsic]`
            // accepted, so every handler took it whether or not it needed to; the registry then
            // recorded `value_handler: None` and `apply` read that ABSENCE as an IMPOSSIBILITY.
            // Same defect as `walk.rs:268`: a dispatch path treating what it was not told as
            // something it knows. Say what is missing. Do not say why.
            RuntimeErrorKind::NotValueDispatchable { name } => {
                write!(
                    f,
                    "{}{} is registered, but no handler taking EVALUATED arguments is \
                     registered under that name, and apply dispatches with evaluated \
                     arguments. Call it directly.",
                    prefix, name
                )
            }
            RuntimeErrorKind::NotCallable { got } => {
                write!(f, "{}not callable: expected Function, got {}", prefix, got)
            }
            RuntimeErrorKind::TypeMismatch { op, expected, got } => {
                write!(f, "{}{}: expected {}, got {}", prefix, op, expected, got)
            }
            RuntimeErrorKind::ArityMismatch { op, expected, got } => {
                write!(f, "{}{}: expected {} arguments, got {}", prefix, op, expected, got)
            }
            RuntimeErrorKind::BadCondition { got } => {
                write!(f, "{}if / when condition must be :wat::core::bool; got {}", prefix, got)
            }
            RuntimeErrorKind::MalformedForm { head, reason } => {
                write!(f, "{}malformed {} form: {}", prefix, head, reason)
            }
            RuntimeErrorKind::ParamShadowsBuiltin(s) => {
                write!(f, "{}parameter name {} shadows a :wat::core builtin; pick another name", prefix, s)
            }
            RuntimeErrorKind::DivisionByZero => {
                write!(f, "{}division by zero", prefix)
            }
            RuntimeErrorKind::IntegerOverflow { op, a, b } => {
                write!(f, "{}i64 overflow: {} {} {} does not fit in 64 bits", prefix, a, op, b)
            }
            RuntimeErrorKind::DuplicateDefine(p) => {
                write!(f, "{}duplicate define: {} already registered", prefix, p)
            }
            RuntimeErrorKind::ReservedPrefix(p) => write!(
                f,
                "{}cannot define {} — reserved prefix ({}); user defines must use their own prefix",
                prefix,
                p,
                crate::resolve::reserved_prefix_list()
            ),
            RuntimeErrorKind::UnreachableClause {
                name,
                clause_index,
                subsumed_by,
                declared_arg_types,
            } => write!(
                f,
                "{}{}: clause #{} [{}] can never be selected — clause #{} is declared earlier and \
                 accepts every value it accepts. defclause dispatch is first-match-wins, so this \
                 arm is dead code. Give it a type no earlier clause accepts, add a :guard to the \
                 earlier clause, or delete it.",
                prefix,
                name,
                clause_index,
                declared_arg_types.join(", "),
                subsumed_by
            ),
            RuntimeErrorKind::UnnamespacedName(name) => write!(
                f,
                "{}top-level name '{}' is not namespaced — only fn arguments and let-bindings \
                 may be bare; give it a namespace, e.g. ':my::{}'",
                prefix,
                name,
                name.trim_start_matches(':')
            ),
            RuntimeErrorKind::DottedName(name) => write!(
                f,
                "{}name '{}' contains a '.' in its name segment — reserved: a dot in a tag's \
                 NAME half means \"this is an enum variant\" (`#ns/Enum.Variant`), so a \
                 registered name may not contain one, or it could forge that tag; rename \
                 without the dot",
                prefix, name
            ),
            RuntimeErrorKind::DeclarationInExpressionPosition(head) => write!(
                f,
                "{}{} is consumed before evaluation — it is registered or spliced at freeze \
                 time and never evaluated, so it cannot appear in expression position",
                prefix,
                head
            ),
            RuntimeErrorKind::EvalForbidsMutationForm { head } => write!(
                f,
                "{}constrained eval refuses mutation form {}; eval evaluates against the frozen symbol table and cannot register / replace / load definitions",
                prefix,
                head
            ),
            RuntimeErrorKind::UserMainMissing => write!(
                f,
                ":user::main not defined — a wat program needs an entry point"
            ),
            RuntimeErrorKind::EvalVerificationFailed { err } => {
                write!(f, "eval verification failed: {}", err)
            }
            RuntimeErrorKind::ChannelDisconnected { op } => write!(
                f,
                "{}{}: channel disconnected — receiver was dropped. `recv` is now Option-returning (disconnect yields :None); only `send` to a dropped receiver raises this error.",
                prefix, op
            ),
            RuntimeErrorKind::NoEncodingCtx { op } => write!(
                f,
                "{}{}: no encoding context attached to SymbolTable; presence / config accessors need a frozen EncodingCtx. Call via the freeze pipeline rather than a bare SymbolTable::new().",
                prefix, op
            ),
            RuntimeErrorKind::NoSourceLoader { op } => write!(
                f,
                "{}{}: no source loader attached to SymbolTable; file-reading primitives require a loader. Call via the freeze pipeline, or set_source_loader on the test SymbolTable.",
                prefix, op
            ),
            RuntimeErrorKind::NoMacroRegistry { op } => write!(
                f,
                "{}{}: no macro registry attached to SymbolTable; macroexpand / macroexpand-1 require one. Call via the freeze pipeline, or set_macro_registry on the test SymbolTable.",
                prefix, op
            ),
            RuntimeErrorKind::MacroExpansionFailed { op, cause } => write!(
                f,
                "{}{}: macro expansion failed: {}",
                prefix, op, cause
            ),
            RuntimeErrorKind::PatternMatchFailed { value_type } => write!(
                f,
                "{}:wat::core::match: no arm matched scrutinee of type {}; exhaustiveness should be caught at type-check time",
                prefix, value_type
            ),
            RuntimeErrorKind::EffectfulInStep { op } => write!(
                f,
                "{}:wat::eval-step!: refuses to step effectful op {}; the BOOK Chapter 59 dual-LRU cache assumes form IS its return value (no side effects). Fall back to :wat::eval-ast! for sub-forms with effects.",
                prefix, op
            ),
            RuntimeErrorKind::NoStepRule { op } => write!(
                f,
                "{}:wat::eval-step!: no step rule for op {}; v1 covers arithmetic / logical / control flow / let / match / function call / holon constructors. Fall back to :wat::eval-ast! for unrecognized heads.",
                prefix, op
            ),
            RuntimeErrorKind::AssertionFailed { message, actual, expected } => {
                write!(f, "{}assertion failed: {}", prefix, message)?;
                if let Some(a) = actual {
                    write!(f, "\n  actual:   {}", a)?;
                }
                if let Some(e) = expected {
                    write!(f, "\n  expected: {}", e)?;
                }
                Ok(())
            }
            RuntimeErrorKind::SandboxScopeLeak { offending_name, outer_define_span } => {
                // outer span (call_span) is in prefix; secondary span here.
                // Arc 298.2: span is always real; always emit the location.
                let define_loc = format!("{}", outer_define_span);
                write!(
                    f,
                    "{}sandbox-scope leak: '{}' invoked here is defined at {} but deftest sandboxes do NOT capture outer-scope. Move (:wat::core::defn {} ...) into this deftest's prelude (the second argument of `(:wat::test::deftest <name> <prelude> <body>)`), or load it into the prelude via `(:wat::core::load! \"path/to/file.wat\")`. Sandbox isolation is intentional — see wat/test.wat's deftest macro.",
                    prefix, offending_name, define_loc, offending_name
                )
            }
            RuntimeErrorKind::ServiceNotRunning { op } => write!(
                f,
                "{}{}: called before stdio services running. The runtime spawns these services at process start (arc 170 slice 1f-δ); when called from a hand-spawned context (e.g., a test), the test must populate the per-thread routing via `wat::services::install_thread_io` before invoking. See arc 170 REALIZATIONS pass 15 + pass 16 for the substrate's thread-aware-helper architecture.",
                prefix, op
            ),
            RuntimeErrorKind::EdnCoerceMismatch { op, expected, got, path } => write!(
                f,
                "{}{}: edn coerce mismatch: expected {}, got {}{}",
                prefix,
                op,
                expected,
                got,
                if path.is_empty() {
                    String::new()
                } else {
                    format!(" at {}", path)
                }
            ),
            RuntimeErrorKind::UnknownField { record_class, field, available } => write!(
                f,
                "{}unknown field '{}' on record {}; available: [{}]",
                prefix,
                field,
                record_class,
                available.join(", ")
            ),
            RuntimeErrorKind::NoMatchingClause { name, called_arity, called_args, attempted_clauses } => {
                let args_fmt: Vec<String> = called_args.iter().map(|s| format!("{}", s)).collect();
                let attempts_fmt: Vec<String> = attempted_clauses.iter().map(|a| {
                    let reason = match &a.failure_reason {
                        ClauseFailureReason::ArityMismatch { expected, got } =>
                            format!("arity {} ≠ {}", expected, got),
                        ClauseFailureReason::ArgTypeMismatch { position, expected, got } =>
                            format!("arg {}: expected {}, got {}", position, expected, got),
                        ClauseFailureReason::GuardFalse =>
                            "guard false".to_string(),
                    };
                    format!("clause {} skipped ({})", a.clause_index, reason)
                }).collect();
                write!(
                    f,
                    "{}no clause of {} matched ({} args); {}{}",
                    prefix,
                    name,
                    called_arity,
                    if args_fmt.is_empty() { String::new() } else { format!("called with ({}); ", args_fmt.join(", ")) },
                    if attempts_fmt.is_empty() {
                        "no clauses declared".to_string()
                    } else {
                        attempts_fmt.join("; ")
                    },
                )
            }
            RuntimeErrorKind::PostconditionFailed { defclause_name, clause_index, ensure_expr_snapshot, returned_value, ensure_span: _ } => {
                // outer span = body_span (in prefix); ensure_span is secondary (informational).
                // We re-render body_span from `prose_span` (which is the outer span).
                let body_span_str = prose_span.map(|s| format!("{}", s)).unwrap_or_default();
                write!(
                    f,
                    "{}defclause {}: postcondition failed on clause {} — :ensure :fn `{}` returned false for result {} (body at {})",
                    prefix,
                    defclause_name,
                    clause_index,
                    ensure_expr_snapshot,
                    returned_value,
                    body_span_str,
                )
            }
            RuntimeErrorKind::MacroAbort { message } => write!(f, "{}{}", prefix, message),
            RuntimeErrorKind::WriteStopped => {
                write!(f, "{}write stopped: a process-wide shutdown was requested", prefix)
            }
            RuntimeErrorKind::ReteDefnAxisViolation { name, axis, head } => write!(
                f,
                "{}(:wat::rete::core::defn {} ...): declaration refused — '{}' is not proven {} \
                 (a rete-defn's body is checked ONCE, at ITS OWN declaration, against Pure ∧ \
                 Deterministic ∧ Total ∧ Law A; nothing calling {} needs to re-derive this)",
                prefix, name, head, axis, name
            ),
            RuntimeErrorKind::ReteDefnRecursive { name, head } => write!(
                f,
                "{}(:wat::rete::core::defn {} ...): declaration refused — recursive callee '{}' \
                 (a rete-defn may not recurse; a user expression may not fault the fire loop — \
                 fold over a finite collection)",
                prefix, name, head
            ),
        }
    }
}

impl fmt::Display for RuntimeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Span-free form: passes None so no prefix or mid-prose span is inserted.
        self.fmt_with_span(None, f)
    }
}

impl fmt::Debug for RuntimeError {
    // Stone B (arc 296): {:?}-impostor wall — Debug emits EDN via to_wire_edn,
    // not the Rust struct layout. Every face that reads this error sees structured
    // EDN, never a Rust debug blob.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::edn::contract::to_wire_edn(self))
    }
}

impl std::error::Error for RuntimeError {}
