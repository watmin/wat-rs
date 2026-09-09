;; probe-f64-comparator-bogus-head.wat — proof for
;; docs/arc/2026/06/278-rules-engine/BRIEF-the-f64-surface-is-a-stub.md EXPECTATIONS row 5
;; (non-vacuity).
;;
;; Trap door 3: `--check` does NOT validate `:wat::*` heads at all — a bogus rete keyword is
;; opaque to the checker exactly like any other unregistered `:wat::` symbol, so this file
;; TYPE-CHECKS (`target/release/wat --check` on it exits 0) despite `:wat::rete::f64::>X` never
;; having been minted. That is why it is safe to keep as an ORDINARY `.wat` under the loader
;; gate: `every_wat_scripts_file_loads` only parses + type-checks (`startup_from_source`), it
;; never runs `main`, so a body that raises at RUNTIME does not rot the gate.
;;
;; EXPECTED: running this file (not `--check`ing it) raises a located `UnknownFunction` at the
;; call site — proving the mint did not accidentally admit a typo'd head as a silent no-op or a
;; vacuous pass.
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
