;; probe-118B2-laziness-all-verbs.wat — confirms EVERY verb migrated by 118.B2 stays lazy over
;; an infinite source (terminates under `take`, rather than hanging). `keep` over an infinite
;; source is already covered by probe-118B2-one-clause-lazy-producer.wat; this covers the other
;; five: interpose, keep-indexed, map-indexed, dedupe, distinct. Scratch, per CLAUDE.md.

(:wat::core::defn :probe::nat
  [i <- :wat::core::i64] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::stream::lazy
    (:wat::stream::cons i (:probe::nat (:wat::core::+ i 1)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; interpose over an infinite source — must not try to find "the last element".
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::take (:wat::core::interpose 0 (:probe::nat 0)) 5))))
    ;; keep-indexed over an infinite source.
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::take
          (:wat::core::keep-indexed
            (:wat::core::fn [i <- :wat::core::i64 x <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
              (:wat::core::if (:wat::core::= 0 (:wat::core::mod i 2)) (:wat::core::Some x) :wat::core::None))
            (:probe::nat 0))
          3))))
    ;; map-indexed over an infinite source.
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::take
          (:wat::core::map-indexed
            (:wat::core::fn [i <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ i x))
            (:probe::nat 100))
          4))))
    ;; dedupe over an infinite source (already non-repeating, so it's just `nat` under `take`).
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::take (:wat::core::dedupe (:probe::nat 0)) 4))))
    ;; distinct over an infinite source.
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::take (:wat::core::distinct (:probe::nat 0)) 4))))))
