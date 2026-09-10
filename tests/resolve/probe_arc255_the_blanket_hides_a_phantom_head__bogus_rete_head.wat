;; probe-f64-comparator-bogus-head.wat — proof for
;; docs/arc/2026/06/278-rules-engine/BRIEF-the-f64-surface-is-a-stub.md EXPECTATIONS row 5
;; (non-vacuity).
;;
;; ⛔ HISTORICAL — the paragraph below described the world BEFORE arc 255 killed the `:wat::*`
;; blanket, and it is kept because it is the defect's own testimony. It read:
;;
;;   "Trap door 3: `--check` does NOT validate `:wat::*` heads at all — a bogus rete keyword is
;;    opaque to the checker exactly like any other unregistered `:wat::` symbol, so this file
;;    TYPE-CHECKS (exits 0) despite `:wat::rete::f64::>X` never having been minted. That is why
;;    it is safe to keep as an ORDINARY `.wat` under the loader gate."
;;
;; ★ EVERY CLAUSE OF THAT IS NOW FALSE, and this file is the proof. `--check` DOES validate
;; `:wat::*` heads: a reserved-prefix name is a call head iff the registry knows it
;; (`src/resolve/walk.rs`). `:wat::rete::f64::>X` was never minted, so `--check` on this file now
;; EXITS 1. And its containment premise — "safe under the loader gate because the gate only
;; type-checks" — expired with the blanket, which is why arc 255 stone ④ moved it out of
;; `wat-scripts/` to here BEFORE the deletion landed.
;;
;; EXPECTED NOW: `--check` refuses this file, naming `:wat::rete::f64::>X` as an unresolved call
;; head. The bogus head is caught at the EARLIEST pass rather than at runtime — arc 255's founding
;; promise, that the undefined-func class dies as a side effect, collecting its last scalp.
;;
;; MOVED 2026-09-09 (arc 255 Stone 4) from `wat-scripts/scratch-pad/probe-f64-comparator-bogus-
;; head.wat` to `tests/resolve/`, out from under the loader gate's scan root
;; (`tests/lint/wat_scripts_fixes_load.rs`). This file's own containment premise, above, IS the
;; `:wat::*` blanket — the moment the blanket dies this file becomes an ordinary `wat-scripts/`
;; file that fails to load, reading as an unrelated floor failure. Asserted (not merely exhibited)
;; by `tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head.rs`, which pins TODAY's
;; behaviour (`--check` exit 0, `run` raises `UnknownFunction`) as a RATCHET aimed at the blanket:
;; when the blanket's own stone lands, this row goes red at exactly the right moment, and that
;; stone owns updating it. `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::rete::f64::>X 1.0 0.5))
    nil))
