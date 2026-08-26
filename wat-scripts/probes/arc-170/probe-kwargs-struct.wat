;; probe-kwargs-struct.wat — disconfirming probe for the kwargs-struct flip.
;;
;; A defn whose `& [...]` kwargs section holds an IMPURE field — a fn value. Today the
;; minted `:<name>::Kwargs` bundle is a defrecord (PURE), so the impure `f` field is rejected
;; by 293.W containment. After core.wat:754 flips defrecord→defstruct, the bundle is a struct
;; (impure, local) and accepts the fn.
;;
;; RED at HEAD: freezing :probe::apply-it errors — an impure fn field in the pure
;;   :probe::apply-it::Kwargs record.
;; GREEN after the flip: prints "42".

(:wat::core::defn :probe::apply-it
  [& [f <- :wat::core::Fn(wat::core::i64)->wat::core::i64
      n <- :wat::core::i64]]
  -> :wat::core::i64
  (:wat::core::apply  f n []))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::i64::to-string
      (:probe::apply-it :f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* x 2))
                        :n 21))))  ;; expect 42
