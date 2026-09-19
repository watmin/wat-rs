;; A work-fn that returns an O above DEFAULT-MAX-MESSAGE-BYTES. Process tier.
;; 1 runner, 1 item: no survivor. Report the collect-loop arm that raises.
(:wat::core::defn :user::double-n
  [s <- :wat::core::String  n <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::if (:wat::i64::<= n 0)
    s
    (:user::double-n (:wat::string::concat s s) (:wat::i64::- n 1))))

(:wat::core::defn :user::work
  [_x <- :wat::core::i64]
  -> :wat::core::String
  (:user::double-n "x" 20))

(:wat::core::defn :user::compute [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::bracket::map (:wat::spawn::process/runner-count 1)
    (:wat::core::Vector :- [:wat::core::i64] 0)
    :user::work))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))
