;; probe-118B2-rider-verification.wat — rider's own verification of the 118.B2 collapse.
;; Exercises all six migrated verbs (interpose, keep, keep-indexed, map-indexed, dedupe,
;; distinct) plus reduce's Stream arms and stream->pvec, across concrete containers and via
;; composition, to confirm behavior is unchanged from the pre-collapse per-container twins.
;; Scratch, per CLAUDE.md convention — durable, loadable, type-checked by
;; every_wat_scripts_file_loads.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; interpose — sep between adjacent elements, no trailing sep, over Vector/List/PV/Stream.
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::interpose 0 (:wat::core::Vector :wat::core::i64 1 2 3)))))
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::interpose 0 (:wat::core::List 1 2 3)))))
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::interpose 0 (:wat::core::PersistentVector 1 2 3)))))
    ;; interpose over a single-element and an empty input (edge cases: no sep at all).
    (:wat::kernel::println
      (:wat::string::join "," (:wat::core::into [] (:wat::core::interpose 0 (:wat::core::Vector :wat::core::i64 1)))))
    (:wat::kernel::println
      (:wat::string::join "," (:wat::core::into [] (:wat::core::interpose 0 (:wat::core::Vector :wat::core::i64)))))

    ;; keep — over List/PersistentVector (Vector+Stream already covered by the shape-probe).
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::keep
          (:wat::core::fn [x <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
            (:wat::core::if (:wat::core::= 0 (:wat::core::mod x 2)) (:wat::core::Some x) :wat::core::None))
          (:wat::core::List 1 2 3 4 5 6)))))

    ;; keep-indexed — f : [i64 T :-> (Option :- [U])]; keep values at even indices.
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::keep-indexed
          (:wat::core::fn [i <- :wat::core::i64 x <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
            (:wat::core::if (:wat::core::= 0 (:wat::core::mod i 2)) (:wat::core::Some x) :wat::core::None))
          (:wat::core::Vector :wat::core::i64 10 11 12 13 14 15)))))

    ;; map-indexed — f : [i64 T :-> U]; pair index with value via a string.
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::map-indexed
          (:wat::core::fn [i <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ i x))
          (:wat::core::Vector :wat::core::i64 100 100 100 100)))))

    ;; dedupe — drop CONSECUTIVE duplicates only (1 2 1 3, not 1 2 3).
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::dedupe (:wat::core::Vector :wat::core::i64 1 1 2 2 1 1 3)))))

    ;; distinct — drop ALL duplicates, keep first occurrence (1 2 3, not 1 2 1 3).
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into [] (:wat::core::distinct (:wat::core::Vector :wat::core::i64 1 2 1 3 2 1)))))

    ;; reduce — 3-arity and 2-arity Stream arms (via a `map` stage so coll is (Stream :- [T])).
    (:wat::kernel::println
      (:wat::core::i64::to-string (:wat::core::reduce
        (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ acc x))
        0
        (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
          (:wat::core::Vector :wat::core::i64 1 2 3 4 5)))))
    (:wat::kernel::println
      (:wat::core::i64::to-string (:wat::core::reduce
        (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ acc x))
        (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
          (:wat::core::Vector :wat::core::i64 1 2 3 4 5)))))

    ;; stream->pvec — the drain, via a lazy stage into a PersistentVector, then Vector for print.
    (:wat::kernel::println
      (:wat::string::join ","
        (:wat::core::into []
          (:wat::core::into (:wat::core::PersistentVector)
            (:wat::core::keep (:wat::core::fn [x <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
                                 (:wat::core::Some x))
              (:wat::core::Vector :wat::core::i64 7 8 9))))))))
