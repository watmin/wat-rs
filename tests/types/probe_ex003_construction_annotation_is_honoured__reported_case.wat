;; Fixture for probe_ex003_construction_annotation_is_honoured.rs — the-little-wat F-107's reported case, verbatim shape: the explicit
;; `:- [:p::E]` pins T before `seed`'s bare variant literal can narrow it to `:p::E.A`.
;; Before stone N this failed with "parameter #2 expects [:p::E.A :-> :p::E.A]" — the annotation
;; was stripped by the macro expander and never reached the checker. Now it CHECKS and runs.
(:wat::core::defenum :p::E :wat::enum::Pure :A [] :B [n <- :wat::core::i64])
(:wat::core::defstruct :p::Ops :- [T] [seed <- T  step <- [T :-> T]])
(:wat::core::defn :p::mk [] -> (:p::Ops :- [:p::E])
  (:p::Ops :- [:p::E] :seed (:p::E.A {}) :step (:wat::core::fn [x <- :p::E] -> :p::E x)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [ops (:p::mk)
                    _   (:wat::kernel::println (:wat::core::match ((:p::Ops/step ops) (:p::Ops/seed ops))
                                                 [:p::E.A {} "A"]
                                                 [:p::E.B {:n n} "B"]))]
    nil))
