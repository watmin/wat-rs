;; 255.5 — known types in annotation / return position, clojure spelling.
;; Call-position control lives in the sibling .wat.bad.
(:wat::core::defn :user::id [x :- wat.type/AST] -> wat.type/AST
  x)
;; Stone 255.81 — i64 is one of the 24 WAT_TYPE_HARD_PRIMITIVES; its `wat.core/` dialect
;; denoted to the now-retired `:wat::core::i64` keyword (`also_accept_type`'s pre-255.67
;; equivalence), so only `wat.type/i64` resolves post-cutover. `wat.time/Instant` below is
;; NOT one of the 24 — its `wat.time/` dialect is untouched by this stone.
(:wat::core::defn :user::add1 [n :- wat.type/i64] -> wat.type/i64
  (:wat::core::+ n 1))
(:wat::core::defn :user::hold [t :- wat.time/Instant] -> wat.time/Instant
  t)
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println "ok"))
