;; tests/types/probe_arc271_multi_param_generic_method.wat — co-located fixture
;;
;; Arc 271 — MULTI-type-param generic surface methods.

(:wat::core::defsurface :t::Combiner :nature :wat::core::Struct
  :features [(combine :- [A B] [self <- :t::Combiner  x <- :A  y <- :B] -> :A)])

(:wat::core::defrecord :t::C [])
(:wat::core::extend-type :t::C :t::Combiner (combine [self x y] x))

(:wat::core::defn :user::go [] -> :wat::core::i64
  (:t::Combiner/combine (:t::C) 5 "hi"))

