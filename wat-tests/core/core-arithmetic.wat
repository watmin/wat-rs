;; wat-tests/core/core-arithmetic.wat — arc 245.2b runtime coverage for
;; :wat::core::{+,-,*,/,<,>,<=,>=} defclauses.
;;
;; Mirrors the 19 PASSING tests in tests/wat_polymorphic_arithmetic.rs so
;; retiring that Rust file loses no valid coverage. Cross-type promotion
;; tests (retired 237.8b behaviour) are NOT mirrored; the new behaviour
;; for those inputs is rejection (NoMatchingClause), covered below.
;;
;; Grounded on:
;;   - deftest idiom:    wat-tests/core/seq-fold-aliases.wat
;;   - rejection idiom:  run-hermetic body (forked subprocess catches check errors)
;;   - forms under test: wat/core.wat (defclauses + 2-ary typed leaves)
;;   - nil value:        bare `nil` (arc 242 Doctrine 1: :wat::core::nil is TYPE only)
;;   - 0-ary/1-ary/variadic arity rules: core.wat comments + arc 148 slice 4

;; ─── 2-ary add ──────────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::add-i64-i64
  
  (:wat::core::let [sum (:wat::core::+ 2 3)]
    (:wat::test::assert-eq sum 5)))

(:wat::test::deftest :wat-tests::core::core-arithmetic::add-f64-f64
  
  (:wat::core::let [sum (:wat::core::+ 2.5 3.0)]
    (:wat::test::assert-eq sum 5.5)))

;; ─── 2-ary subtract ─────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::sub-i64-i64
  
  (:wat::core::let [d (:wat::core::- 5 2)]
    (:wat::test::assert-eq d 3)))

(:wat::test::deftest :wat-tests::core::core-arithmetic::sub-f64-f64
  
  (:wat::core::let [d (:wat::core::- 5.5 2.5)]
    (:wat::test::assert-eq d 3.0)))

;; ─── 2-ary multiply ─────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::mul-i64-i64
  
  (:wat::core::let [p (:wat::core::* 4 3)]
    (:wat::test::assert-eq p 12)))

(:wat::test::deftest :wat-tests::core::core-arithmetic::mul-f64-f64
  
  (:wat::core::let [p (:wat::core::* 2.0 3.0)]
    (:wat::test::assert-eq p 6.0)))

;; ─── 2-ary divide ───────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::div-i64-i64-truncates
  
  ;; i64 division truncates toward zero; 10/3 = 3.
  (:wat::core::let [q (:wat::core::/ 10 3)]
    (:wat::test::assert-eq q 3)))

(:wat::test::deftest :wat-tests::core::core-arithmetic::div-i64-i64-seven-two
  
  ;; Mirrors poly_div_i64_i64_returns_i64 from the retired Rust file.
  (:wat::core::let [q (:wat::core::/ 7 2)]
    (:wat::test::assert-eq q 3)))

(:wat::test::deftest :wat-tests::core::core-arithmetic::div-f64-f64
  
  (:wat::core::let [q (:wat::core::/ 9.0 2.0)]
    (:wat::test::assert-eq q 4.5)))

;; ─── 0-ary identity: + and * ────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::add-zero-ary-returns-zero
  
  ;; (:wat::core::+) → 0:i64  (additive identity per Lisp/Clojure tradition)
  (:wat::core::let [zero (:wat::core::+)]
    (:wat::test::assert-eq zero 0)))

(:wat::test::deftest :wat-tests::core::core-arithmetic::mul-zero-ary-returns-one
  
  ;; (:wat::core::*) → 1:i64  (multiplicative identity)
  (:wat::core::let [one (:wat::core::*)]
    (:wat::test::assert-eq one 1)))

;; ─── 1-ary negate (subtract) ────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::sub-one-ary-negates-i64
  
  ;; (- x) = (- 0 x): negate
  (:wat::core::let [neg (:wat::core::- 5)]
    (:wat::test::assert-eq neg -5)))

