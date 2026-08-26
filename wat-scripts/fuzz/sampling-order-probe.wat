;; wat-scripts/fuzz/sampling-order-probe.wat — DISCONFIRMING PROBE for the sampling
;; driver's traversal order (docs/GENERATIVE-TESTING.md), computed IN WAT against
;; the REAL library.
;;
;; WHY THIS FILE EXISTS. Both properties below were first "verified" in a throwaway
;; Python script that reimplemented mixed-radix digits. That checked a PYTHON MODEL
;; of the design, not `gen.wat` — had `gen-digit`/`gen-shift` carried a bug, the
;; Python would still have gone green. This computes the same two properties
;; THROUGH the library's own verbs, so the thing under test is the thing that ships.
;;
;; PROPERTY A — digit-reversal is a BIJECTION on 0..card. Without it, sampling
;; silently revisits points and misses others while reporting a clean count.
;; PROPERTY B — reversed order reaches the SLOWEST-varying dimensions first, the
;; mirror of sequential order. Decisive here: dim4 of the rete space is CHAIN
;; DEPTH, the dial that exposed the leading-filter class.

;; `:wat::gen::` is STDLIB as of 2026-08-25 — no load-file! needed.

(:wat::core::defn :user::bases [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector 3 3 3 3 4))

;; One fold, no vector reversal. Digit j sits at position (n-1-j) of the reversed
;; sequence, whose place value is the product of the bases AFTER j — that is,
;; card / (b0*..*bj). Carrying a running prefix product gives each digit its
;; reversed place directly.
(:wat::core::defstruct :user::Rev
  [rem <- :wat::core::i64  idx <- :wat::core::i64  pref <- :wat::core::i64])

(:wat::core::defn :user::rev [k <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [bs   (:user::bases)
                    card (:wat::gen::card-of bs)]
    (:user::Rev/idx
      (:wat::core::foldl
        (:wat::core::fn [a <- :user::Rev  b <- :wat::core::i64] -> :user::Rev
          (:wat::core::let [d  (:wat::gen::digit (:user::Rev/rem a) b)
                            pf (:wat::core::i64::* (:user::Rev/pref a) b)]
            (:user::Rev
              :rem  (:wat::gen::shift (:user::Rev/rem a) b)
              :idx  (:wat::core::i64::+ (:user::Rev/idx a)
                      (:wat::core::i64::* d (:wat::core::i64::/ card pf)))
              :pref pf)))
        (:user::Rev :rem k :idx 0 :pref 1)
        bs))))

;; PROPERTY A — every k in 0..card maps to a DISTINCT index.
(:wat::core::defn :user::distinct-images [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::foldl
      (:wat::core::fn [s <- (:wat::core::HashSet :- [:wat::core::i64])  k <- :wat::core::i64]
                      -> (:wat::core::HashSet :- [:wat::core::i64])
        (:wat::core::HashSet/conj s (:user::rev k)))
      (:wat::core::HashSet :wat::core::i64)
      (:wat::core::range 0 (:wat::gen::card-of (:user::bases))))))

;; PROPERTY B — distinct values of dimension `dim` seen in the first K.
(:wat::core::defn :user::cover
  [dim <- :wat::core::i64  k-count <- :wat::core::i64  reversed <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let [g (:wat::gen::coords (:user::bases))]
    (:wat::core::length
      (:wat::core::foldl
        (:wat::core::fn [s <- (:wat::core::HashSet :- [:wat::core::i64])  k <- :wat::core::i64]
                        -> (:wat::core::HashSet :- [:wat::core::i64])
          (:wat::core::HashSet/conj s
            (:wat::gen::nth ((:wat::gen::Gen/at g) (:wat::core::if (:wat::core::= reversed 1) (:user::rev k) k)) dim)))
        (:wat::core::HashSet :wat::core::i64)
        (:wat::core::range 0 k-count)))))

(:wat::core::defn :user::row [k <- :wat::core::i64  r <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:user::cover 0 k r) (:user::cover 1 k r) (:user::cover 2 k r)
    (:user::cover 3 k r) (:user::cover 4 k r)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [card (:wat::gen::card-of (:user::bases))
     _ (:wat::kernel::println (:wat::core::PersistentVector card))
     _ (:wat::kernel::println (:wat::core::PersistentVector (:user::distinct-images)))
     _ (:wat::kernel::println (:user::row 16 0))
     _ (:wat::kernel::println (:user::row 16 1))
     _ (:wat::kernel::println (:user::row 64 0))]
    (:wat::kernel::println (:user::row 64 1))))
