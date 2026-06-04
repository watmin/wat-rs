;; wat-tests/core/list-fold-aliases.wat — exercise :wat::list::reduce
;; and :wat::list::fold at runtime.
;;
;; Both are defalias forms over :wat::core::foldl.  register_defalias
;; installs a silent nil-stub when the target is missing and defers the
;; error to call-time, so the delegation MUST be exercised by a passing
;; test or a broken target can go undetected while the suite stays green.
;;
;; Grounded on:
;;   - fn + foldl syntax: wat/core.wat:101-106, wat/test.wat:147-151
;;   - Vector literal:    wat-tests/test.wat ((:wat::core::Vector :T …))
;;   - deftest shape:     wat-tests/core/option-expect.wat


;; ─── reduce: sum [1 2 3 4] = 10 ──────────────────────────────────────

(:wat::test::deftest :wat-tests::core::list-fold-aliases::reduce-sum-i64
  ()
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::i64 1 2 3 4)
     result
      (:wat::list::reduce
        (:wat::core::fn [acc <- :wat::core::i64
                         n   <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::+ acc n))
        0
        xs)]
    (:wat::test::assert-eq result 10)))


;; ─── fold: sum [1 2 3 4] = 10 ────────────────────────────────────────

(:wat::test::deftest :wat-tests::core::list-fold-aliases::fold-sum-i64
  ()
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::i64 1 2 3 4)
     result
      (:wat::list::fold
        (:wat::core::fn [acc <- :wat::core::i64
                         n   <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::+ acc n))
        0
        xs)]
    (:wat::test::assert-eq result 10)))
