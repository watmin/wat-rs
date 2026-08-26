;; BENCH — 118.B6. The native `:wat::core::foldl` against its wat specification
;; `:wat::core::foldl-spec`.
;;
;; ★ WHAT THIS IS FOR, and it is NOT "is wat slow". Builder, 2026-08-18: *"we should be striving to
;; build correct-but-slow wat-oracles that are references for wat-native to satisfy fast-and-correct
;; ... the wat-native using rust provided intrinsics MUST BE FASTER than wat-oracle."* This bench
;; measures that REQUIRED RELATIONSHIP. A spec slower than its native is the design working; a
;; native no faster than its spec is a native that has stopped earning its existence.
;;
;; Correctness is NOT measured here — `wat-tests/core/core-foldl-spec.wat` holds the differential,
;; with an order-sensitive `f`. This is only the ratio.
;;
;; Shape discipline (`[[feedback_a_benchmarks_shape_manufactures_its_result]]`): fixed n, BOTH block
;; orderings, non-vacuity proving both arms compute the SAME value. No recalibration inside the run.

;; ⚠ PLAIN ADDITION HERE, deliberately — unlike the differential, which uses an ORDER-SENSITIVE
;; `acc*2 + x`. That `f` is right for correctness and impossible for a bench: doubling an
;; accumulator 200,000 times overflows i64 long before the timer matters, and the substrate
;; (correctly) raises IntegerOverflow rather than wrapping silently. Order-sensitivity is proven in
;; `wat-tests/core/core-foldl-spec.wat`; this file measures only the RATIO, and its non-vacuity is
;; that both arms return the same sum.
(:wat::core::defn :bench::shift-add [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ acc x))

(:wat::core::defn :bench::native [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl :bench::shift-add 0 v))

(:wat::core::defn :bench::spec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl-spec :bench::shift-add 0 v))

(:wat::core::defn :bench::ns [t0 <- :wat::time::Instant t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n  200000
     v  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::range 0 n))
     a0 (:wat::time::now) ra (:bench::spec v)   a1 (:wat::time::now)
     b0 (:wat::time::now) rb (:bench::native v) b1 (:wat::time::now)
     c0 (:wat::time::now) rc (:bench::native v) c1 (:wat::time::now)
     d0 (:wat::time::now) rd (:bench::spec v)   d1 (:wat::time::now)]
    (:wat::kernel::println
      (:wat::string::interpolate
        "n={n} NONVACUITY ra={ra} rb={rb} rc={rc} rd={rd} | A: spec={ad}ms native={bd}ms | B: native={cd}ms spec={dd}ms"
        :n n :ra ra :rb rb :rc rc :rd rd
        :ad (:wat::i64::/ (:bench::ns a0 a1) 1000000)
        :bd (:wat::i64::/ (:bench::ns b0 b1) 1000000)
        :cd (:wat::i64::/ (:bench::ns c0 c1) 1000000)
        :dd (:wat::i64::/ (:bench::ns d0 d1) 1000000)))))
