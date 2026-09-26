//! Call-stack tracking — FrameInfo (per-frame data), FrameGuard (RAII push/pop of the thread-local stack), and snapshot_call_stack/replace_top_frame for reading and amending the top frame.
use crate::span::Span;

/// The `symbol` marker (callee-path) for an ANONYMOUS fn's identity.
///
/// Arc 109 (`NOTE-anon-fn-identity-structured-not-stringy.md`): an anon fn's
/// identity used to be rendered as the string `format!("<fn@{}>", ast.span())`
/// — a structured unit (a source location) wearing a string costume. That
/// costume is killed at the three sites (`freeze.rs` ×2, `runtime.rs`
/// `fn_display_name`); each now uses THIS marker as the anon fn's callee-path /
/// `symbol`, and the location travels structurally via the enclosing
/// `:wat::kernel::Frame`'s `file`/`line` (`value_from_frame_info`) — never
/// packed into the name again.
///
/// Value: the FQDN of the Fn TYPE, `:wat::core::Fn`. wat is FQDN at all times;
/// a bare `<anonymous>` marker would be the un-namespaced outlier in a slot that
/// otherwise holds FQDN callee-paths (a named fn's `symbol` is its own path,
/// e.g. `:user::compute`). An anonymous fn's honest FQDN identity is its TYPE —
/// it IS a `:wat::core::Fn` — so `symbol = :wat::core::Fn`, stored in the SAME
/// raw callee-path string form named symbols use (empirically confirmed: the
/// `symbol` field renders as a raw quoted EDN string of the path, e.g. a named
/// fn renders `":user::compute"` — `tests/services/probe_arc278_journal_service_logs.rs`).
/// So this renders `":wat::core::Fn"`. The rule: symbol = the FQDN name if
/// bound, else the FQDN type `:wat::core::Fn`.
pub(crate) const ANON_FN_SYMBOL: &str = ":wat::core::Fn";

/// One entry on the wat call stack.
#[derive(Debug, Clone)]
pub struct FrameInfo {
    pub callee_path: String,
    pub call_span: Span,
}