(:wat::test::deftest :wat-tests::core::core-arithmetic::sub-one-ary-negates-f64
  
  (:wat::core::let [neg (:wat::core::- 5.5)]
    (:wat::test::assert-eq neg -5.5)))

;; ─── 1-ary reciprocal (divide) ──────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::div-one-ary-reciprocal-i64-truncates
  
  ;; (/ x) = (/ 1 x): reciprocal; 1/5 = 0 in i64 (truncation).
  (:wat::core::let [r (:wat::core::/ 5)]
    (:wat::test::assert-eq r 0)))

;; ─── 3+-ary variadic fold ───────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::add-variadic-i64-folds
  
  ;; (+ 1 2 3 4 5) = 15 via left fold
  (:wat::core::let [sum (:wat::core::+ 1 2 3 4 5)]
    (:wat::test::assert-eq sum 15)))

(:wat::test::deftest :wat-tests::core::core-arithmetic::add-variadic-f64-folds
  
  (:wat::core::let [sum (:wat::core::+ 1.0 2.0 3.0)]
    (:wat::test::assert-eq sum 6.0)))

;; ─── Typed leaves coexist with polymorphic surface ──────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::typed-leaves-coexist
  
  ;; Mirrors typed_strict_arithmetic_coexists: :wat::core::i64::+ (2-ary),
  ;; :wat::core::f64::+ (2-ary), and :wat::core::+ (polymorphic 2-ary) all
  ;; work alongside each other. Per arc 237.8b the typed leaves are strictly
  ;; 2-ary; variadic on the typed surface is ArityMismatch.
  (:wat::core::let
    [a (:wat::i64::+ 1 2)
     b (:wat::f64::+ 1.0 2.0)
     c (:wat::core::+ 1 2)]
    (:wat::core::do
      (:wat::test::assert-eq a 3)
      (:wat::test::assert-eq b 3.0)
      (:wat::test::assert-eq c 3))))

;; ─── Strict typed helper: f64 ordering works with homogeneous args ───────
;;
;; Mirrors typed_strict_f64_lt_homogeneous_works. The prelude registers a
;; helper that wraps < with explicit f64 params; strict type enforcement
;; lives at the binding site (arc 148 slice 5).

(:wat::core::defn :wat-tests::core::core-arithmetic::lt-f64
  [a <- :wat::core::f64
   b <- :wat::core::f64]
  -> :wat::core::bool
  (:wat::core::< a b))

(:wat::test::deftest :wat-tests::core::core-arithmetic::typed-f64-lt-homogeneous-works
  
  (:wat::test::assert-eq (:wat-tests::core::core-arithmetic::lt-f64 1.5 2.5) true))

;; ─── Ordering: 2-ary i64 ────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::lt-i64-true
  
  (:wat::test::assert-eq (:wat::core::< 1 2) true))

(:wat::test::deftest :wat-tests::core::core-arithmetic::lt-i64-false
  
  (:wat::test::assert-eq (:wat::core::< 2 1) false))

(:wat::test::deftest :wat-tests::core::core-arithmetic::gt-i64-true
  
  (:wat::test::assert-eq (:wat::core::> 2 1) true))

(:wat::test::deftest :wat-tests::core::core-arithmetic::gt-i64-false
  
  (:wat::test::assert-eq (:wat::core::> 1 2) false))

(:wat::test::deftest :wat-tests::core::core-arithmetic::lte-i64-equal
  
  (:wat::test::assert-eq (:wat::core::<= 1 1) true))

(:wat::test::deftest :wat-tests::core::core-arithmetic::lte-i64-less
  
  (:wat::test::assert-eq (:wat::core::<= 1 2) true))

(:wat::test::deftest :wat-tests::core::core-arithmetic::lte-i64-greater
  
  (:wat::test::assert-eq (:wat::core::<= 2 1) false))

