;; red-owner-signals-child.wat — the RED probe for DESIGN-STONE-process-signal-owner-to-child.md.
;;
;; ⛔ THIS FILE IS RED BY DESIGN, TODAY. It lives under docs/…/probes/ (NOT wat-scripts/) precisely
;;    because `every_wat_scripts_file_loads` walks `wat-scripts` only — a deliberately-failing probe
;;    parked there would break that gate.
;;
;; WHAT IT PROVES: the owner->child signal verb does not exist, and NOTHING ELSE is missing. The
;; spawn tooling, the Process handle, the child program, the forms quoting — all work. Execution
;; reaches the signal call and dies on exactly one head.
;;
;; MEASURED 2026-08-03 (own run, `./target/release/wat <this file>`):
;;
;;   [#wat.kernel.LociDiedError/RuntimeError
;;     ["#wat.runtime/UnknownFunction {:message \"unknown function: :wat::kernel::signal\"
;;       :location #wat.core/Span {:file \"…/red-signal.wat\" :line 11 :col 11 …}
;;       :causes [] :path \":wat::kernel::signal\"}"]]
;;   exit=1
;;
;; ★ THE ARBITER IS THE RUNTIME, NOT `--check`. Measured, with a positive control, the same session:
;;   `wat --check` returns **exit 0** on this file, and also on a control containing
;;   `:wat::kernel::this-verb-certainly-does-not-exist`. An unknown callee is not a check-phase
;;   failure — it defers to a runtime UnknownFunction. Pick the arbiter by the gap's phase, and
;;   positive-control the arbiter before believing either colour.
;;   (This is `reference_check_is_not_a_complete_red_arbiter` lived, not recalled.)
;;
;; TURNS GREEN AT P2. When `:wat::kernel::signal` + `:wat::kernel::Signal` + `:wat::kernel::SignalOutcome`
;; are minted, this file must run to completion — and the `_sig` binding must then be FACED (matched
;; over the SignalOutcome variants), because SignalOutcome joins MUST_USE_TYPES and a dropped outcome
;; is a compile error in both discard doors. The un-faced `let` binding below is deliberate: it is
;; what a caller writes BEFORE the wall exists, and P2's must-use gate must reject it.

(:wat::core::defn :user::compute [] -> :wat::core::nil
  (:wat::core::let
    [proc (:wat::test::spawn-peer
            (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::kernel::println "child up"))))
     _sig (:wat::kernel::signal proc :wat::kernel::Signal::User1)]
    nil))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [_ (:user::compute)] nil))
