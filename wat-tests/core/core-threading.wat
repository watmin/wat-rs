;; wat-tests/core/core-threading.wat — corpus witness for the threading
;; macros :wat::core::-> (thread-first) and :wat::core::->> (thread-last).
;;
;; Both macros live in wat/core.wat lines 172-202 and are pure macro-
;; expansion-time source-to-source rewrites: they desugar before type-check
;; so the checker and runtime never see -> / ->> directly.
;;
;; Thread-first ->:  inject accumulator as the FIRST arg of each step.
;;   (-> x (f a b) g)  =>  (g (f x a b))
;;   A list step (f a…) => (f acc a…); a bare keyword step f => (f acc).
;;
;; Thread-last ->>:  inject accumulator as the LAST arg of each step.
;;   (->> x (f a b) g)  =>  (g (f a b x))
;;   A list step (f a…) => (f a… acc); a bare keyword step f => (f acc).
;;
;; Grounded on:
;;   - macro semantics:  wat/core.wat lines 172-202
;;   - probe contracts:  tests/probe_arc249_threading.rs (5 threading mints)
;;   - deftest shape:    wat-tests/core/core-arithmetic.wat
;;   - arc / stone:      arc 249 stone 249.3b

;; ─── Thread-first: multi-arg list step ───────────────────────────────────
;;
;; (-> 10 (:wat::i64::- 3)) expands to (:wat::i64::- 10 3) = 7.
;; Hand-written equivalent: (:wat::i64::- 10 3).
;; Asserts: threaded result == hand-written result.

(:wat::test::deftest :wat-tests::core::core-threading::thread-first-list-step
  
  (:wat::core::let
    [threaded (:wat::core::-> 10 (:wat::i64::- 3))
     direct   (:wat::i64::- 10 3)]
    (:wat::test::assert-eq threaded direct)))

;; ─── Thread-first: two list steps ────────────────────────────────────────
;;
;; (-> 10 (i64::- 3) (i64::* 2))
;;   step 1: (i64::- 10 3) = 7
;;   step 2: (i64::* 7 2)  = 14
;; Hand-written: (:wat::i64::* (:wat::i64::- 10 3) 2) = 14.

(:wat::test::deftest :wat-tests::core::core-threading::thread-first-two-list-steps
  
  (:wat::core::let
    [threaded (:wat::core::-> 10
                (:wat::i64::- 3)
                (:wat::i64::* 2))
     direct   (:wat::i64::* (:wat::i64::- 10 3) 2)]
    (:wat::test::assert-eq threaded direct)))

;; ─── Thread-last: multi-arg list step ────────────────────────────────────
;;
;; (->> 5 (:wat::i64::- 3)) expands to (:wat::i64::- 3 5) = -2.
;; Contrast with thread-first (10 - 3 = 7 vs 3 - 5 = -2).
;; Hand-written equivalent: (:wat::i64::- 3 5).

(:wat::test::deftest :wat-tests::core::core-threading::thread-last-list-step
  
  (:wat::core::let
    [threaded (:wat::core::->> 5 (:wat::i64::- 3))
     direct   (:wat::i64::- 3 5)]
    (:wat::test::assert-eq threaded direct)))

;; ─── Thread-last: two list steps ─────────────────────────────────────────
;;
;; (->> 1 (i64::+ 2) (i64::* 4))
;;   step 1: (i64::+ 2 1) = 3
;;   step 2: (i64::* 4 3) = 12
;; Hand-written: (:wat::i64::* 4 (:wat::i64::+ 2 1)) = 12.

(:wat::test::deftest :wat-tests::core::core-threading::thread-last-two-list-steps
  
  (:wat::core::let
    [threaded (:wat::core::->> 1
                (:wat::i64::+ 2)
                (:wat::i64::* 4))
     direct   (:wat::i64::* 4 (:wat::i64::+ 2 1))]
    (:wat::test::assert-eq threaded direct)))

;; ─── Thread-first vs thread-last: asymmetry proof ────────────────────────
;;
;; Same seed and step, opposite injection: proves -> ≠ ->>.
;; (-> 5 (i64::- 3)) = (i64::- 5 3) = 2
;; (->> 5 (i64::- 3)) = (i64::- 3 5) = -2
;; assert 2 ≠ -2 (i.e. results differ).

