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

(:wat::test::ignore "arc-170 concurrency layer (subprocess spawn / thread-on-channel) — leaks/hangs; remove before arc 170 closes")
(:wat::test::deftest :wat-tests::core::core-equality::typed-i64-eq-rejects-f64-arg
  
  ;; prelude-free (arc 278 — prelude annihilated): the check-time type error rides INLINE
  ;; in the hermetic child's body. `=` unifies its args to the SAME type; i64 3 vs f64 3.0
  ;; is a mismatch with no promotion path → the child fails its own startup check → recv'
  ;; returns Lost → RunResult::Failed. No child-local helper needed.
  (:wat::core::match (:wat::test::run-hermetic' (:wat::core::= 3 3.0))
    ((:wat::kernel::RunResult::Failed _f) nil)
    (:wat::kernel::RunResult::Passed
      (:wat::kernel::assertion-failed!
        "expected check-time error: i64 3 = f64 3.0 (mismatched types under =)"
        :wat::core::None :wat::core::None))))

;; ─── REJECTION: direct cross-type = → check-time type error ─────────────
;;
;; (= 1 1.5) is a type mismatch: = unifies arg0 and arg1 to the SAME type;
;; i64 and f64 are distinct types with no promotion path post-237.8b.
;; The poly_eq_mixed_promotes Rust test (which expected "equal") was in the
;; 13-failing list — the correct new behaviour is rejection.

(:wat::test::ignore "arc-170 concurrency layer (subprocess spawn / thread-on-channel) — leaks/hangs; remove before arc 170 closes")
(:wat::test::deftest :wat-tests::core::core-equality::cross-type-eq-rejected
  
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let [b (:wat::core::= 1 1.5)] b))))]
    (:wat::core::match (:wat::kernel::recv' p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected check-time type error for (= 1 1.5)"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected check-time type error for (= 1 1.5)"
          :wat::core::None :wat::core::None)))))
