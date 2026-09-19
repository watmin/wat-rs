;; Tier B step C battery — door: defined_values.
;; A user `def` of a shadowing-adjacent name (the old `:repl::turn` shape, under
;; a prefix the user CAN write).
(:wat::core::def :battery::turn "not-repl")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println :battery::turn))
