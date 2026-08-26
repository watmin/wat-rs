;; wat-tests/core/core-foldl-spec.wat — stone 118.B6's DIFFERENTIAL: the native `:wat::core::foldl`
;; must agree with its wat specification `:wat::core::foldl-spec` on every input.
;;
;; ★ THE SHAPE, and it is the point of the stone. `foldl` is a Rust intrinsic. `foldl-spec` is the
;; same fold written in wat as obviously as possible — correct and slow on purpose. Builder,
;; 2026-08-18: *"we build wat-oracles that guide the rust code... the wat-native using rust provided
;; intrinsics must be faster than wat-oracle."* Same shape as `:wat::rete::insert-all-spec`, whose
;; own comment says it best: *"the native kernel is the fast impl, the spec keeps it honest."*
;;
;; ⚠ EVERY ROW USES AN ORDER-SENSITIVE `f`: `acc*2 + x`. This matters more than it looks.
;; A `+` differential is nearly vacuous — it passes while iterating BACKWARDS or re-associating.
;; ★ AND SO DOES SUBTRACTION, which is what this file used first: a LEFT fold with `acc - x` is just
;; `init - sum`, so it is order-BLIND, and the non-vacuity row below caught that on its first run.
;; `acc*2 + x` genuinely observes order — the same elements folded the other way give a different
;; answer — which is most of what a fold's contract IS.
;;
;; ⚠ AND `foldl-spec` MUST NEVER DELEGATE TO `foldl`. A spec that calls its subject proves nothing.
;; If someone "simplifies" it, these rows keep passing and stop meaning anything.
;; `[[feedback_a_green_test_can_prove_nothing]]`

;; ORDER-SENSITIVE by construction: doubling the accumulator before adding makes each element's
;; contribution depend on how many elements followed it.
(:wat::core::defn :wat-tests::core::core-foldl-spec::shift-add
  [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ (:wat::i64::* acc 2) x))

(:wat::core::defn :wat-tests::core::core-foldl-spec::id
  [x <- :wat::core::i64] -> :wat::core::i64 x)

;; ─── the four containers, at a length where order and associativity both bite ─────────────────

(:wat::test::deftest :wat-tests::core::core-foldl-spec::agree-on-vector
  (:wat::core::let [v (:wat::core::Vector :wat::core::i64 1 2 3 4 5)]
    (:wat::test::assert-eq
      (:wat::core::foldl      :wat-tests::core::core-foldl-spec::shift-add 0 v)
      (:wat::core::foldl-spec :wat-tests::core::core-foldl-spec::shift-add 0 v))))

(:wat::test::deftest :wat-tests::core::core-foldl-spec::agree-on-list
  (:wat::core::let [l (:wat::core::List 1 2 3 4 5)]
    (:wat::test::assert-eq
      (:wat::core::foldl      :wat-tests::core::core-foldl-spec::shift-add 0 l)
      (:wat::core::foldl-spec :wat-tests::core::core-foldl-spec::shift-add 0 l))))

(:wat::test::deftest :wat-tests::core::core-foldl-spec::agree-on-persistentvector
  (:wat::core::let [pv (:wat::core::PersistentVector 1 2 3 4 5)]
    (:wat::test::assert-eq
      (:wat::core::foldl      :wat-tests::core::core-foldl-spec::shift-add 0 pv)
      (:wat::core::foldl-spec :wat-tests::core::core-foldl-spec::shift-add 0 pv))))

;; ★ THE ROW B6 EXISTS FOR. Before this stone `foldl` REFUSED a Stream outright — `mappable()`'s
;; `Stream => false` arm carried arc 118's own "later strike. ○ gap" note. This row is that gap
;; closed, and it is checked against the spec rather than against a hand-written expectation.
(:wat::test::deftest :wat-tests::core::core-foldl-spec::agree-on-stream
  (:wat::core::let [st (:wat::core::map :wat-tests::core::core-foldl-spec::id
                         (:wat::core::Vector :wat::core::i64 1 2 3 4 5))]
    (:wat::test::assert-eq
      (:wat::core::foldl      :wat-tests::core::core-foldl-spec::shift-add 0 st)
      (:wat::core::foldl-spec :wat-tests::core::core-foldl-spec::shift-add 0 st))))

;; ─── the lengths where a fold's edges live: 0, 1, 2 ───────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-foldl-spec::agree-on-empty
  (:wat::core::let [v (:wat::core::Vector :wat::core::i64)]
    (:wat::test::assert-eq
      (:wat::core::foldl      :wat-tests::core::core-foldl-spec::shift-add 0 v)
      (:wat::core::foldl-spec :wat-tests::core::core-foldl-spec::shift-add 0 v))))

(:wat::test::deftest :wat-tests::core::core-foldl-spec::agree-on-one-and-two
  (:wat::core::do
    (:wat::core::let [v (:wat::core::Vector :wat::core::i64 7)]
      (:wat::test::assert-eq
        (:wat::core::foldl      :wat-tests::core::core-foldl-spec::shift-add 0 v)
        (:wat::core::foldl-spec :wat-tests::core::core-foldl-spec::shift-add 0 v)))
    (:wat::core::let [v (:wat::core::Vector :wat::core::i64 7 3)]
      (:wat::test::assert-eq
        (:wat::core::foldl      :wat-tests::core::core-foldl-spec::shift-add 0 v)
        (:wat::core::foldl-spec :wat-tests::core::core-foldl-spec::shift-add 0 v)))))

;; ─── ★ NON-VACUITY — the rows above must be capable of DISAGREEING ─────────────────────────────
;;
;; Every row so far asserts two expressions are equal. If both sides were computing something
;; trivially identical — or if `f` were associative and commutative — they would agree no matter
;; what either implementation did. This row pins the actual value AND proves the chosen `f` is
;; order-sensitive: left-fold gives 100-1-2-3-4-5 = 85, and ANY re-ordering or re-association gives
;; something else. Without it, "they agree" is a claim about nothing.
(:wat::test::deftest :wat-tests::core::core-foldl-spec::nonvacuity-order-is-observable
  (:wat::core::let [fwd (:wat::core::Vector :wat::core::i64 1 2 3 4 5)
                    rev (:wat::core::Vector :wat::core::i64 5 4 3 2 1)]
    (:wat::core::do
      (:wat::test::assert-eq
        (:wat::core::foldl :wat-tests::core::core-foldl-spec::shift-add 0 fwd) 57)
      ;; THE SAME MULTISET, opposite order, DIFFERENT answer. That inequality is what makes every
      ;; `agree-on-*` row above a real constraint on traversal order rather than an accident of the
      ;; operator.
      (:wat::test::assert-eq
        (:wat::core::foldl :wat-tests::core::core-foldl-spec::shift-add 0 rev) 129))))
