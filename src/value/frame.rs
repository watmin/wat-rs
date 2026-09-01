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
