;; wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat — arc 255 Stone A-ii
;; acceptance row 3: BOTH spellings run.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-A-ii-the-f64-home.md
;;
;; The 19 f64 ops now live under BOTH `:wat::f64::*` (new, registered via
;; `#[wat_intrinsic]` in `src/intrinsic/f64.rs`) and `:wat::core::f64::*` (old,
;; the pre-existing `runtime.rs` dispatch arm — UNCHANGED, nothing retired
;; this stone). 38 assertions: one per op, per spelling — each `assert-eq`
;; checks the spelling actually dispatches and returns the correct value (a
;; wrong/missing registration would raise `UnknownFunction` or a bad result,
;; not silently pass).
;;
;; `max-of` / `min-of` are VARIADIC under the NEW spelling (bare args) but
;; take a single `(Vector :- [f64])` under the OLD spelling — see
;; `src/intrinsic/f64.rs`'s module header for why. Both are exercised here at
;; FOUR arguments (more than two), so the variadic row actually proves
;; something beyond the binary case.
;;
;; Run:  ./target/release/wat --check ./wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat   # EXIT=0
;;       ./target/release/wat        ./wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat   # EXIT=0 (38/38 assertions pass)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; ── + ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::+ 1.0 2.0) 3.0)
    (:wat::test::assert-eq (:wat::core::f64::+ 1.0 2.0) 3.0)
    ;; ── - ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::- 5.0 3.0) 2.0)
    (:wat::test::assert-eq (:wat::core::f64::- 5.0 3.0) 2.0)
    ;; ── * ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::* 3.0 4.0) 12.0)
    (:wat::test::assert-eq (:wat::core::f64::* 3.0 4.0) 12.0)
    ;; ── / ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::/ 6.0 2.0) 3.0)
    (:wat::test::assert-eq (:wat::core::f64::/ 6.0 2.0) 3.0)
    ;; ── max ────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::max 1.0 2.0) 2.0)
    (:wat::test::assert-eq (:wat::core::f64::max 1.0 2.0) 2.0)
    ;; ── min ────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::min 1.0 2.0) 1.0)
    (:wat::test::assert-eq (:wat::core::f64::min 1.0 2.0) 1.0)

    ;; ── < ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::< 1.0 2.0) true)
    (:wat::test::assert-eq (:wat::core::f64::< 1.0 2.0) true)
    ;; ── <= ─────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::<= 2.0 2.0) true)
    (:wat::test::assert-eq (:wat::core::f64::<= 2.0 2.0) true)
    ;; ── > ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::> 3.0 2.0) true)
    (:wat::test::assert-eq (:wat::core::f64::> 3.0 2.0) true)
    ;; ── >= ─────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::>= 2.0 2.0) true)
    (:wat::test::assert-eq (:wat::core::f64::>= 2.0 2.0) true)
    ;; ── = ──────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::= 2.0 2.0) true)
    (:wat::test::assert-eq (:wat::core::f64::= 2.0 2.0) true)
    ;; ── not= ───────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::not= 2.0 3.0) true)
    (:wat::test::assert-eq (:wat::core::f64::not= 2.0 3.0) true)

    ;; ── abs ────────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::abs -3.5) 3.5)
    (:wat::test::assert-eq (:wat::core::f64::abs -3.5) 3.5)
    ;; ── round ──────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::round 1.5 0) 2.0)
    (:wat::test::assert-eq (:wat::core::f64::round 1.5 0) 2.0)
    ;; ── to-i64 ─────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::to-i64 3.75) (:wat::core::Some 3))
    (:wat::test::assert-eq (:wat::core::f64::to-i64 3.75) (:wat::core::Some 3))
    ;; ── to-string ──────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::to-string 2.5) "2.5")
    (:wat::test::assert-eq (:wat::core::f64::to-string 2.5) "2.5")
    ;; ── clamp ──────────────────────────────────────────────────────────
    (:wat::test::assert-eq (:wat::f64::clamp 5.0 -1.0 1.0) 1.0)
    (:wat::test::assert-eq (:wat::core::f64::clamp 5.0 -1.0 1.0) 1.0)

    ;; ── max-of (variadic, NEW; single-Vector, OLD) — 4 args, > 2 ────────
    (:wat::test::assert-eq (:wat::f64::max-of 1.0 -5.0 4.2 3.0) (:wat::core::Some 4.2))
    (:wat::test::assert-eq
      (:wat::core::f64::max-of (:wat::core::Vector :wat::core::f64 1.0 -5.0 4.2 3.0))
      (:wat::core::Some 4.2))
    ;; ── min-of (variadic, NEW; single-Vector, OLD) — 4 args, > 2 ────────
    (:wat::test::assert-eq (:wat::f64::min-of 1.0 -5.0 4.2 3.0) (:wat::core::Some -5.0))
    (:wat::test::assert-eq
      (:wat::core::f64::min-of (:wat::core::Vector :wat::core::f64 1.0 -5.0 4.2 3.0))
      (:wat::core::Some -5.0))))