thread_local! {
    static CALL_STACK: std::cell::RefCell<Vec<FrameInfo>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Scope guard that pushes a frame on construction and pops on drop.
/// Ensures the call stack unwinds cleanly on early return / panic.
#[must_use = "FrameGuard must be bound to a local (let _g = ...); dropping it immediately pops the frame"]
pub(crate) struct FrameGuard;

impl FrameGuard {
    pub(crate) fn push(callee_path: String, call_span: Span) -> Self {
        CALL_STACK.with(|s| {
            s.borrow_mut().push(FrameInfo { callee_path, call_span });
        });
        FrameGuard
    }
}

impl Drop for FrameGuard {
    fn drop(&mut self) {
        CALL_STACK.with(|s| {
            s.borrow_mut().pop();
        });
    }
}

/// Replace the top frame's contents in place — called on tail-call
/// iteration inside apply_function's trampoline. The stack depth
/// stays the same; the content substitutes.
pub(crate) fn replace_top_frame(callee_path: String, call_span: Span) {
    CALL_STACK.with(|s| {
        if let Some(top) = s.borrow_mut().last_mut() {
            *top = FrameInfo { callee_path, call_span };
        }
    });
}

/// Snapshot the current call stack (newest-first order). Used by
/// `:wat::kernel::assertion-failed!` at panic time to populate the
/// `AssertionPayload`'s `location` + `frames` fields.
pub fn snapshot_call_stack() -> Vec<FrameInfo> {
    CALL_STACK.with(|s| {
        let stack = s.borrow();
        stack.iter().rev().cloned().collect()
    })
}

// ─── Arc 278 §4 — macro-invocation call-site stack ───────────────────────────
//
// The expand-time twin of `CALL_STACK` above. `:wat::kernel::macro-call-site`
// (`src/kernel/source.rs`, beside `eval_kernel_call_site`) needs the SOURCE SPAN of the
// macro invocation currently being expanded — not a runtime call-stack frame
// (macro expansion runs before any wat fn-call happens, so `CALL_STACK` is
// empty/irrelevant at expand time). `expand_macro_call` (src/macros/expand.rs)
// already has that span in scope (`call_site_span`) for every macro
// invocation it expands; it pushes it here via `MacroCallSiteGuard` before
// evaluating the macro body, and the guard pops it on scope exit. A stack
// (not a single cell) because macro expansion nests: expanding macro A's
// body can itself expand a call to macro B, and `macro-call-site` used
// inside B's body must read B's own invocation span (the top), not A's.
//
// Each entry carries the invocation's call-site span AND the NAME of the macro
// being expanded (its full keyword-path, e.g. `:wat::holon::Subtract`). The
// name becomes the resulting `:wat::kernel::Frame`'s `symbol`: at expand time
// there is no enclosing runtime fn, but there IS a known macro — so the frame's
// `symbol` is the macro's own name, never absent (the Frame's fields are all
// non-`Option`; a value is always known here).
thread_local! {
    static MACRO_CALL_SITE: std::cell::RefCell<Vec<(Span, String)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Scope guard that pushes the current macro invocation's call-site span +
/// macro name on construction and pops it on drop. Mirrors [`FrameGuard`] but
/// tracks expand-time macro-invocation spans instead of runtime call frames.
#[must_use = "MacroCallSiteGuard must be bound to a local (let _g = ...); dropping it immediately pops the span"]
pub(crate) struct MacroCallSiteGuard;

impl MacroCallSiteGuard {
    pub(crate) fn push(call_site_span: Span, macro_name: String) -> Self {
        MACRO_CALL_SITE.with(|s| {
            s.borrow_mut().push((call_site_span, macro_name));
        });
        MacroCallSiteGuard
    }
}

impl Drop for MacroCallSiteGuard {
    fn drop(&mut self) {
        MACRO_CALL_SITE.with(|s| {
            s.borrow_mut().pop();
        });
    }
}

/// Read the innermost (top-of-stack) macro invocation's call-site span + macro
/// name, if any macro expansion is currently in progress on this thread. `None`
/// means `:wat::kernel::macro-call-site` was reached outside macro expansion
/// (e.g. evaluated directly at runtime) — the caller should refuse it, not
/// fabricate a Frame (mirrored now by `call-site`, which also refuses on an
/// empty runtime stack rather than masking with a fabricated value).
pub(crate) fn current_macro_call_site() -> Option<(Span, String)> {
    MACRO_CALL_SITE.with(|s| s.borrow().last().cloned())
}

// ─── Excursus 003 D3 — one frame shape, whether it came from wat or Rust ──────
//
// `:wat::kernel::FrameKind` (`wat/kernel/diagnostics.wat`) names WHERE a frame came
// from: a wat call-stack entry (`:Wat`) or the one Rust site that raised the error
// carrying it (`:Rust`). Generated from the wat `defenum` — wat is the source of
// truth (mirrors `DefinedIn`/`Layer`/`Kind` in `src/intrinsic/mod.rs`).
::wat_source_derive::wat_enum_from!(
    pub enum FrameKind,
    "wat/kernel/diagnostics.wat",
    ":wat::kernel::FrameKind"
);

impl crate::intrinsic::ToEnumValue for FrameKind {
    const WAT_TYPE_PATH: &'static str = <FrameKind>::WAT_TYPE_PATH;
    fn variant_str(&self) -> &'static str {
        self.as_str()
    }
}

/// EDN tag for a nullary `FrameKind` variant: `#wat.kernel/FrameKind.Wat {}` /
/// `#wat.kernel/FrameKind.Rust {}` — the same `<Enum>.<Variant>` shape every other
/// nullary enum in this crate renders (`#wat.core/Option.None {}`, `wat/core.wat`'s
/// `:purity :wat::runtime::Purity.Pure` bare-keyword convention). Hand-written rather
/// than routed through `Value::Enum`'s renderer: this fn serves `Frame::to_edn`, which
/// builds `OwnedValue` directly from Rust data (a `RuntimeError`/`AssertionPayload`
/// has no live `Value`/`TypeRegistry` in hand at the point it renders its frames).
impl crate::edn::contract::ToEdn for FrameKind {
    fn to_edn(&self) -> wat_edn::OwnedValue {
        wat_edn::OwnedValue::Tagged(
            wat_edn::Tag::ns("wat.kernel", format!("FrameKind.{}", self.as_str())),
            Box::new(wat_edn::OwnedValue::Map(vec![])),
        )
    }
}

/// The `symbol` marker for a `:Rust` frame — mirrors [`ANON_FN_SYMBOL`]'s role for an
/// anonymous wat fn: `symbol` is a mandatory, non-`Option` field (arc 109), and a
/// `:Rust` frame has no wat keyword-path name to put there. Honest rather than
/// fabricated: this says plainly "this frame's origin is Rust, not a wat symbol" —
/// the frame's `span` (file/line/col of the `#[track_caller]` site) is where the real
/// identifying information lives.
pub(crate) const RUST_FRAME_SYMBOL: &str = "<rust>";

/// One frame in a captured trace — a wat call-stack entry or the Rust site that raised
/// the error. Excursus 003 D3: ONE shape and ONE builder for both, so a consumer
/// (`RuntimeError`'s frames, `AssertionPayload`'s frames) never has to special-case
/// which door a frame came through. `pub` (not `pub(crate)`): `AssertionPayload`
/// (`src/assertion.rs`) is itself a `pub` struct with a `pub frames: Vec<Frame>`
/// field, so `Frame` must be reachable at the same visibility.
#[derive(Debug, Clone)]
pub struct Frame {
    pub symbol: String,
    pub span: Span,
    pub kind: FrameKind,
}

impl From<FrameInfo> for Frame {
    fn from(fi: FrameInfo) -> Self {
        Frame {
            symbol: fi.callee_path,
            span: fi.call_span,
            kind: FrameKind::Wat,
        }
    }
}

impl Frame {
    /// The ONE Rust frame D3 asks for: the `#[track_caller]` view of whoever wrote
    /// the call to `RuntimeError::new` — "like clojure has java in its traces". `end`
    /// is always `None`: Rust's `Location` knows only where the call began, never
    /// where it ends (the same `end`-is-`Option` distinction D1 drew for `Span`).
    ///
    /// ⚠ Records the Rust function that CALLED `new`, not `new`'s own body — so where
    /// a helper builds the error for MANY callers (e.g. `eval_opt_string`,
    /// `src/assertion.rs`), that helper is the recorded site, not each of ITS
    /// callers. Accepted per BRIEF-envelope-step-2-every-error-carries-its-frames.md.
    pub(crate) fn rust_site(loc: &std::panic::Location<'_>) -> Self {
        Frame {
            symbol: RUST_FRAME_SYMBOL.into(),
            span: Span::new(
                std::sync::Arc::new(loc.file().to_string()),
                loc.line() as i64,
                loc.column() as i64,
            ),
            kind: FrameKind::Rust,
        }
    }

    /// Render one frame to `#wat.kernel/Frame {:symbol :span :kind}` — the ONE EDN
    /// builder for a frame, shared by `RuntimeError`'s `:frames` (`src/edn/error.rs`)
    /// and `AssertionPayload`'s `:frames` (`src/panic_hook.rs`), so the two capture
    /// paths D3's brief asks to unify render identically rather than by coincidence.
    pub(crate) fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::edn::contract::ToEdn;
        wat_edn::OwnedValue::Tagged(
            wat_edn::Tag::ns("wat.kernel", "Frame"),
            Box::new(wat_edn::OwnedValue::Map(vec![
                (
                    wat_edn::OwnedValue::Keyword(wat_edn::Keyword::new("symbol")),
                    wat_edn::OwnedValue::String(std::borrow::Cow::Owned(self.symbol.clone())),
                ),
                (
                    wat_edn::OwnedValue::Keyword(wat_edn::Keyword::new("span")),
                    self.span.to_edn(),
                ),
                (
                    wat_edn::OwnedValue::Keyword(wat_edn::Keyword::new("kind")),
                    self.kind.to_edn(),
                ),
            ])),
        )
    }
}

