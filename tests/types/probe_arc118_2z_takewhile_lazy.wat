;; Arc 118.2-Z strike A — DISCONFIRMING PROBE fixture: `:wat::core::take-while` is LAZY.
;;
;; boom errors (div-by-zero) only when applied to 99. The source is the LAZY `map` stream
;; `(map boom [1 2 5 99])` — so an element's boom is applied only when that cell is realized.
;;
;; `take-while (< x 3)` over that stream realizes: boom(1)=1 (keep), boom(2)=2 (keep),
;; boom(5)=5 (pred false → STOP) — and NEVER realizes the 99 cell. So boom(99) is never called.
;;
;; RED at HEAD: `take-while` is undefined → resolution error.
;; GREEN after strike A (lazy): `into []` forces the take-while stream → [1 2]; boom(99) untouched.
;; (If take-while were EAGER, it would drain the whole map stream first → boom(99) → DivisionByZero.)

(:wat::core::defn :my::boom [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= x 99) (:wat::core::/ 1 0) x))

(:wat::core::defn :my::compute [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::into []
    (:wat::core::take-while
      (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::< x 3))
      (:wat::core::map :my::boom [1 2 5 99]))))
