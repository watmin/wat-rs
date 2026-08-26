;; probe-brief-f64-surface-is-a-stub.wat — RUN proof for
;; docs/arc/2026/06/278-rules-engine/BRIEF-the-f64-surface-is-a-stub.md.
;;
;; A vacuous probe (no `:user::main`) fails before resolving anything and proves nothing — that
;; exact mistake was made on this arc three days ago. This file has a real `:user::main` and is
;; run via `target/release/wat <this file>`, printing one line per assertion so the transcript is
;; the proof, not an eyeballed "it compiled."
;;
;; Proves EXPECTATIONS rows, by actually executing:
;;   2 — a float rule is now expressible: `f64::>` / `f64::<=` compute real booleans.
;;   4 — NaN is total, not a hole: the NaN is built by COMPUTATION (`0.0 / 0.0` via the core f64
;;       op), never a literal — a literal may not parse and would prove nothing about the runtime
;;       comparator path (trap door 4).
;;   6 — the casing rename: `:wat::rete::core::string::=` / `not=` resolve and run (lowercase,
;;       per Part D; nested under `core::` per BRIEF-one-naming-rule-then-first-nth-to-string.md's
;;       naming-rule exception — core has no per-type `string::=`, so this row keeps its type
;;       qualifier rather than colliding with `bool::=`/`keyword::=` under one shared name).
;;   7 — the re-point: the four equality rows now name `:wat::core::{i64,f64}::{=,not=}` (Part E)
;;       and still return the correct boolean.
;;
;; Rows 3 (a type error at `--check`) and 5 (an UnknownFunction at runtime) are proven by two
;; SEPARATE files next to this one — neither can live inside a program that must otherwise run
;; clean to completion (a type error would fail to type-check at all; an UnknownFunction would
;; abort `main` before later assertions ran). See:
;;   probe-f64-domain-hole-stays-deleted.wat.bad  (row 3)
;;   probe-f64-comparator-bogus-head.wat           (row 5)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ── row 2 — a float rule is now expressible ──────────────────────────────────
    (:wat::kernel::println (:wat::rete::f64::> 0.9 0.8))    ;; expect true
    (:wat::kernel::println (:wat::rete::f64::<= 0.1 0.2))   ;; expect true
    (:wat::kernel::println (:wat::rete::f64::< 0.2 0.1))    ;; expect false
    (:wat::kernel::println (:wat::rete::f64::>= 0.1 0.1))   ;; expect true

    ;; ── row 4 — NaN is total, not a hole (NaN built by computation) ──────────────
    (:wat::core::let [nan (:wat::f64::/ 0.0 0.0)]
      (:wat::kernel::println (:wat::rete::f64::> nan 1.0))) ;; expect false, no raise

    ;; ── row 6 — the casing rename resolves and runs ───────────────────────────────
    (:wat::kernel::println (:wat::rete::string::= "abc" "abc"))     ;; expect true
    (:wat::kernel::println (:wat::rete::string::not= "abc" "xyz"))  ;; expect true

    ;; ── row 7 — the re-point: right door, right boolean ───────────────────────────
    (:wat::kernel::println (:wat::rete::i64::= 3 3))       ;; expect true
    (:wat::kernel::println (:wat::rete::i64::not= 3 4))    ;; expect true
    (:wat::kernel::println (:wat::rete::f64::= 1.5 1.5))   ;; expect true
    (:wat::kernel::println (:wat::rete::f64::not= 1.5 2.5)) ;; expect true
    nil))