// ─── Excursus 003 D3 — the cap ─────────────────────────────────────────────────
//
// Non-tail recursion can reach ~110,000 frames before the Rust stack overflows
// (the-little-wat F-099). MEASURED (`cargo nextest run --release measure_snapshot_cost
// -- --no-capture`, `src/value/frame.rs` `#[cfg(test)] mod cap_measurement`, six-run
// median-of-11, release build, this host — `cargo nextest run --release -p wat
// cap_measurement --no-capture`, `cap_measurement` mod below; run TWICE to check the
// spread before trusting either):
//
//   depth        snapshot_call_stack() cost (full clone, pre-cap)     run 1      run 2
//   10           ~600-700ns                                          567ns      708ns
//   1,000        ~90-113µs                                           90,392ns   112,646ns
//   100,000      ~4.1ms                                              4,094,624ns 4,104,471ns
//
// Cost is linear in depth (a `Vec` clone of every `FrameInfo`, one `Arc` clone each)
// — a 100,000-deep non-tail recursion that errors pays ~4.1ms PER RAISE to copy
// frames nobody asked to see 100,000 of. **This is why `capped_wat_frames` does NOT
// call `snapshot_call_stack()`** — an earlier draft of this fn did, measured, and was
// wrong: it paid the full linear cost above and then threw most of it away, capping
// the OUTPUT while leaving the COST exactly as depth-dependent as the thing being
// capped. Reading `CALL_STACK` directly and cloning only the innermost N + outermost
// M elements bounds the cost itself to O(cap), independent of depth — MEASURED (same
// method, `measure_capped_snapshot_cost_by_depth`):
//
//   depth        capped_wat_frames() cost                            run 1      run 2
//   10           ~240ns (n <= cap, no capping needed)                239ns      —
//   1,000        ~1.6µs (bounded by the 40-frame cap, not depth)     1,635ns    —
//   100,000      ~1.6µs (SAME — does not grow with depth)            1,559ns    —
//
// depth-1,000 and depth-100,000 land within noise of each other (unlike the ~46x
// growth in the uncapped numbers above) — the fix does what its doc claims.
//
// Cap chosen as innermost 32 + outermost 8 (40 frames): 32 nested calls is already
// deep enough to show the whole call chain a person would read in one screen; the 8
// outermost anchor "where this ultimately started" (`:user::main` and its first few
// callees) without paying to walk the middle of a 100,000-frame stack.
pub(crate) const FRAME_CAP_INNERMOST: usize = 32;
pub(crate) const FRAME_CAP_OUTERMOST: usize = 8;

