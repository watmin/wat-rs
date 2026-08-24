;; T11: recursive struct Tree (children is (Vector :- [my::Tree])) + root-value.
;; Type must appear exactly once in prologue after extraction.
(:wat::core::defstruct :my::Tree
  [value    <- :wat::core::i64
   children <- (:wat::core::Vector :- [:my::Tree])])
(:wat::core::defn :my::root-value [t <- :my::Tree] -> :wat::core::i64 (:my::Tree/value t))
