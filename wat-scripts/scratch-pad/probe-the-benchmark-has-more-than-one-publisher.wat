;; probe-the-benchmark-has-more-than-one-publisher.wat
;;
;; Seeds differ across publishers. Id shares tile 0..n.

(:wat::core::def :fanout::BACKOFF-SEED 1)

(:wat::core::defn :fanout::share-lo
  [i <- :wat::core::i64  n <- :wat::core::i64  p <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::* i n) p))

(:wat::core::defn :fanout::share-hi
  [i <- :wat::core::i64  n <- :wat::core::i64  p <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::* (:wat::i64::+ i 1) n) p))

(:wat::core::defn :fanout::publisher-seed [i <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ :fanout::BACKOFF-SEED (:wat::i64::* i 7919)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [n 2000
     p 3
     s0 (:fanout::publisher-seed 0)
     s1 (:fanout::publisher-seed 1)
     s2 (:fanout::publisher-seed 2)
     distinct-seeds (:wat::core::if (:wat::core::and (:wat::core::not (:wat::core::= s0 s1))
                                          (:wat::core::and (:wat::core::not (:wat::core::= s1 s2))
                                                           (:wat::core::not (:wat::core::= s0 s2))))
                       "yes" "no")
     a0 (:fanout::share-lo 0 n p)
     a1 (:fanout::share-hi 0 n p)
     b0 (:fanout::share-lo 1 n p)
     b1 (:fanout::share-hi 1 n p)
     c0 (:fanout::share-lo 2 n p)
     c1 (:fanout::share-hi 2 n p)
     p1lo (:fanout::share-lo 0 n 1)
     p1hi (:fanout::share-hi 0 n 1)
     tiled (:wat::core::if (:wat::core::and (:wat::core::= a0 0)
                                 (:wat::core::and (:wat::core::= a1 b0)
                                      (:wat::core::and (:wat::core::= b1 c0)
                                           (:wat::core::and (:wat::core::= c1 n)
                                                (:wat::core::and (:wat::core::= p1lo 0)
                                                     (:wat::core::= p1hi n))))))
              "yes" "no")]
    (:wat::core::format
      "seeds={s0},{s1},{s2};distinct={ds};p3=[{a0},{a1})[{b0},{b1})[{c0},{c1});p1=[{p1lo},{p1hi});tiled={t}"
      :s0 s0 :s1 s1 :s2 s2 :ds distinct-seeds
      :a0 a0 :a1 a1 :b0 b0 :b1 b1 :c0 c0 :c1 c1
      :p1lo p1lo :p1hi p1hi :t tiled)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
