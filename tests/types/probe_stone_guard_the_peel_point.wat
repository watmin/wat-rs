;; tests/types/probe_stone_guard_the_peel_point.wat — co-located fixture for
;; probe_stone_guard_the_peel_point.rs (STONE-guard-the-peel-point, arc 109).
;;
;; The three rows that must stay ACCEPTED once check.rs:5635 refuses too-many type args:
;;
;;  row 3 — the empty binder `:- []` against a ZERO-param callee (`:wat::eval-edn!`). `0 > 0`
;;          is false, so this is admitted BY CONSTRUCTION, not by a special case — the whole
;;          point of writing the guard as `>`.
;;  row 4 — a genuine generic call (`:wat::eval-ast!`, one declared param) supplied its exact
;;          declared count. Unchanged behaviour.
;;  row 5 — a genuine generic call (`:wat::kernel::eprintln`, TWO declared params `T,R`)
;;          supplied FEWER than declared (one of two). Must still type-check: the missing `R`
;;          completes by inference from the enclosing `-> :wat::core::i64` context, exactly the
;;          "partial application" case the design calls out as the reason `>` and not `!=`.
;;          `eprintln` never returns at runtime, so this defn is checked but never evaluated —
;;          this row is a check-time-only probe.

;; ─── Row 3 — empty binder against a zero-param callee; identical to no binder. ─────────────

(:wat::core::defn :t::row3_empty_binder [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::core::EvalError])
  (:wat::eval-edn! :- [] "42"))

;; ─── Row 4 — exact declared count (one param, one arg). ────────────────────────────────────

(:wat::core::defn :t::row4_exact_count [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::EvalError])
  (:wat::core::let
    [program (:wat::core::quote (:wat::i64::+ 40 2))]
    (:wat::eval-ast! :- [:wat::core::i64] program)))

;; ─── Row 5 — fewer than declared (one of `eprintln`'s two params, `T` bound, `R` inferred
;; from the enclosing return type). Check-time only — never called; `eprintln` terminates. ──

(:wat::core::defn :t::row5_fewer_than_declared [] -> :wat::core::i64
  (:wat::kernel::eprintln :- [:wat::core::String] "diagnostic"))
