;; Same mixed input as probe_poisoned_item_mixed.wat, through `map` (no slot).
;; The poison is re-queued onto every runner until REPORT-GONE. Mutation: this
;; is what map-by-outcome's per-item recording prevents.
(:wat::core::defn :user::double-n
  [s <- :wat::core::String  n <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::if (:wat::i64::<= n 0)
    s
    (:user::double-n (:wat::string::concat s s) (:wat::i64::- n 1))))

(:wat::core::defn :user::work
  [x <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= x -1)
    (:user::double-n "x" 20)
    (:wat::i64::to-string (:wat::core::+ x 1))))

(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::bracket::map (:wat::spawn::process/runner-count 3)
    (:wat::core::Vector :- [:wat::core::i64] 10 20 -1 40)
    :user::work))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
