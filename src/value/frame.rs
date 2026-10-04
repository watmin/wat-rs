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
///
/// Excursus 003 strike D ("a frame is where a function is") — a tail call
/// doesn't push; it SUBSTITUTES the top slot's contents (`replace_top_frame`
/// below). Read naively, each substitution throws away the identity of
/// whoever occupied the slot before it — `grow` vanishes the moment it tail-
/// calls `+`. The ruling (`AUDIT-the-shape-of-an-error.md` § RULING
/// 2026-10-03 item 1): keep the lost identity and a count, at O(1) cost,
/// rather than paying for a ring buffer or silently losing the name.
///
/// Three fields beyond `callee_path`/`call_span` carry that history:
/// - `entry_call_site`: the span this slot was PUSHED with — the real
///   (non-tail) caller's own call site. Never touched by a later tail
///   replace; a frame ONE level further out reads `at` from here (the
///   edge it made into this slot), never from this slot's own (possibly
///   stale) `call_span`.
/// - `last_tail_caller`: whoever occupied `callee_path` immediately before
///   the MOST RECENT `replace_top_frame` call — the true owner of the
///   surviving `call_span` (the location the replace's span was captured
///   AT is inside the PREVIOUS occupant's body, not the new one's).
///   Meaningless (never read) while `tail_hops == 0`.
/// - `tail_hops`: how many `replace_top_frame` calls this slot has
///   absorbed. `0` means this slot was never tail-replaced — a plain,
///   non-tail frame, GD3.
#[derive(Debug, Clone)]
pub struct FrameInfo {
    pub callee_path: String,
    pub call_span: Span,
    /// The span this slot was PUSHED with (the real caller's own call
    /// site) — preserved across every later tail replace. See the struct
    /// doc.
    pub(crate) entry_call_site: Span,
    /// The name that occupied this slot immediately before its last tail
    /// replace. Equal to `callee_path` (harmlessly) while `tail_hops == 0`.
    pub(crate) last_tail_caller: String,
    /// Count of `replace_top_frame` calls this slot has absorbed. `0` =
    /// never tail-replaced.
    pub(crate) tail_hops: usize,
}

