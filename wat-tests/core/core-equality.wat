;; wat-tests/core/core-equality.wat — arc 245.2b runtime coverage for
;; the :wat::core::= equality intrinsic.
;;
;; = is a RELATIONAL intrinsic (not a defclause): it unifies its two
;; arguments to the same type ∀T, then compares by value. Cross-type
;; (i64 vs f64) is a TYPE ERROR at check time — no promotion occurs
;; post-arc-237.8b. The inference lives in check.rs `infer_equality`.
;;
;; Preserves the equality-related passing tests from the retired
;; tests/wat_polymorphic_arithmetic.rs:
;;   - poly_eq_strings_still_works
;;   - typed_strict_i64_eq_homogeneous_works
;;   - typed_strict_i64_eq_rejects_f64_arg
;;
;; Grounded on:
;;   - deftest idiom:   wat-tests/core/seq-fold-aliases.wat
;;   - rejection idiom: run-hermetic (forked subprocess catches check errors)
;;   - = intrinsic:     check.rs `infer_equality` + eval-side `eval_eq`
;;   - nil value:       bare `nil` (arc 242 Doctrine 1: :wat::core::nil is TYPE only)
;;   - bool literals:   true / false (as used in wat-tests/test.wat)

;; ─── i64 equality ───────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-equality::eq-i64-equal
  
  (:wat::test::assert-eq (:wat::core::= 1 1) true))

(:wat::test::deftest :wat-tests::core::core-equality::eq-i64-not-equal
  
  (:wat::test::assert-eq (:wat::core::= 1 2) false))

;; ─── f64 equality ───────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-equality::eq-f64-equal
  
  (:wat::test::assert-eq (:wat::core::= 1.5 1.5) true))

(:wat::test::deftest :wat-tests::core::core-equality::eq-f64-not-equal
  
  (:wat::test::assert-eq (:wat::core::= 1.5 2.5) false))

;; ─── String equality ────────────────────────────────────────────────────
;;
;; Mirrors poly_eq_strings_still_works from the retired Rust file.
;; Same-type string equality works via the = intrinsic.

(:wat::test::deftest :wat-tests::core::core-equality::eq-string-equal
  
  (:wat::test::assert-eq (:wat::core::= "a" "a") true))

(:wat::test::deftest :wat-tests::core::core-equality::eq-string-not-equal
  
  (:wat::test::assert-eq (:wat::core::= "a" "b") false))

;; ─── i64 equality via typed helper ──────────────────────────────────────
;;
;; Mirrors typed_strict_i64_eq_homogeneous_works. A typed wrapper with
;; i64-param bindings enforces same-type equality at the call site.

(:wat::core::defn :wat-tests::core::core-equality::eq-i64
  [a <- :wat::core::i64
   b <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::= a b))

(:wat::test::deftest :wat-tests::core::core-equality::typed-i64-eq-homogeneous-works
  
  (:wat::test::assert-eq (:wat-tests::core::core-equality::eq-i64 3 3) true))

(:wat::core::defn :wat-tests::core::core-equality::eq-i64-b
  [a <- :wat::core::i64
   b <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::= a b))

(:wat::test::deftest :wat-tests::core::core-equality::typed-i64-eq-homogeneous-false
  
  (:wat::test::assert-eq (:wat-tests::core::core-equality::eq-i64-b 3 4) false))

;; ─── REJECTION: cross-type equality → check-time type error ─────────────
;;
;; Mirrors typed_strict_i64_eq_rejects_f64_arg. The = intrinsic unifies
;; arg types; passing i64 and f64 is a type mismatch at check time (the
;; intrinsic's relational constraint requires arg0 and arg1 to be the same
;; type — no promotion). run-hermetic-with-prelude registers the helper
;; in the child prelude; the call with mismatched types errors at freeze.


;; Arc 300 Stone C5 — mixed-numeric `=` is WELL-FORMED at check, and evaluates
;; category-aware: `(= 3 3.0)` checks clean and evaluates to FALSE (an i64 and an f64 are
;; never the same value), while `(= 1 1N)` is true. C5 reversed 237.8a's cross-numeric
;; check-reject because the checker was the OUTLIER — eval already computed these and clj
;; computes them, so a real program was rejected at check for something that would have run.
;;
;; This test previously asserted the 237.8a reject. C5's Rooms listed the Rust-side
;; siblings to flip (probe_arc237_8a / probe_arc237_8b) and they were flipped; THIS one was
;; invisible to that sweep because it sat under an arc-170 `ignore` marker. A suppressed
;; test is invisible to the migration that owns it — it kept accusing the substrate of a
;; defect the substrate does not have. Flipped now to the shipped contract.
(:wat::test::deftest :wat-tests::core::core-equality::typed-i64-eq-mixed-numeric-is-false

  (:wat::core::match (:wat::test::run-hermetic (:wat::test::assert-false (:wat::core::= 3 3.0)))
    (:wat::kernel::RunResult::Passed nil)
    ((:wat::kernel::RunResult::Failed _f)
      (:wat::kernel::assertion-failed!
        "expected mixed-numeric = to check clean and evaluate FALSE (arc 300 C5 + C4 category-aware =)"
        :wat::core::None :wat::core::None))))

;; ─── REJECTION: direct cross-type = → check-time type error ─────────────
;;
;; (= 1 1.5) is a type mismatch: = unifies arg0 and arg1 to the SAME type;
;; i64 and f64 are distinct types with no promotion path post-237.8b.
;; The poly_eq_mixed_promotes Rust test (which expected "equal") was in the
;; 13-failing list — the correct new behaviour is rejection.


(:wat::test::deftest :wat-tests::core::core-equality::cross-type-eq-rejected
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let [b (:wat::core::= 1 1.5)] b))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected check-time type error for (= 1 1.5)"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected check-time type error for (= 1 1.5)"
          :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut nil))))
