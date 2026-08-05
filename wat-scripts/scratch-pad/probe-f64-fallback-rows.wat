;; probe-f64-fallback-rows.wat — scorecard runner for BRIEF-f64-fallback-rows.md.
;;
;; Exercises the four new `:wat::rete::f64::{+,-,*,/}` Fallback rows and the
;; `dispatch_rete_op` arm's extended `Ok`-value check (NaN/±Inf take the fallback;
;; every ordinary finite float, including -0.0, passes through). Not a test file
;; itself (see `tests/rete` for the durable gate) — a loadable, type-checked
;; reference proving the scorecard's numbered claims by RUN.
;;
;; NOTE — overflow-to-Inf is built by repeated squaring (`o0`..`o5`), never a bare
;; huge-magnitude float LITERAL. `crates/wat-edn/src/writer.rs`'s `write_float`
;; formats any finite `f64` with `|f| >= 1e16` via plain `{}` Display, which never
;; switches to exponential notation — a `1e200`-shaped SOURCE literal round-trips
;; through `program_to_edn` as a bare ~200-digit run with no `.`/`e`, which the EDN
;; reader then reads as an out-of-range integer and rejects
;; (`probe_arc170_edn_bridge_unspellable::c03_the_whole_corpus_crosses_the_wire`
;; caught exactly this on this file). A pre-existing EDN-writer gap, orthogonal to
;; this brief's scope — not fixed here; worked around by keeping every literal
;; below the 1e16 boundary and reaching the overflow at RUNTIME instead, where
;; only the source AST (never an intermediate value) is ever re-serialized.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; row 2 — ordinary arithmetic untouched.
     r2 (:wat::rete::f64::+ 1.5 2.0 :undefined 0.0)
     ;; row 3 — NaN result takes the fallback.
     r3 (:wat::rete::f64::/ 0.0 0.0 :undefined -1.0)
     ;; row 4 — +Inf result takes the fallback.
     r4 (:wat::rete::f64::/ 1.0 0.0 :undefined -1.0)
     ;; Repeated squaring from a sub-1e16 literal: o0 < 1e16 (safe to spell), each
     ;; step doubles the exponent, o4 is still finite (~8.5e255), and o4*o4
     ;; overflows to +Inf — reached at runtime, never spelled as a literal.
     o0 9.9e15
     o1 (:wat::core::f64::* o0 o0)
     o2 (:wat::core::f64::* o1 o1)
     o3 (:wat::core::f64::* o2 o2)
     o4 (:wat::core::f64::* o3 o3)
     ;; row 5 — overflow-to-Inf takes the fallback.
     r5 (:wat::rete::f64::* o4 o4 :undefined -1.0)
     ;; row 6 — SAME expression as row 5, DIFFERENT fallback value.
     r6a (:wat::rete::f64::* o4 o4 :undefined -1.0)
     r6b (:wat::rete::f64::* o4 o4 :undefined 42.0)
     ;; row 7 — -0.0 is finite, passes through (not the fallback).
     r7 (:wat::rete::f64::* 0.0 -1.0 :undefined 99.0)
     ;; row 9 — i64 rows still fall back (Part A must not regress the Err path).
     r9 (:wat::rete::i64::/ 1 0 :undefined -1)
     ;; row 14 — core is untouched, still raw IEEE, still exit 0.
     r14 (:wat::core::f64::/ 0.0 0.0)]
    (:wat::kernel::println (:wat::core::string::concat "row2  ordinary (expect 3.5): "   (:wat::core::str r2)))
    (:wat::kernel::println (:wat::core::string::concat "row3  NaN->fallback (expect -1.0): " (:wat::core::str r3)))
    (:wat::kernel::println (:wat::core::string::concat "row4  +Inf->fallback (expect -1.0): " (:wat::core::str r4)))
    (:wat::kernel::println (:wat::core::string::concat "row5  overflow->fallback (expect -1.0): " (:wat::core::str r5)))
    (:wat::kernel::println (:wat::core::string::concat "row6a fallback=-1.0 (expect -1.0): " (:wat::core::str r6a)))
    (:wat::kernel::println (:wat::core::string::concat "row6b fallback=42.0 (expect 42.0): " (:wat::core::str r6b)))
    (:wat::kernel::println (:wat::core::string::concat "row7  -0.0 passthrough (expect -0.0, not 99.0): " (:wat::core::str r7)))
    (:wat::kernel::println (:wat::core::string::concat "row9  i64 fallback still fires (expect -1): " (:wat::core::str r9)))
    (:wat::kernel::println (:wat::core::string::concat "row14 core untouched (expect nan): " (:wat::core::str r14)))))