(:wat::test::deftest :wat-tests::core::core-threading::thread-first-vs-last-asymmetry
  
  (:wat::core::let
    [tf  (:wat::core::-> 5 (:wat::i64::- 3))
     tl  (:wat::core::->> 5 (:wat::i64::- 3))]
    (:wat::test::assert-eq (:wat::core::= tf tl) false)))

;; ─── Thread-first: bare keyword step ─────────────────────────────────────
;;
;; A bare keyword step f (not a list) desugars to (f acc).
;; (-> 3 :wat-tests::core::core-threading::inc1)
;;   => (:wat-tests::core::core-threading::inc1 3) = 4.

(:wat::core::defn :wat-tests::core::core-threading::inc1
  [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ x 1))

(:wat::test::deftest :wat-tests::core::core-threading::thread-first-bare-step
  
  (:wat::core::let
    [result (:wat::core::-> 3 :wat-tests::core::core-threading::inc1)]
    (:wat::test::assert-eq result 4)))

;; ─── Thread-last: bare keyword step ──────────────────────────────────────
;;
;; Bare step behaves identically for ->>: (f acc).
;; (->> 7 :wat-tests::core::core-threading::double)
;;   => (:wat-tests::core::core-threading::double 7) = 14.

(:wat::core::defn :wat-tests::core::core-threading::double
  [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::* x 2))

(:wat::test::deftest :wat-tests::core::core-threading::thread-last-bare-step
  
  (:wat::core::let
    [result (:wat::core::->> 7 :wat-tests::core::core-threading::double)]
    (:wat::test::assert-eq result 14)))

;; ─── Realistic pipeline: WHY threading reads better ──────────────────────
;;
;; Compute the sum of squares of [1 2 3 4 5] using ->>:
;;   (->> [1 2 3 4 5]
;;        (map square)          ; [1 4 9 16 25]
;;        (foldl + 0))          ; 55
;;
;; Hand-written equivalent (nested calls, reading inside-out):
;;   (foldl add 0 (map square [1 2 3 4 5]))
;;
;; Threading wins here: reads left-to-right data-flow without mental
;; bracket-counting.  Both paths must equal 55.

(:wat::core::defn :wat-tests::core::core-threading::square
  [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::* n n))

(:wat::core::defn :wat-tests::core::core-threading::add
  [a <- :wat::core::i64
   b <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ a b))

(:wat::test::deftest :wat-tests::core::core-threading::pipeline-sum-of-squares
  
  ;; Arc 118.2a — `map` flipped LAZY (returns Stream); `foldl` stays eager/Vector-only, so the
  ;; fold step here becomes `:wat::core::reduce` (same 3-arg shape, Stream-aware) instead —
  ;; `map` stays lazy (consumed exactly once by the fold; no materializer needed).
  (:wat::core::let
    [xs      (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)
     threaded
       (:wat::core::->> xs
         (:wat::core::map :wat-tests::core::core-threading::square)
         (:wat::core::reduce :wat-tests::core::core-threading::add 0))
     direct
       (:wat::core::reduce
         :wat-tests::core::core-threading::add
         0
         (:wat::core::map :wat-tests::core::core-threading::square xs))]
    (:wat::core::do
      (:wat::test::assert-eq threaded 55)
      (:wat::test::assert-eq direct 55))))

;; ─── Zero-steps identity: -> with no steps returns acc unchanged ──────────
;;
;; (-> x) with no steps: foldl over empty returns the accumulator.
;; The steps rest-binder is empty; foldl over empty returns acc unchanged.
;; Witnesses the identity law for ->.

(:wat::test::deftest :wat-tests::core::core-threading::thread-first-zero-steps-identity
  
  (:wat::test::assert-eq (:wat::core::-> 42) 42))

;; ─── Zero-steps identity: ->> with no steps returns acc unchanged ────────
;;
;; (->> x) with no steps: foldl over empty returns the accumulator.
;; Symmetric identity law for ->>.

(:wat::test::deftest :wat-tests::core::core-threading::thread-last-zero-steps-identity
  
  (:wat::test::assert-eq (:wat::core::->> 42) 42))