/// Snapshot the wat call stack for a new `RuntimeError`, capped to innermost
/// [`FRAME_CAP_INNERMOST`] + outermost [`FRAME_CAP_OUTERMOST`] frames. Returns the
/// (possibly capped) frames, innermost first, plus the count elided from the middle
/// (0 when the stack fit under the cap).
///
/// ⚠ Deliberately reads `CALL_STACK` directly rather than calling
/// [`snapshot_call_stack`] — that fn clones the WHOLE stack (measured linear in depth,
/// see the doc comment on [`FRAME_CAP_INNERMOST`]), which would pay the full
/// depth-dependent cost this cap exists to avoid before throwing the middle away.
/// Cloning only the `FRAME_CAP_INNERMOST + FRAME_CAP_OUTERMOST` elements this fn
/// actually keeps bounds the COST to the cap, not just the rendered output.
pub(crate) fn capped_wat_frames() -> (Vec<Frame>, usize) {
    CALL_STACK.with(|s| {
        let stack = s.borrow(); // storage order: oldest (outermost) first, newest (innermost) last
        let n = stack.len();
        let cap = FRAME_CAP_INNERMOST + FRAME_CAP_OUTERMOST;
        if n <= cap {
            // Fits whole — innermost-first order is the reverse of storage order.
            return (stack.iter().rev().cloned().map(Frame::from).collect(), 0);
        }
        // Innermost N: the last N elements of storage, innermost (top) first.
        let mut frames: Vec<Frame> = stack[n - FRAME_CAP_INNERMOST..]
            .iter()
            .rev()
            .cloned()
            .map(Frame::from)
            .collect();
        // Outermost M: the first M elements of storage, continuing the same
        // innermost-first convention (the one closest to the elided middle comes
        // first, the true outermost — e.g. :user::main — comes last).
        frames.extend(
            stack[..FRAME_CAP_OUTERMOST]
                .iter()
                .rev()
                .cloned()
                .map(Frame::from),
        );
        let elided = n - FRAME_CAP_INNERMOST - FRAME_CAP_OUTERMOST;
        (frames, elided)
    })
}

