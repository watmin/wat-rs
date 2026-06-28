;; Fixture: probe 5 — mixed declaration prelude now includes def.
;; All 7 declaration forms lift together: def, struct, enum, newtype, typealias, defn, defmacro.

(:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
  (:wat::kernel::spawn-process
    (:wat::core::forms
      (:wat::core::def :h::def-answer 99)
      (:wat::core::defstruct :h::MixPoint8
        [x <- :wat::core::i64
         y <- :wat::core::i64])
      (:wat::core::defenum :h::MixDir8
        :Up
        :Down)
      (:wat::core::newtype :h::MixAmount8 :wat::core::i64)
      (:wat::core::typealias :h::MixCount8 :wat::core::i64)
      (:wat::core::defn :h::mix-i64-fn8 [v <- :wat::core::i64] -> :h::MixCount8
        v)
      (:wat::core::defmacro :h::mix-id8 [z <- :wat::WatAST] -> :wat::WatAST `~z)
      (:wat::core::defn :user::main [] -> :wat::core::nil
        (:wat::core::let
          [_ans :h::def-answer
           _p   (:h::MixPoint8/new 1 2)
           _d   :h::MixDir8::Up
           _a   (:h::MixAmount8/new 10)
           _n   (:h::mix-i64-fn8 7)]
          nil)))))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
