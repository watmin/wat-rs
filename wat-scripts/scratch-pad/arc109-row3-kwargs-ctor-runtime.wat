;; wat-scripts/scratch-pad/arc109-row3-kwargs-ctor-runtime.wat — arc 109 stone "a type reference
;; is not an expression", RUNTIME negative control for acceptance row 3.
;;
;; The type-check-only rungs live in arc109-type-reference-not-expression.wat. This file adds a
;; `:user::main` that actually EXECUTES the kwargs constructor — `(:user::R :field v)`, order-free,
;; arc 294 item 9a's construction ergonomics — to prove at runtime, not just at check time, that the
;; macro-dispatch guard (which declines `(Head :- [args])` on shape) still lets the companion macro
;; fire for the ordinary kwargs-call shape (no `:-` at index 1).

(:wat::core::defrecord :arc109row3::Pair [a <- :wat::core::i64 b <- :wat::core::i64])
(:wat::core::defstruct :arc109row3::SPair [a <- :wat::core::i64 b <- :wat::core::i64])
(:wat::holon::defrecord :arc109row3::HPair [a <- :wat::core::i64 b <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; order-free kwargs: b before a — only possible if the companion macro still fires.
    (:wat::kernel::println
      (:wat::string::interpolate "record  a={a} b={b}"
        :a (:wat::i64::to-string (:arc109row3::Pair/a (:arc109row3::Pair :b 2 :a 1)))
        :b (:wat::i64::to-string (:arc109row3::Pair/b (:arc109row3::Pair :b 2 :a 1)))))
    (:wat::kernel::println
      (:wat::string::interpolate "struct  a={a} b={b}"
        :a (:wat::i64::to-string (:arc109row3::SPair/a (:arc109row3::SPair :b 20 :a 10)))
        :b (:wat::i64::to-string (:arc109row3::SPair/b (:arc109row3::SPair :b 20 :a 10)))))
    (:wat::kernel::println
      (:wat::string::interpolate "holon   a={a} b={b}"
        :a (:wat::i64::to-string (:arc109row3::HPair/a (:arc109row3::HPair :b 200 :a 100)))
        :b (:wat::i64::to-string (:arc109row3::HPair/b (:arc109row3::HPair :b 200 :a 100)))))
    nil))
