;; red-owner-signals-child.wat — the RED probe for DESIGN-STONE-process-signal-owner-to-child.md.
;;
;; ⛔ THIS FILE IS RED BY DESIGN, TODAY. It lives under docs/…/probes/ (NOT wat-scripts/) precisely
;;    because `every_wat_scripts_file_loads` walks `wat-scripts` only — a deliberately-failing probe
;;    parked there would break that gate.
;;
;; ⛔ CORRECTION 2026-08-30 — P2 HAS LANDED AND THIS FILE'S RED MOVED PHASE. Two claims in the
;;    prose below are now FALSE, kept only as the record of what was measured on 2026-08-03:
;;    (a) "the owner->child signal verb does not exist" — `:wat::kernel::signal` is minted
;;    (`src/check.rs:4133`, `src/check.rs:11103`); (b) "THE ARBITER IS THE RUNTIME, NOT `--check`"
;;    — the arbiter is now the CHECK phase. The header's LAST paragraph ("TURNS GREEN AT P2")
;;    predicted exactly this, and it is the live claim.
;;
;; rune:lint(red-by-design) — the refusal PROVES the peer-lifecycle OUTCOME WALL bites on the bare
;;    `_` discard door: a SignalOutcome dropped in statement/discard position is a CHECK-phase
;;    error. The binding at the foot is deliberately un-faced — it is what a caller writes BEFORE
;;    the wall exists. A reader can check the sentence by running the file: startup raises
;;    `#wat.check/MalformedForm` on `:wat::core::let`, "unhandled :wat::kernel::SignalOutcome in
;;    statement/discard position — … the peer-lifecycle OUTCOME WALL (Phase 3)". FACE the binding
;;    (match Delivered/Failed) and the file goes green, which is precisely why it must not be.
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
;; ⛔ A FORM THIS PROBE ORIGINALLY TAUGHT, AND MUST NOT — corrected 2026-08-03 at the builder's catch.
;;    The first draft split the work into `:user::compute` and then wrote main as:
;;
;;        (:wat::core::defn :user::main [] -> :wat::core::nil
;;          (:wat::core::let [_ (:user::compute)] nil))
;;
;;    Pure ceremony. `compute` is `[] -> nil` and `main` is `[] -> nil`, so calling it IS a complete
;;    main body — no `let`, no trailing `nil`, and in a probe no reason for two functions at all.
;;    Collapsed into one `main` here.
;;
;;    THE REASON IT MATTERS IS NOT TIDINESS: `(let [_ X] nil)` is spelled EXACTLY like the swallow the
;;    `let`-`_` discard door exists to catch — the door the send/recv walls spent an arc closing, and
;;    the one SignalOutcome is about to join. It is harmless here only because the discarded value is
;;    `nil`. As a TAUGHT form it is the defect: a reader learns the wrapper as boilerplate and applies
;;    it to a must-use outcome. "Do not educate bad forms." It had already propagated into a live
;;    rider's control files before it was caught.
;;
;; TURNS GREEN AT P2. When `:wat::kernel::signal` + `:wat::kernel::Signal` + `:wat::kernel::SignalOutcome`
;; are minted, this file must run to completion — and the `_sig` binding must then be FACED (matched
;; over the SignalOutcome variants), because SignalOutcome joins MUST_USE_TYPES and a dropped outcome
;; is a compile error in both discard doors. The un-faced `let` binding below is deliberate: it is
;; what a caller writes BEFORE the wall exists, and P2's must-use gate must reject it.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [proc (:wat::test::spawn-peer
            (:wat::spawn::process)
            (:wat::core::forms
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::kernel::println "child up"))))
     ;; ⚠ DELIBERATELY UN-FACED, AND THE BINDING MUST BE THE BARE `_` — see the header.
     ;;    `_sig` does NOT work here: the gate is an EXACT match on the one-character symbol
     ;;    (`check.rs:10926`, `ident.as_str() == "_"`), so `_sig` is an ordinary named binding
     ;;    and sails through. This line is the gate's evidence; it only bites as `_`.
     _ (:wat::kernel::signal proc :wat::kernel::Signal::User1)]
    nil))
