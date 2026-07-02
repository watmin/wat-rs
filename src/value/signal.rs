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
    Diagnostic(RuntimeError),
    /// An eval-loop control signal — TCO / Result/try / Option/try
    /// propagation. Caught at function boundaries; never surfaces to
    /// user code.
    Signal(EvalSignal),
}

impl From<RuntimeError> for EvalBreak {
    fn from(e: RuntimeError) -> Self {
        EvalBreak::Diagnostic(e)
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
#[derive(Debug)]
pub struct RuntimeError {
    pub span: Span,
    pub kind: RuntimeErrorKind,
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
/// hand-written `runtime_error_to_edn` match in `runtime_error_edn.rs`.
#[derive(Debug, wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::RUNTIME)]
pub enum RuntimeErrorKind {
    #[to_edn(key = "name")]
    UnboundSymbol(String),
    #[to_edn(key = "path")]
    UnknownFunction(String),
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
    #[to_edn(key = "name")]
    DuplicateDefine(String),
    #[to_edn(key = "prefix")]
    ReservedPrefix(String),
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
        #[to_edn(via = crate::to_edn::error_edn_of_boxed)]
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
    /// Sibling rule to `CheckError::SandboxScopeLeak`: arc 140
    /// slice 2 catches the static case at outer freeze; this slice 1
    /// variant is the runtime backstop for dynamic / `eval-ast!` /
    /// otherwise check-walker-bypassing call paths. Same DESIGN.md
    /// covers both.
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
    /// in `crate::edn_shim`) found a shape mismatch between the
    /// caller's declared `-> :T` annotation and the EDN form on
    /// the wire. `expected` is the wat type the caller asked for;
    /// `got` is the EDN shape that actually arrived; `path`
    /// names the sub-field of the recursive coercion that
    /// failed (`""` for a top-level mismatch; `".name"`,
    /// `".[0]"`, etc. for nested cases).
    ///
    /// The diagnostic surface intentionally mirrors `EdnReadError`
    /// (the inverse direction — `wat_edn::OwnedValue` → wat `Value`
    /// without a target-T annotation); see `crate::edn_shim`.
    EdnCoerceMismatch {
        op: String,
        expected: String,
        got: String,
        #[to_edn(via = crate::runtime_error_edn::edn_path_segments)]
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
        attempted_clauses: Vec<ClauseAttempt>,
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
        ensure_span: Span,
    },
    /// Arc 258 Stone 258.2b — a `(:wat::core::macro-error "msg")` call aborting
    /// macro expansion with a user diagnostic. Returned as `Err` (not panic) so
    /// the macro engine (`macro_eval_pre_validated`) can wrap it into a clean
    /// `MacroError` — surfaced without "runtime::eval failed:" prefix noise.
    /// Macro-body-only: evaluated at expand time (step 4), never post-expansion.
    MacroAbort { message: String },
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
                write!(f, "{}unknown function: {}", prefix, p)
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
            RuntimeErrorKind::DeclarationInExpressionPosition(head) => write!(
                f,
                "{}{} is a declaration form, not an expression — declaration forms are \
                 top-level registration forms and cannot appear in expression position",
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
        }
    }
}

impl fmt::Display for RuntimeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Span-free form: passes None so no prefix or mid-prose span is inserted.
        self.fmt_with_span(None, f)
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Span-bearing form: passes Some so prefix and mid-prose spans are woven in.
        self.kind.fmt_with_span(Some(&self.span), f)
    }
}

impl std::error::Error for RuntimeError {}
