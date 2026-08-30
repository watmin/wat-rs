;; wat-tests/core/core-reduce.wat — exercise :wat::core::reduce at runtime.
;;
;; Arc 118.2a — :wat::seq::reduce / :wat::seq::fold (both defalias forms over
;; :wat::core::foldl) are RETIRED; both promote to the single :wat::core::reduce
;; (proper clojure reduce: 2-arity + 3-arity, built over the unchanged foldl for
;; the eager containers and a dedicated walker for Stream). This file (renamed
;; from seq-fold-aliases.wat) is reduce's runtime exercise — a defclause is a
;; silent nil-stub if never called, so the delegation MUST be exercised by a
;; passing test or a broken clause can go undetected while the suite stays green.
;;
;; Grounded on:
;;   - fn + foldl syntax: wat/core.wat:101-106, wat/test.wat:147-151
;;   - Vector literal:    wat-tests/test.wat ((:wat::core::Vector :T …))
;;   - deftest shape:     wat-tests/core/option-expect.wat
;;   - reduce itself:     wat/seq.wat


;; ─── 3-arity reduce (explicit init) over a Vector: sum [1 2 3 4] = 10 ───────

(:wat::test::deftest :wat-tests::core::core-reduce::reduce-3-arity-sum-i64
  
  (:wat::core::let
    [xs (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4)
     result
      (:wat::core::reduce
        (:wat::core::fn [acc <- :wat::core::i64
                         n   <- :wat::core::i64] -> :wat::core::i64
          (:wat::i64::+ acc n))
        0
        xs)]
    (:wat::test::assert-eq result 10)))


;; ─── 2-arity reduce (no init — first element seeds the fold) over a Vector:
;; sum [1 2 3 4] = 10 ──────────────────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::core-reduce::reduce-2-arity-sum-i64
  
  (:wat::core::let
    [xs (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4)
     result
      (:wat::core::reduce
        (:wat::core::fn [acc <- :wat::core::i64
                         n   <- :wat::core::i64] -> :wat::core::i64
          (:wat::i64::+ acc n))
        xs)]
    (:wat::test::assert-eq result 10)))


;; ─── 3-arity reduce over a lazy Stream (map's output) — the new capability
;; :wat::seq::reduce never had: sum of [1 2 3 4] doubled = 20 ────────────────

(:wat::test::deftest :wat-tests::core::core-reduce::reduce-3-arity-over-stream
  
  (:wat::core::let
    [xs      (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4)
     doubled (:wat::core::map
               (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
                 (:wat::i64::* n 2))
               xs)
     result
      (:wat::core::reduce
        (:wat::core::fn [acc <- :wat::core::i64
                         n   <- :wat::core::i64] -> :wat::core::i64
          (:wat::i64::+ acc n))
        0
        doubled)]
    (:wat::test::assert-eq result 20)))