#[cfg(test)]
mod cap_measurement {
    //! Excursus 003 D3 — the cap-choice measurement the brief owes (not a regression
    //! gate; a recorded instrument). Pushes N synthetic frames directly onto
    //! `CALL_STACK` (no actual wat recursion — avoids the ~110,000-frame Rust stack
    //! overflow the-little-wat F-099 measured) and times `snapshot_call_stack()`.
    //! Run with `cargo nextest run --release -p wat measure_snapshot_cost_by_depth
    //! -- --no-capture` to see the printed numbers; the constants above are this
    //! test's output, hand-copied into the doc comment as the measured record.
    use super::*;
    use std::time::Instant;

    fn push_n(n: usize) {
        CALL_STACK.with(|s| {
            let mut stack = s.borrow_mut();
            stack.clear();
            for i in 0..n {
                stack.push(FrameInfo {
                    callee_path: format!(":user::fn-{i}"),
                    call_span: Span::new(std::sync::Arc::new("bench.wat".to_string()), i as i64, 0),
                });
            }
        });
    }

    fn median_ns(depth: usize, runs: usize) -> u128 {
        push_n(depth);
        let mut samples = Vec::with_capacity(runs);
        for _ in 0..runs {
            let start = Instant::now();
            let snap = snapshot_call_stack();
            let elapsed = start.elapsed();
            std::hint::black_box(&snap);
            samples.push(elapsed.as_nanos());
        }
        CALL_STACK.with(|s| s.borrow_mut().clear());
        samples.sort_unstable();
        samples[runs / 2]
    }

    fn median_capped_ns(depth: usize, runs: usize) -> u128 {
        push_n(depth);
        let mut samples = Vec::with_capacity(runs);
        for _ in 0..runs {
            let start = Instant::now();
            let capped = capped_wat_frames();
            let elapsed = start.elapsed();
            std::hint::black_box(&capped);
            samples.push(elapsed.as_nanos());
        }
        CALL_STACK.with(|s| s.borrow_mut().clear());
        samples.sort_unstable();
        samples[runs / 2]
    }

    /// Confirms the fix `capped_wat_frames`'s own doc comment claims: reading
    /// `CALL_STACK` directly and cloning only the cap bounds the cost to O(cap),
    /// independent of depth — unlike `snapshot_call_stack()` above, this should NOT
    /// grow ~40x from depth 1,000 to depth 100,000.
    #[test]
    fn measure_capped_snapshot_cost_by_depth() {
        for depth in [10usize, 1_000, 100_000] {
            let ns = median_capped_ns(depth, 11);
            eprintln!("depth={depth} median_capped_ns={ns}");
        }
    }

    #[test]
    fn measure_snapshot_cost_by_depth() {
        for depth in [10usize, 1_000, 100_000] {
            let ns = median_ns(depth, 11);
            eprintln!("depth={depth} median_snapshot_ns={ns}");
        }
    }
}
