;; tests/diagnostics/probe_arc296_raise_gate_bad.wat — wall proof for raise! re-gate.
;;
;; Arc 296 S3: :wat::kernel::raise! now requires :wat::core::Error.
;; This file attempts (raise! 42) — an i64 does NOT satisfy :wat::core::Error.
;; At HEAD (after re-gate): startup FAILS with a type-mismatch / assignability error
;; because 42 (:wat::core::i64) cannot satisfy the :wat::core::Error surface param.
;;
;; The probe_arc296_raise_gate.rs Rust test loads this file via startup_from_file
;; and asserts that startup returns Err (the wall holds).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::raise! 42))
