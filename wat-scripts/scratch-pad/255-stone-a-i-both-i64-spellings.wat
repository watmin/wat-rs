;; wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat — arc 255 Stone A-i
;; acceptance row 3: BOTH spellings run.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-i-the-i64-home.md
;;
;; The 17 i64 ops now live under BOTH `:wat::i64::*` (new, registered via
;; `#[wat_intrinsic]` in `src/intrinsic/i64.rs`) and `:wat::core::i64::*` (old,
;; the pre-existing `runtime.rs` dispatch arm — UNCHANGED, nothing retired
;; this stone). 34 assertions: one per op, per spelling — each `assert-eq`
;; checks the spelling actually dispatches and returns the correct value
;; (a wrong/missing registration would raise `UnknownFunction` or a bad
;; result, not silently pass). Where no simpler literal exists for the
;; result type (bigint, rational), the two spellings are asserted directly
;; against each other in both directions, which proves both are callable
;; AND agree.
;;
;; Run:  ./target/release/wat --check ./wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat   # EXIT=0
;;       ./target/release/wat        ./wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat   # EXIT=0 (34/34 assertions pass)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ── + ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::+ 1 2) 3)
    (:wat::test::assert-eq (:wat::core::i64::+ 1 2) 3)
    ;; ── - ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::- 5 3) 2)
    (:wat::test::assert-eq (:wat::core::i64::- 5 3) 2)
    ;; ── * ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::* 3 4) 12)
    (:wat::test::assert-eq (:wat::core::i64::* 3 4) 12)
    ;; ── / ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::/ 6 2) 3)
    (:wat::test::assert-eq (:wat::core::i64::/ 6 2) 3)
    ;; ── mod (floored, sign of divisor) ────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::mod -7 3) 2)
    (:wat::test::assert-eq (:wat::core::i64::mod -7 3) 2)
    ;; ── quot (truncate toward zero) ───────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::quot -7 3) -2)
    (:wat::test::assert-eq (:wat::core::i64::quot -7 3) -2)
    ;; ── rem (sign of dividend) ─────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::rem -7 3) -1)
    (:wat::test::assert-eq (:wat::core::i64::rem -7 3) -1)

    ;; ── < ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::< 1 2) true)
    (:wat::test::assert-eq (:wat::core::i64::< 1 2) true)
    ;; ── <= ─────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::<= 2 2) true)
    (:wat::test::assert-eq (:wat::core::i64::<= 2 2) true)
    ;; ── > ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::> 3 2) true)
    (:wat::test::assert-eq (:wat::core::i64::> 3 2) true)
    ;; ── >= ─────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::>= 2 2) true)
    (:wat::test::assert-eq (:wat::core::i64::>= 2 2) true)
    ;; ── = ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::= 2 2) true)
    (:wat::test::assert-eq (:wat::core::i64::= 2 2) true)
    ;; ── not= ───────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::not= 2 3) true)
    (:wat::test::assert-eq (:wat::core::i64::not= 2 3) true)

    ;; ── to-bigint (no simpler bigint literal — cross-check both directions) ──
    (:wat::test::assert-eq (:wat::i64::to-bigint 5) (:wat::core::i64::to-bigint 5))
    (:wat::test::assert-eq (:wat::core::i64::to-bigint 5) (:wat::i64::to-bigint 5))
    ;; ── to-f64 ─────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::to-f64 5) 5.0)
    (:wat::test::assert-eq (:wat::core::i64::to-f64 5) 5.0)
    ;; ── to-rational (no simpler rational literal — cross-check both directions) ──
    (:wat::test::assert-eq (:wat::i64::to-rational 5) (:wat::core::i64::to-rational 5))
    (:wat::test::assert-eq (:wat::core::i64::to-rational 5) (:wat::i64::to-rational 5))
    ;; ── to-string ──────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::i64::to-string 42) "42")
    (:wat::test::assert-eq (:wat::core::i64::to-string 42) "42")))
