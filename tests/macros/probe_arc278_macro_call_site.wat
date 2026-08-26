;; Co-located fixture for probe_arc278_macro_call_site.rs — arc 278 §4 "macro-call-site".
;; (DESIGN-telemetry-caller-and-capacity.md §4)
;;
;; `:wat::kernel::macro-call-site` is the EXPAND-time twin of the runtime `:wat::kernel::call-site`:
;; used inside a MACRO BODY it returns the macro INVOCATION's OWN call-site as a spliceable
;; `:wat::kernel::Frame` construction form, baked from the engine's `call_site_span`. It fills the
;; per-log-line `emitted-from` the `log` widget macro needs — the runtime `call-site` captures the
;; ENCLOSING fn's caller (offset 0), NOT the `(log …)` line, so it cannot do this job.
;;
;; `:probe::here-frame`'s body splices `(:wat::kernel::macro-call-site)`, so each `(:probe::here-frame)`
;; invocation captures ITS OWN source line. The deftest' invokes it on TWO ADJACENT lines (`f1` then
;; `f2`, no line between) and asserts the captured lines differ by EXACTLY 1 — a self-consistent
;; invariant the code structure guarantees, with NO magic absolute line number (feedback: probe
;; numbers must match the code). This disproves the exact failure mode the primitive fixes: a constant
;; enclosing-fn frame would give BOTH invocations the SAME line → difference 0. (If a line is ever
;; inserted between `f1` and `f2`, the diff becomes 2 and the test correctly goes RED — self-guarding.)
;;
;; RED at HEAD: `:wat::kernel::macro-call-site` is not on `is_pure_total` → the default-deny purity
;;   gate refuses it in the macro body at expand → startup fails.
;; GREEN after: it folds to the Frame construction form → adjacent invocations differ by exactly 1.

(:wat::core::defmacro :probe::here-frame [] -> :wat::WatAST
  `~(:wat::kernel::macro-call-site))

(:wat::test::deftest :user::macro-call-site-captures-invocation-line 
  (:wat::core::let
    [f1  (:probe::here-frame)
     f2  (:probe::here-frame)
     ;; Arc 109 — Frame/line is a concrete (non-Option) i64, read directly.
     l1  (:wat::kernel::Frame/line f1)
     l2  (:wat::kernel::Frame/line f2)]
    (:wat::test::assert-true (:wat::core::= (:wat::i64::- l2 l1) 1))))
