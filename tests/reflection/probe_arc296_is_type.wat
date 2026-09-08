;; Stone Q — `:wat::runtime::is-type?` membership, not structure.
;; stdout, one EDN bool per line, in this order:
;;   primitive (i64)            true
;;   builtin container (Vector) true
;;   user type (usr::Shape)     true
;;   nonexistent name           false   ← the row the stone exists for
(:wat::core::defrecord :usr::Shape [n <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::runtime::is-type? :wat::core::i64))
  (:wat::kernel::println (:wat::runtime::is-type? :wat::core::Vector))
  (:wat::kernel::println (:wat::runtime::is-type? :usr::Shape))
  (:wat::kernel::println (:wat::runtime::is-type? :usr::TotallyMadeUp)))