(:wat::test::deftest :wat-tests::core::core-arithmetic::gte-i64-equal
  
  (:wat::test::assert-eq (:wat::core::>= 2 2) true))

(:wat::test::deftest :wat-tests::core::core-arithmetic::gte-i64-greater
  
  (:wat::test::assert-eq (:wat::core::>= 2 1) true))

(:wat::test::deftest :wat-tests::core::core-arithmetic::gte-i64-less
  
  (:wat::test::assert-eq (:wat::core::>= 1 2) false))

;; ─── Ordering: 2-ary f64 ────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-arithmetic::lt-f64-true
  
  (:wat::test::assert-eq (:wat::core::< 1.5 2.5) true))

(:wat::test::deftest :wat-tests::core::core-arithmetic::gt-f64-true
  
  (:wat::test::assert-eq (:wat::core::> 2.5 1.5) true))

(:wat::test::deftest :wat-tests::core::core-arithmetic::lte-f64-equal
  
  (:wat::test::assert-eq (:wat::core::<= 1.5 1.5) true))

(:wat::test::deftest :wat-tests::core::core-arithmetic::gte-f64-true
  
  (:wat::test::assert-eq (:wat::core::>= 2.5 1.5) true))

;; ─── Division by zero: i64 (runtime error) ──────────────────────────────
;;
;; Mirrors poly_div_i64_zero_errors. i64 division by zero is a RuntimeError.
;; The primed thread peer crashes on the panic; the parent's recv' faces Lost
;; (the detected failure) instead of a clean completion Message.
;;
;; NOTE: f64 / 0.0 returns infinity in this substrate (IEEE 754) and does NOT
;; error; poly_div_f64_zero_errors was in the 13-failing list in the retired
;; Rust file — that behaviour is intentionally NOT mirrored here.


(:wat::test::deftest :wat-tests::core::core-arithmetic::div-i64-zero-runtime-error
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
           ;; Body must return nil; do discards the i64 result, then returns nil.
           ;; Division panics before nil is reached, which is the whole point — the
           ;; crash reaches the parent's recv' as Lost BEFORE the completion send'.
           (:wat::core::do
             (:wat::core::do (:wat::core::/ 5 0) nil)
             (:wat::core::match (:wat::kernel::send self 0)
               (:wat::kernel::SendOutcome::Sent   nil)
               (:wat::kernel::SendOutcome::Closed nil)
               ;; arc 278 #73 — same body as Sent/Closed: this send-outcome wall just
               ;; needs to proceed regardless (never a `_`-swallow); the divide-by-zero
               ;; above already panicked before this line could even run.
               (:wat::kernel::SendOutcome::Stopped nil)
               ((:wat::kernel::SendOutcome::Lost _c) nil)))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected RuntimeError for i64 / 0"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected RuntimeError for i64 / 0"
          :wat::core::None :wat::core::None)))))

;; ─── REJECTION: cross-type arithmetic → NoMatchingClause ────────────────
;;
;; Cross-type arithmetic was retired in arc 237.8b. NoMatchingClause fires at
;; CHECK time (no clause matches i64×f64). These spawn a primed PROCESS peer
;; (:user::main over the forms wire) so the check error crashes the child and
;; surfaces as recv' Lost, without breaking the outer file's type-check.
;;
;; Mirrors the behaviour asserted (negatively) by the retired Rust tests
;; poly_add_i64_f64_promotes_to_f64, poly_add_f64_i64_promotes_to_f64, etc.


(:wat::test::deftest :wat-tests::core::core-arithmetic::cross-type-add-rejected
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let [x (:wat::core::+ 1 2.0)] x))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected NoMatchingClause for (+ 1 2.0)"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected NoMatchingClause for (+ 1 2.0)"
          :wat::core::None :wat::core::None)))))

;; ─── REJECTION: string arithmetic → NoMatchingClause ────────────────────
;;
;; Mirrors poly_add_string_rejected_at_check. String is not a numeric type;
;; no defclause clause matches (String, String) for arithmetic.


