;; wat-tests/deporder.wat — proof tests for wat/deporder.wat.
;;
;; Arc 275 Stone 275.1. The tool carries its own proof (complectens):
;; literal SourceFile fixtures, no I/O — the pure verify function is
;; the target.
;;
;; Four cases:
;;   1. defmacro ref is order-free — a before b is fine when b defines a defmacro.
;;   2. eval-dep wrong order is a violation — a refs b's defn, a loads before b.
;;   3. intrinsic ref ignored — ref to a symbol defined in no fixture → no violation.
;;   4. the surface runs — (:wat::deporder::verify-stdlib) returns a Vector.

;; ─── Case 1: defmacro ref is order-free ─────────────────────────────

(:wat::test::deftest :wat-tests::deporder::defmacro-ref-is-order-free
  
  ;; File "a" calls (:t::m), which is defined in file "b" as a defmacro.
  ;; a loads before b. This must NOT be a violation because defmacros
  ;; are registered in the pre-expansion pass (order-free).
  (:wat::core::let
    [a  (:wat::source::File :path "a" :source "(:t::caller (:t::m))")
     b  (:wat::source::File :path "b" :source "(:wat::core::defmacro :t::m [] 1)")
     files (:wat::core::Vector :- [:wat::source::File] a b)
     viols (:wat::deporder::verify files)]
    (:wat::test::assert-eq (:wat::core::length viols) 0)))

;; ─── Case 2: eval-dep wrong order is a violation ─────────────────────

(:wat::test::deftest :wat-tests::deporder::eval-dep-wrong-order-is-violation
  
  ;; File "a" calls (:t::f), which is defined in file "b" as a defn.
  ;; a loads before b (position 0 before position 1). This IS a violation.
  (:wat::core::let
    [a  (:wat::source::File :path "a" :source "(:t::caller (:t::f))")
     b  (:wat::source::File :path "b" :source "(:wat::core::defn :t::f [] 1)")
     files-bad  (:wat::core::Vector :- [:wat::source::File] a b)
     files-good (:wat::core::Vector :- [:wat::source::File] b a)
     viols-bad  (:wat::deporder::verify files-bad)
     viols-good (:wat::deporder::verify files-good)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::length viols-bad) 1)
      (:wat::test::assert-eq (:wat::core::length viols-good) 0))))

;; ─── Case 3: intrinsic ref ignored ───────────────────────────────────

(:wat::test::deftest :wat-tests::deporder::intrinsic-ref-ignored
  
  ;; A file referencing :wat::io::read-file (defined in no fixture)
  ;; must produce no violation (it resolves to an intrinsic / built-in).
  (:wat::core::let
    [f (:wat::source::File :path "f" :source "(:wat::io::read-file \"some-path\")")
     files (:wat::core::Vector :- [:wat::source::File] f)
     viols (:wat::deporder::verify files)]
    (:wat::test::assert-eq (:wat::core::length viols) 0)))

;; ─── Case 4: the surface runs ────────────────────────────────────────

;; REVISIT: `verify-stdlib` walks the ENTIRE stdlib dep-order, so it grows with the
;; stdlib and is the one deftest heavy enough to approach the default budget — it races
;; under full-suite parallel contention (passes solo ~1s, crosses the default only under
;; load). Explicit 30s headroom until the test is made load-insensitive (off wall-clock).
;; A long-standing problematic-but-good-intentioned test; arc 300 C1 (bigint, +85 stdlib
;; lines) consumed the last of its margin.
(:wat::test::time-limit "30s")
(:wat::test::deftest :wat-tests::deporder::verify-stdlib-runs
  
  ;; (:wat::deporder::verify-stdlib) must evaluate without error and return
  ;; a Vector (its length may be zero or more — the enforcement test is 275.2).
  (:wat::core::let
    [viols (:wat::deporder::verify-stdlib)]
    (:wat::test::assert-true (:wat::i64::>= (:wat::core::length viols) 0))))