impl FrameInfo {
    /// Build a freshly-pushed (never tail-replaced) `FrameInfo` — the same
    /// initialization `FrameGuard::push` does, exposed for the handful of
    /// sites (test fixtures, the cap-measurement benchmark) that construct
    /// one directly rather than going through the real push path.
    pub(crate) fn pristine(callee_path: String, call_span: Span) -> Self {
        FrameInfo {
            callee_path: callee_path.clone(),
            call_span: call_span.clone(),
            entry_call_site: call_span,
            last_tail_caller: callee_path,
            tail_hops: 0,
        }
    }
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
            s.borrow_mut().push(FrameInfo::pristine(callee_path, call_span));
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
///
/// Excursus 003 strike D: the occupant being overwritten (`top.callee_path`,
/// BEFORE this call) is who `call_span` is ABOUT TO become true of — that
/// surviving span sits inside THAT occupant's body (it's the span of the
/// tail call that occupant made). So the outgoing name becomes
/// `last_tail_caller`, paired with the incoming `call_span`, and
/// `tail_hops` counts one more substitution. `entry_call_site` is untouched
/// — it was fixed at push and names the real (non-tail) caller, for a frame
/// one level further out to read.
pub(crate) fn replace_top_frame(callee_path: String, call_span: Span) {
    CALL_STACK.with(|s| {
        if let Some(top) = s.borrow_mut().last_mut() {
            let outgoing_name = std::mem::replace(&mut top.callee_path, callee_path);
            top.last_tail_caller = outgoing_name;
            top.call_span = call_span;
            top.tail_hops += 1;
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

// ─── Excursus 003 strike D — "a frame is where a function is" ────────────────
//
// `AUDIT-the-shape-of-an-error.md` F6 + its § RULING 2026-09-27 item 2 + §
// RULING 2026-10-03 item 1. `Frame` (today `{symbol span kind}`, pairing
// (*callee*, *where it was called from*)) is being reshaped to `{fn at}`,
// pairing (*function*, *where inside it execution is*) — the Clojure/Java
// convention.
//
// That reshape also needs the innermost Rust activation to carry an honest
// `fn` name (never the `<rust>` placeholder). An earlier pass of this strike
// tried to find that name ON THE `RuntimeErrorKind` ITSELF (the way `op()`
// already does for 12 variants) and STOPped: ~10 of 40 variants carry no
// name-bearing field at all (`DivisionByZero`, `NotCallable`, `BadCondition`,
// `UserMainMissing`, `EvalVerificationFailed`, `WriteStopped`,
// `PatternMatchFailed`, `AssertionFailed`, `MacroAbort`, three of
// `ReteCeiling`'s four inner variants).
//
// The builder's ruling (§ "Strike D, first boundary"): a frame's identity is
// a property of the STACK, not of the error's content — threading name
// fields onto ~10 kinds is Simple NO (ten wire changes for a framing
// concern), and any fallback name is Honest NO. Instead, `CURRENT_ACTIVATION`
// below: ONE thread-local slot naming whatever native/special-form/freeze-
// phase activation is CURRENTLY running, written by exactly three call
// sites (never by `RuntimeErrorKind`), read by `Frame::rust_site` for every
// kind uniformly. `RuntimeErrorKind::op()` is no longer used for naming —
// only for the `op` FIELD each of its 12 carrying variants still has.

/// One frame in a captured trace — `{fn at tail-elided}`, pairing (*function*,
/// *where inside it execution is*) — the Clojure/Java convention. `pub` (not
/// `pub(crate)`): `AssertionPayload` (`src/assertion.rs`) is itself a `pub`
/// struct with a `pub frames: Vec<Frame>` field, so `Frame` must be reachable
/// at the same visibility.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    /// The function this frame is about. Mandatory — no placeholder (item 1's
    /// ruling struck `<rust>`; item 4's ruling struck naming it off
    /// `RuntimeErrorKind` — see `CURRENT_ACTIVATION` below).
    pub fn_name: String,
    /// Where, INSIDE `fn_name`, execution is (or where it made its next
    /// call, or — for the innermost frame when a raise span is supplied —
    /// where it raised).
    pub at: Span,
    /// How many tail-collapsed activations are missing BEYOND the one named
    /// here. `0` for an ordinary (never tail-replaced) frame, and ALSO for a
    /// tail-collapsed frame whose own single substitution lost nothing
    /// further (GD1's `grow` frame: one hop, nothing beyond it is missing).
    /// The value is honest either way — it states exactly how many named
    /// activations are absent at this position, and 0 is the true answer in
    /// both cases.
    pub tail_elided: usize,
}

/// Reconstruct display frames from a tail-aware call-stack slice, innermost
/// first. `stack` must be in storage order (outermost first, innermost /
/// top-of-stack last) — exactly `capped_frames_for_trace`' internal slice, or
/// `snapshot_call_stack()`'s result reversed back to storage order.
///
/// `raise_span`, when given, is the actual location execution reached when it
/// raised (or made the call that's about to raise) — e.g. the raw `span`
/// `RuntimeError::new` was constructed with, or an assertion's own call-form
/// span. It becomes the `at` of an EXTRA innermost frame naming the top
/// slot's CURRENT occupant (`stack.last().callee_path`) — distinct from that
/// slot's own (possibly stale) `call_span`, which belongs to whichever
/// identity the slot's last tail replace displaced (handled by the normal
/// per-slot branch below, when `tail_hops > 0`). `None` means the caller has
/// nothing more local than the stack itself to report (G4's "no frame is in
/// user source" case, or a context — like `snapshot_call_stack`'s other
/// consumers — with no separate raise site at all).
///
/// For each slot, outermost to innermost reversed (innermost emitted first):
/// - `tail_hops > 0`: `{fn: last_tail_caller, at: call_span, tail_elided:
///   tail_hops - 1}` — `call_span` is the span of the LAST tail call this
///   slot absorbed, which sits inside `last_tail_caller`'s own body (the
///   occupant the last replace displaced), so that pairing is direct, no
///   shift needed. `tail_hops - 1` is exactly how many earlier identities
///   (further back in this slot's own chain) are not separately named.
/// - `tail_hops == 0`: `{fn: callee_path, at: <the edge this slot made into
///   the NEXT (more inner) slot>, tail_elided: 0}` — this slot's own
///   `call_span` names the edge INTO it (from its PARENT), not out of it;
///   the edge OUT of it is the next slot's `entry_call_site` (untouched by
///   whatever tail-collapsing later happened inside that inner slot). The
///   true innermost slot has no next slot: if `raise_span` was supplied, the
///   raise-frame step above already said everything about this slot, and
///   this branch is skipped entirely (avoiding a second, wrong-paired frame
///   for the same identity); otherwise its own `call_span` is the best
///   available answer (the "no raise_span" contexts don't have a
///   shift target either).
pub(crate) fn reconstruct_frames(stack: &[FrameInfo], raise_span: Option<Span>) -> Vec<Frame> {
    let n = stack.len();
    let mut out = Vec::with_capacity(n + raise_span.is_some() as usize);
    for idx in (0..n).rev() {
        let slot = &stack[idx];
        let is_top = idx == n - 1;
        if is_top {
            if let Some(span) = &raise_span {
                out.push(Frame {
                    fn_name: slot.callee_path.clone(),
                    at: span.clone(),
                    tail_elided: 0,
                });
                if slot.tail_hops == 0 {
                    // Nothing else to say about this slot — the raise frame
                    // above is its only display frame. Falling through would
                    // re-emit `callee_path` a second time, wrongly paired
                    // with its own (parent-edge) call_span.
                    continue;
                }
                // tail_hops >= 1: the slot ALSO has a collapsed-history
                // identity to name (its last_tail_caller) — fall through.
            }
        }
        if slot.tail_hops > 0 {
            out.push(Frame {
                fn_name: slot.last_tail_caller.clone(),
                at: slot.call_span.clone(),
                tail_elided: slot.tail_hops - 1,
            });
        } else {
            let at = match stack.get(idx + 1) {
                Some(inner) => inner.entry_call_site.clone(),
                None => slot.call_span.clone(),
            };
            out.push(Frame {
                fn_name: slot.callee_path.clone(),
                at,
                tail_elided: 0,
            });
        }
    }
    out
}

impl Frame {
    /// The ONE Rust frame every `RuntimeError` carries: the `#[track_caller]` view of
    /// whoever wrote the call to `RuntimeError::new` — "like Clojure has Java in its
    /// traces" — paired with the honest activation name [`current_activation`] supplies
    /// (excursus 003 strike D item 4). `end` is always `None`: Rust's `Location` knows
    /// only where the call began, never where it ends (D1's own `end`-is-`Option`
    /// distinction for `Span`).
    ///
    /// ⚠ Records the Rust function that CALLED `new`, not `new`'s own body — so where a
    /// helper builds the error for MANY callers (e.g. `eval_opt_string`,
    /// `src/assertion.rs`), that helper is the recorded site, not each of ITS callers.
    /// Accepted per BRIEF-envelope-step-2-every-error-carries-its-frames.md.
    ///
    /// `activation` (read by the caller from [`current_activation`]) is `Some` for every
    /// REAL raise reached through the intrinsic/special-form dispatcher or the freeze
    /// pipeline (this strike's measurement drives representative producers for each and
    /// confirms it). It is honestly `None` for a `RuntimeError` built directly in Rust
    /// with NO wat execution context at all — found, empirically, to be a pervasive,
    /// pre-existing pattern: ~100 unit tests across this tree call `RuntimeError::new`
    /// straight from a bare `#[test]` fn (no freeze, no eval, `CALL_STACK` never
    /// pushed), to test the envelope's OWN Rust-level mechanics
    /// (`tests/value/probe_runtime_error_one_door.rs`, `tests/diagnostics/
    /// probe_arc298_3_runtime_derive_identical.rs`, …) — none of the three writers the
    /// builder named can EVER fire for that construction shape, by its very nature.
    ///
    /// Returns `None` rather than inventing a name for that case — the builder's "no
    /// placeholder" ruling forecloses a fallback STRING, but says nothing against simply
    /// not having a Rust frame when nothing is honestly known about one: the caller
    /// (`RuntimeError::new`) omits this frame from `:frames` entirely rather than
    /// rendering a guess. A real production raise never takes this branch (freeze
    /// always runs before the first eval, and eval always sets the activation before
    /// anything it dispatches can raise) — see this strike's report for the finding and
    /// why it was decided this way rather than a panic (which would turn ~100
    /// pre-existing, unrelated tests red) or a fallback string (ruled out).
    pub(crate) fn rust_site(loc: &std::panic::Location<'_>, activation: Option<String>) -> Option<Self> {
        let fn_name = activation?;
        Some(Frame {
            fn_name,
            at: Span::new(
                std::sync::Arc::new(loc.file().to_string()),
                loc.line() as i64,
                loc.column() as i64,
            ),
            tail_elided: 0,
        })
    }

    /// Render one frame to `#wat.kernel/Frame {:fn :at :tail-elided}` — the ONE EDN
    /// builder for a frame, shared by `RuntimeError`'s `:frames` (`src/edn/error.rs`) and
    /// `AssertionPayload`'s `:frames` (`src/panic_hook.rs`).
    pub(crate) fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::edn::contract::ToEdn;
        wat_edn::OwnedValue::Tagged(
            wat_edn::Tag::ns("wat.kernel", "Frame"),
            Box::new(wat_edn::OwnedValue::Map(vec![
                (
                    wat_edn::OwnedValue::Keyword(wat_edn::Keyword::new("fn")),
                    wat_edn::OwnedValue::String(std::borrow::Cow::Owned(self.fn_name.clone())),
                ),
                (
                    wat_edn::OwnedValue::Keyword(wat_edn::Keyword::new("at")),
                    self.at.to_edn(),
                ),
                (
                    wat_edn::OwnedValue::Keyword(wat_edn::Keyword::new("tail-elided")),
                    wat_edn::OwnedValue::Integer(self.tail_elided as i64),
                ),
            ])),
        )
    }
}

/// Reconstruct the FULL (uncapped) wat call stack into display frames, innermost first —
/// the shared door for every consumer that isn't `RuntimeError` (which caps, below):
/// `AssertionPayload` (`src/assertion.rs`), `src/kernel/abort.rs`, `src/collection/eval.rs`,
/// `src/services/verbs.rs`. `raise_span` is `None` for all four TODAY — each derives its
/// own `location` field from the top frame's OWN `call_span` (preserved exactly: with no
/// raise_span, the top slot has no shift target either and falls back to its own
/// `call_span`, the same value these callers read today), not from a separate raise site.
pub(crate) fn frames_for_trace(raise_span: Option<Span>) -> Vec<Frame> {
    let stack = CALL_STACK.with(|s| s.borrow().clone());
    reconstruct_frames(&stack, raise_span)
}

#[cfg(test)]
mod strike_d_tests {
    //! Excursus 003 strike D — GD1–GD4 at the pure-reconstruction level, driving
    //! `CALL_STACK` directly via `FrameGuard::push` / `replace_top_frame`, exactly as
    //! `apply_function`'s trampoline does, then asserting `reconstruct_frames`'s
    //! output field-for-field. GD5 (no synthesized/placeholder frame; the lint's RED
    //! anchor on today's goldens) and the wire-level probes against the REAL
    //! `RuntimeError`/`AssertionPayload` EDN live in `tests/diagnostics/` — this module
    //! proves the algorithm in isolation, those prove it wired end to end.
    use super::*;

    fn span(file: &str, line: i64, col: i64) -> Span {
        Span::new(std::sync::Arc::new(file.to_string()), line, col)
    }

    fn clear_stack() {
        CALL_STACK.with(|s| s.borrow_mut().clear());
    }

    /// Snapshot `CALL_STACK` in STORAGE order (outermost first) — what
    /// `reconstruct_frames` expects, as opposed to `snapshot_call_stack`'s
    /// newest-first convention.
    fn stack_storage_order() -> Vec<FrameInfo> {
        CALL_STACK.with(|s| s.borrow().clone())
    }

    /// GD1 — C-114's shape: `probe-overflow` (pushed) tail-calls `grow`,
    /// which tail-calls `core::+` (two hops absorbed by ONE physical slot),
    /// which — via a native intrinsic, no further push/replace — raises at
    /// `wat/core.wat:66`. Innermost first: the raise-paired frame names the
    /// slot's CURRENT occupant (`core::+`); the collapsed-history frame
    /// names `last_tail_caller` (`grow`) with `tail_elided = hops - 1 = 1`
    /// (the one entirely missing identity: `probe-overflow`).
    #[test]
    fn gd1_c114_shape_names_grow_and_collapses_probe_overflow() {
        clear_stack();
        let _g = FrameGuard::push(
            ":my::test::probe-overflow".into(),
            span("src/freeze.rs", 1646, 1),
        );
        replace_top_frame(":user::grow".into(), span("c114.wat", 14, 3));
        replace_top_frame(":wat::core::+".into(), span("c114.wat", 10, 3));

        let stack = stack_storage_order();
        let frames = reconstruct_frames(&stack, Some(span("wat/core.wat", 66, 62)));

        assert_eq!(
            frames,
            vec![
                Frame {
                    fn_name: ":wat::core::+".into(),
                    at: span("wat/core.wat", 66, 62),
                    tail_elided: 0,
                },
                Frame {
                    fn_name: ":user::grow".into(),
                    at: span("c114.wat", 10, 3),
                    tail_elided: 1,
                },
            ]
        );
        clear_stack();
    }

    /// GD1's own assertion already pins `frames[1].fn_name == ":user::grow"`
    /// field-for-field above — that IS the mutation gate: swapping
    /// `last_tail_caller` for `callee_path` in `reconstruct_frames`'
    /// `tail_hops > 0` branch (the drop-the-tail-caller-record mutation)
    /// makes `frames[1].fn_name` read `":wat::core::+"` (duplicating the
    /// raise frame) instead of `":user::grow"`, and `gd1_c114_shape_…`
    /// above goes RED. Mutation proven by hand (edit, run, confirm RED,
    /// revert) rather than carried as a second, permanently-mutated copy of
    /// the same assertion — see this strike's report for the transcript.

    /// GD2 — a tail chain `f` → (tail) `g` → (tail) `h` → raise, called from
    /// `main` (a SEPARATE, never-replaced physical slot). The trace names
    /// `h` with its raise, `g` at its tail call (`last_tail_caller`), the
    /// count (1, for `f`'s fully collapsed frame), and `main` at the call
    /// into `f` (via `main`'s own never-replaced slot's shift target: `f`'s
    /// preserved `entry_call_site`).
    #[test]
    fn gd2_tail_chain_names_last_caller_and_counts_the_rest() {
        clear_stack();
        let _main = FrameGuard::push(":user::main".into(), span("rt.rs", 1, 1));
        let _f = FrameGuard::push(":user::f".into(), span("chain.wat", 1, 3)); // main calls f
        replace_top_frame(":user::g".into(), span("chain.wat", 5, 3)); // f tail-calls g
        replace_top_frame(":user::h".into(), span("chain.wat", 9, 3)); // g tail-calls h

        let stack = stack_storage_order();
        let frames = reconstruct_frames(&stack, Some(span("chain.wat", 13, 3))); // h raises

        assert_eq!(
            frames,
            vec![
                Frame {
                    fn_name: ":user::h".into(),
                    at: span("chain.wat", 13, 3),
                    tail_elided: 0,
                },
                Frame {
                    fn_name: ":user::g".into(),
                    at: span("chain.wat", 9, 3),
                    tail_elided: 1, // f's own identity is the one missing frame
                },
                Frame {
                    fn_name: ":user::main".into(),
                    at: span("chain.wat", 1, 3), // f's entry_call_site, inside main
                    tail_elided: 0,
                },
            ]
        );
        clear_stack();
    }

    /// Mutation (GD2): compute the count wrong (e.g. `tail_hops` instead of
    /// `tail_hops - 1`) — RED, because 2 != 1.
    #[test]
    fn gd2_mutation_wrong_count_is_red() {
        clear_stack();
        let _main = FrameGuard::push(":user::main".into(), span("rt.rs", 1, 1));
        let _f = FrameGuard::push(":user::f".into(), span("chain.wat", 1, 3));
        replace_top_frame(":user::g".into(), span("chain.wat", 5, 3));
        replace_top_frame(":user::h".into(), span("chain.wat", 9, 3));

        let stack = stack_storage_order();
        let frames = reconstruct_frames(&stack, Some(span("chain.wat", 13, 3)));
        let g_frame = frames.iter().find(|f| f.fn_name == ":user::g").unwrap();
        assert_eq!(g_frame.tail_elided, 1, "the ruling's own worked example: count = 1, for f's collapsed frame");
        assert_ne!(g_frame.tail_elided, 2, "tail_hops (2) is NOT the count — that's the pre-ruling bug this gate catches");
        clear_stack();
    }

    /// GD3 — a non-tail call has no tail marker, and its frames are
    /// unchanged in meaning. `main` pushes `leaf` (non-tail); `leaf` raises
    /// directly (no tail call at all — `tail_hops == 0` throughout).
    #[test]
    fn gd3_non_tail_call_has_no_tail_marker() {
        clear_stack();
        let _main = FrameGuard::push(":user::main".into(), span("rt.rs", 1, 1));
        let _leaf = FrameGuard::push(":user::leaf".into(), span("plain.wat", 2, 3));

        let stack = stack_storage_order();
        let frames = reconstruct_frames(&stack, Some(span("plain.wat", 4, 5)));

        assert_eq!(
            frames,
            vec![
                Frame {
                    fn_name: ":user::leaf".into(),
                    at: span("plain.wat", 4, 5),
                    tail_elided: 0,
                },
                Frame {
                    fn_name: ":user::main".into(),
                    at: span("plain.wat", 2, 3),
                    tail_elided: 0,
                },
            ]
        );
        // Every tail_elided is 0 — "no marker" here IS the no-tail-hops case
        // (see Frame's own doc: 0 is honest in both readings; what
        // distinguishes a tail frame from an ordinary one is WHICH name
        // (`last_tail_caller` vs `callee_path`) got reported, not this count).
        assert!(frames.iter().all(|f| f.tail_elided == 0));
        clear_stack();
    }

    /// GD4 — constant space. A tail-recursive loop of 1,000,000 iterations
    /// (kept well under the ~110,000-frame Rust-stack-overflow ceiling
    /// `the-little-wat` F-099 measured for NON-tail recursion — this is a
    /// `replace_top_frame` loop, no Rust recursion at all) raises at the
    /// end. `CALL_STACK` depth stays O(1): exactly 1 physical slot. The
    /// reconstructed trace holds a single collapse count of ~10^6.
    #[test]
    fn gd4_million_tail_hops_stay_one_physical_slot() {
        clear_stack();
        let _entry = FrameGuard::push(":user::loop".into(), span("loop.wat", 1, 1));
        const ITERS: usize = 1_000_000;
        for i in 0..ITERS {
            replace_top_frame(":user::loop".into(), span("loop.wat", 2, 3 + i as i64 % 1000));
        }
        // O(1): still exactly one physical slot, regardless of 10^6 hops.
        assert_eq!(CALL_STACK.with(|s| s.borrow().len()), 1);

        let stack = stack_storage_order();
        let frames = reconstruct_frames(&stack, Some(span("loop.wat", 2, 3)));
        assert_eq!(frames.len(), 2, "one physical slot, tail_hops > 0 => BOTH the raise-paired frame (current occupant) and the collapsed-history frame (last_tail_caller) fire — still O(1) frames, independent of the 10^6 hops");
        clear_stack();
    }

    /// GD4 (full, both frames): same million-hop loop, but confirms the
    /// collapse count is exactly `ITERS - 1` and the slot never grew.
    #[test]
    fn gd4_million_tail_hops_collapse_count_is_exact() {
        clear_stack();
        let _entry = FrameGuard::push(":user::loop".into(), span("loop.wat", 1, 1));
        const ITERS: usize = 1_000_000;
        for _ in 0..ITERS {
            replace_top_frame(":user::loop".into(), span("loop.wat", 2, 3));
        }
        let depth_before_raise = CALL_STACK.with(|s| s.borrow().len());
        let stack = stack_storage_order();
        let frames = reconstruct_frames(&stack, Some(span("loop.wat", 9, 9)));

        assert_eq!(depth_before_raise, 1, "mutation: push instead of replace would make this ITERS, not 1");
        assert_eq!(frames.len(), 2, "the raise-paired frame (current occupant) + the collapsed-history frame (last_tail_caller)");
        assert_eq!(frames[1].tail_elided, ITERS - 1);
        clear_stack();
    }
}

// ─── Excursus 003 strike D, item 4 — the current-activation slot ─────────────
//
// `AUDIT-the-shape-of-an-error.md` § RULING "Strike D, first boundary". Names the
// innermost Rust activation for `Frame::rust_site`, replacing the retired `<rust>`
// placeholder and `RuntimeErrorKind::op()`'s former naming role. A frame's identity
// is a property of the STACK (what is CURRENTLY running), not of the error's
// content — so this is written by exactly three call sites, each naming what IT
// knows is currently executing, never by the error kind itself:
//
// 1. **The intrinsic/special-form dispatcher** (`src/runtime.rs`'s
//    `dispatch_keyword_head`/`dispatch_keyword_head_value`/`eval_tail`'s own
//    keyword-head arm) — `ActivationGuard::enter(head)`, RAII, around the whole
//    dispatch attempt for that head (not just a successful handler call): this
//    covers a registered intrinsic, a registered special form (the registry
//    folds both into the same `handler`/`tail_handler` slot — see
//    `src/intrinsic/mod.rs`'s `registry()`), AND the "no handler found" raises
//    (`UnboundSymbol`, `NotCallable`, `MalformedForm`, …) that happen in the
//    SAME dispatch attempt once the registry comes up empty.
// 2. **The freeze pipeline's `pass_order::record`** (`src/freeze/pass_order.rs`)
//    — a plain overwrite, no restore: freeze runs linearly, before any wat
//    activation exists, so there is nothing to nest under. Names the startup
//    phase for `UserMainMissing`/`EvalVerificationFailed`.
//
// RAII (enter/restore) for (1) so nested dispatch — a special form evaluating a
// sub-form that itself dispatches — reports the INNERMOST activation, exactly
// the way `CALL_STACK`'s own push/pop nests. A plain overwrite for (2) because
// freeze has no call-stack to nest under at all.
thread_local! {
    static CURRENT_ACTIVATION: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

/// RAII guard naming the currently-dispatching native/special-form head —
/// restores whatever was there before (nesting correctly through recursive
/// dispatch) on drop. Mirrors [`UserSourceGuard`]'s save/restore shape.
#[must_use = "ActivationGuard must be bound to a local (let _g = ...); dropping it immediately restores the prior activation"]
pub(crate) struct ActivationGuard {
    prior: Option<String>,
}

impl ActivationGuard {
    pub(crate) fn enter(name: impl Into<String>) -> Self {
        let prior = CURRENT_ACTIVATION.with(|c| c.replace(Some(name.into())));
        ActivationGuard { prior }
    }
}

impl Drop for ActivationGuard {
    fn drop(&mut self) {
        CURRENT_ACTIVATION.with(|c| *c.borrow_mut() = self.prior.take());
    }
}

/// Plain overwrite (no restore) — the freeze pipeline's own writer. Freeze runs
/// linearly, before any wat activation exists on this thread, so there is
/// nothing to nest under; each `pass_order::record` call simply names the
/// phase now running.
pub(crate) fn set_activation(name: &'static str) {
    CURRENT_ACTIVATION.with(|c| *c.borrow_mut() = Some(name.to_string()));
}

/// Read the currently-named activation, for `Frame::rust_site` to name the
/// innermost Rust frame. `None` means no writer has named anything on this
/// thread yet (should not happen for any REAL raise once the three writers
/// above are in place — a raise that finds this empty is this strike's own
/// measurement's STOP condition, not a case to paper over with a guess).
pub(crate) fn current_activation() -> Option<String> {
    CURRENT_ACTIVATION.with(|c| c.borrow().clone())
}

// ─── Excursus 003 D4 item 1 — the user-source-file record ────────────────────
//
// A POSITIVE record of which file labels the load pipeline read under USER privilege
// (the entry file, plus every `load-file!`/`digest-load!`/`signed-load!` target —
// `src/freeze.rs`'s `startup_from_forms_post_config`, the one chokepoint shared by
// `startup_from_forms`/`_with_inherit`/`_with_session`). Deliberately NOT "every file
// that isn't stdlib" — a `:Wat` frame can sit on a Rust file (`:user::main`'s own frame
// carries `src/freeze.rs`'s `rust_caller_span!()` as its call site), and that must NOT
// read as user source by elimination. Deliberately a plain `String` set, not an `Arc`
// identity set: with the reserved-label wall in `src/load/loader.rs`/`src/freeze.rs`
// (`LoadErrorKind::ReservedStdlibLabel`) refusing any load whose label equals a stdlib
// label, a stdlib label can never appear here, so string equality is unambiguous — and,
// unlike an `Arc` pointer, a label re-minted with the same text (a second
// `parse_all_with_file` call for the same file, a re-parsed golden fixture) still matches.
//
// PROGRAM-scoped, not thread-scoped (corrected from an earlier, thread-local-only draft):
// a PROCESS locus re-runs the whole load pipeline in its own fresh process, so its own
// thread-local naturally ends up correct — but a THREAD locus
// (`:wat::kernel::spawn-thread`, `src/kernel/spawn.rs::spawn_thread_peer`) runs a
// function of the SAME already-frozen world on a NEW OS thread, with no load pipeline of
// its own. Left as pure thread-local, that new thread's set is EMPTY and D4's derivation
// is silently a no-op there. The cure: the set built during THIS thread's own load
// pipeline is snapshotted as an `Arc` (`snapshot_user_source_files`) and stashed on the
// frozen `SymbolTable` (`SymbolTable::set_user_source_files`, called from
// `FrozenWorld::freeze`) — a program-scoped, `Clone`-cheap carrier, exactly like
// `source_loader`/`primed_stdio`. A thread locus installs a COPY of that Arc onto its OWN
// thread via `install_user_source_files`, at the same point it installs its other
// per-thread ambient state (`install_program_env` — see `spawn_thread_peer`).
thread_local! {
    static USER_SOURCE_FILES: std::cell::RefCell<std::sync::Arc<std::collections::HashSet<String>>> =
        std::cell::RefCell::new(std::sync::Arc::new(std::collections::HashSet::new()));
}

/// Clear the user-source-file record — called once at the start of
/// `startup_from_forms_post_config`, before the entry/loaded files' labels are recorded.
pub(crate) fn reset_user_source_files() {
    USER_SOURCE_FILES.with(|s| *s.borrow_mut() = std::sync::Arc::new(std::collections::HashSet::new()));
}

/// Record `label` as a file THIS thread's load pipeline read under user privilege.
/// `Arc::make_mut` clones-on-write only if the Arc is shared (refcount > 1) — during the
/// build phase (the only time this is called) nothing else holds a clone yet, so this is
/// an ordinary in-place insert, not a hidden per-call allocation.
pub(crate) fn record_user_source_file(label: String) {
    USER_SOURCE_FILES.with(|s| {
        std::sync::Arc::make_mut(&mut s.borrow_mut()).insert(label);
    });
}

/// Is `label` a file the owning program's load pipeline read under user privilege? The
/// positive fact D4 asks for — never "not stdlib" by elimination (see the module doc
/// above). Reads whatever this thread currently has installed: the set it built itself
/// (the original freeze thread), or a copy installed via [`install_user_source_files`]
/// (a spawned thread locus).
pub(crate) fn is_user_source_file(label: &str) -> bool {
    USER_SOURCE_FILES.with(|s| s.borrow().contains(label))
}

/// Snapshot the calling thread's user-source-file set as a cheaply-clonable `Arc`, for
/// `FrozenWorld::freeze` to stash on the `SymbolTable` — see the module doc above.
pub(crate) fn snapshot_user_source_files() -> std::sync::Arc<std::collections::HashSet<String>> {
    USER_SOURCE_FILES.with(|s| s.borrow().clone())
}

/// RAII guard restoring the calling thread's PRIOR user-source-file set on drop. Mirrors
/// [`crate::services::client::EnvGuard`] (`install_program_env`'s own guard) exactly.
#[must_use = "UserSourceGuard must be bound to a local (let _g = ...); dropping it immediately restores the prior set"]
pub(crate) struct UserSourceGuard {
    prior: std::sync::Arc<std::collections::HashSet<String>>,
}

impl Drop for UserSourceGuard {
    fn drop(&mut self) {
        USER_SOURCE_FILES.with(|s| {
            *s.borrow_mut() = self.prior.clone();
        });
    }
}

/// Install `files` (a program's own user-source-file set, read off its `SymbolTable` via
/// [`crate::value::SymbolTable::user_source_files`]) as the CALLING thread's ambient set,
/// returning a guard that restores the prior set on drop. A thread locus
/// (`:wat::kernel::spawn-thread`, `src/kernel/spawn.rs::spawn_thread_peer`) calls this —
/// alongside its other per-thread installs (`install_program_env`) — because it shares
/// its parent's already-frozen world on a NEW OS thread with no load pipeline of its own
/// to populate `USER_SOURCE_FILES` the normal way; without this install, D4's derivation
/// is silently a no-op on that thread (every `is_user_source_file` lookup sees an empty
/// set, so every error there locates exactly as it did before D4 — see the module doc).
pub(crate) fn install_user_source_files(
    files: std::sync::Arc<std::collections::HashSet<String>>,
) -> UserSourceGuard {
    USER_SOURCE_FILES.with(|s| {
        let prior = s.replace(files);
        UserSourceGuard { prior }
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
// frames nobody asked to see 100,000 of. **This is why `capped_frames_for_trace` does NOT
// call `snapshot_call_stack()`** — an earlier draft of this fn did, measured, and was
// wrong: it paid the full linear cost above and then threw most of it away, capping
// the OUTPUT while leaving the COST exactly as depth-dependent as the thing being
// capped. Reading `CALL_STACK` directly and cloning only the innermost N + outermost
// M elements bounds the cost itself to O(cap), independent of depth — MEASURED (same
// method, `measure_capped_snapshot_cost_by_depth`):
//
//   depth        capped_frames_for_trace() cost                            run 1      run 2
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

/// Snapshot the wat call stack for a new `RuntimeError`, reconstructed and capped to
/// innermost [`FRAME_CAP_INNERMOST`] + outermost [`FRAME_CAP_OUTERMOST`] frames. Returns
/// the (possibly capped) display frames, innermost first, plus the count elided from the
/// middle (0 when the stack fit under the cap). `raise_span` is the raw raise site
/// (`RuntimeError`'s own `span` argument) — see [`reconstruct_frames`]'s doc for what it
/// does to the innermost frame.
///
/// ⚠ Deliberately reads `CALL_STACK` directly rather than calling
/// [`snapshot_call_stack`] — that fn clones the WHOLE stack (measured linear in depth,
/// see the doc comment on [`FRAME_CAP_INNERMOST`]), which would pay the full
/// depth-dependent cost this cap exists to avoid before throwing the middle away.
/// Cloning only the `FRAME_CAP_INNERMOST + FRAME_CAP_OUTERMOST` elements this fn
/// actually keeps bounds the COST to the cap, not just the rendered output.
///
/// Over the cap, the innermost-N and outermost-M windows are reconstructed
/// SEPARATELY (not concatenated into one slice first): `reconstruct_frames`'s shift
/// logic reads a slot's TRUE next-inner neighbour via `stack.get(idx + 1)`, and the two
/// windows are NOT truly adjacent (a real, elided middle sits between them) — splicing
/// them into one contiguous slice would make the outer window's innermost slot
/// incorrectly read the inner window's outermost slot as if it were its real neighbour.
/// Reconstructing each window on its own means that boundary slot instead takes
/// `reconstruct_frames`' own "no next slot" fallback (its own `call_span`) — not a wrong
/// answer, the honest "nothing more local is available here" one, exactly as it would be
/// for a true outermost/innermost slot. `frames-elided` already states a gap exists;
/// this is never-before-exercised (F6: "`frames-elided` was never non-zero in any
/// golden"), so there is no existing golden shape to preserve here, only the invariant:
/// no frame ever attributes a location to a function that does not contain it.
/// `raise_span` is `None` for G2 (the raw raise site is ALREADY user source, so
/// `:location` IS the raise site — injecting it a second time as a synthesized frame
/// would duplicate it across `:location` ∪ `:frames`, which G1's invariant forbids;
/// see `tests/diagnostics/probe_excursus003_step4_g2_user_raised_untouched.rs`). Callers
/// pass `Some(span)` only for the non-G2 case, where the top slot's `at` needs the
/// raw raise site to locate it (G3/G4).
pub(crate) fn capped_frames_for_trace(raise_span: Option<Span>) -> (Vec<Frame>, usize) {
    CALL_STACK.with(|s| {
        let stack = s.borrow(); // storage order: oldest (outermost) first, newest (innermost) last
        let n = stack.len();
        let cap = FRAME_CAP_INNERMOST + FRAME_CAP_OUTERMOST;
        if n <= cap {
            return (reconstruct_frames(&stack, raise_span), 0);
        }
        // Innermost N: the last N elements of storage — the TRUE top of stack, so the
        // raise span (when there is one) belongs here.
        let mut frames = reconstruct_frames(&stack[n - FRAME_CAP_INNERMOST..], raise_span);
        // Outermost M: the first M elements of storage — no raise happens here; these
        // are the anchor frames ("where this ultimately started").
        frames.extend(reconstruct_frames(&stack[..FRAME_CAP_OUTERMOST], None));
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
                stack.push(FrameInfo::pristine(
                    format!(":user::fn-{i}"),
                    Span::new(std::sync::Arc::new("bench.wat".to_string()), i as i64, 0),
                ));
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
        let raise_span = Span::new(std::sync::Arc::new("bench.wat".to_string()), 0, 0);
        let mut samples = Vec::with_capacity(runs);
        for _ in 0..runs {
            let start = Instant::now();
            let capped = capped_frames_for_trace(Some(raise_span.clone()));
            let elapsed = start.elapsed();
            std::hint::black_box(&capped);
            samples.push(elapsed.as_nanos());
        }
        CALL_STACK.with(|s| s.borrow_mut().clear());
        samples.sort_unstable();
        samples[runs / 2]
    }

    /// Confirms the fix `capped_frames_for_trace`'s own doc comment claims: reading
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
