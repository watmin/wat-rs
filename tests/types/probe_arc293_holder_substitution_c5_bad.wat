;; tests/types/probe_arc293_holder_substitution_c5_bad.wat — case 5: struct REJECTED where :wat::Record wanted

(:wat::core::defstruct :geo::SPt [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :u::wants-record [r <- :wat::Record] -> :wat::Record r)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::wants-record (:geo::SPt/new 1 2))
  nil)