(:wat::test::deftest :wat-tests::core::core-arithmetic::string-add-rejected
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let [x (:wat::core::+ "a" "b")] x))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected NoMatchingClause for (+ \"a\" \"b\")"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected NoMatchingClause for (+ \"a\" \"b\")"
          :wat::core::None :wat::core::None)))))

;; ─── REJECTION: cross-type ordering → TypeMismatch (check-time) ─────────
;;
;; Stone 245.8 — `<`/`>`/`<=`/`>=` are now a relational intrinsic.
;; Cross-type (i64 vs f64) is rejected at CHECK TIME by unify failure
;; (TypeMismatch), not at runtime by clause absence.
;; The test was previously `:NoMatchingClause`; intent (cross-type rejected)
;; is unchanged; error kind changed to TypeMismatch.


(:wat::test::deftest :wat-tests::core::core-arithmetic::cross-type-lt-rejected
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let [b (:wat::core::< 1 2.5)] b))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected TypeMismatch (check-time) for (< 1 2.5)"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected TypeMismatch (check-time) for (< 1 2.5)"
          :wat::core::None :wat::core::None)))))

;; ─── REJECTION: 0-ary - and / → NoMatchingClause ────────────────────────
;;
;; Mirrors slice4_variadic_sub_zero_ary_errors and
;; slice4_variadic_div_zero_ary_errors. Neither - nor / has a 0-ary clause.


(:wat::test::deftest :wat-tests::core::core-arithmetic::sub-zero-ary-rejected
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let [x (:wat::core::-)] x))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected NoMatchingClause for 0-ary (-)"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected NoMatchingClause for 0-ary (-)"
          :wat::core::None :wat::core::None)))))


(:wat::test::deftest :wat-tests::core::core-arithmetic::div-zero-ary-rejected
  
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let [x (:wat::core::/)] x))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed!
          "expected NoMatchingClause for 0-ary (/)"
          :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost _cause) nil)
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed!
          "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open"
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed!
          "expected NoMatchingClause for 0-ary (/)"
          :wat::core::None :wat::core::None)))))

;; ─── REJECTION: strict typed helper rejects wrong-type arg ──────────────
;;
;; Mirrors typed_strict_f64_lt_rejects_i64_arg and
;; typed_strict_i64_eq_rejects_f64_arg. A typed wrapper enforces at the
;; binding site; passing a wrong-type arg is a check-time type error.
;; run-hermetic-with-prelude registers the helper in the child's prelude so
;; it is visible to user::main in the forked subprocess.


(:wat::test::deftest :wat-tests::core::core-arithmetic::typed-mixed-numeric-lt-is-true
  
  ;; Arc 300 Stone C5 — mixed-numeric ORDERING is WELL-FORMED at check and evaluates to the
  ;; numeric-value comparison: `(< 1 2.5)` => true. C5 reversed 237.8a's cross-numeric
  ;; check-reject because the checker was the OUTLIER — eval already computed these and clj
  ;; computes them, so a real program was rejected at check for something that would have run.
  ;;
  ;; This test previously asserted the 237.8a reject. C5's Rooms listed the Rust-side
  ;; siblings to flip (probe_arc237_8a / probe_arc237_8b) and they were flipped; THIS one was
  ;; invisible to that sweep because it sat under an arc-170 `ignore` marker. A suppressed
  ;; test is invisible to the migration that owns it. Flipped now to the shipped contract.
  (:wat::core::match (:wat::test::run-hermetic (:wat::test::assert-true (:wat::core::< 1 2.5)))
    (:wat::kernel::RunResult::Passed nil)
    ((:wat::kernel::RunResult::Failed _f)
      (:wat::kernel::assertion-failed!
        "expected mixed-numeric < to check clean and evaluate TRUE (arc 300 C5)"
        :wat::core::None :wat::core::None))))
