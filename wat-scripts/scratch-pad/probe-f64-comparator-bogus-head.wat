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
;; rune:lint(rete-name-unminted) :wat::rete::f64::>X — this head is the NEGATIVE CONTROL; being unminted IS the experiment, and spelling it as a real row destroys the proof.
;; (`tests/lint/rete_names_in_wat_scripts_resolve.rs` would otherwise read this deliberate absence
;; as the rot it hunts.)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::rete::f64::>X 1.0 0.5))
    nil))
