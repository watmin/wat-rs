;; Tier B step C battery — door: defined_values + redef_allowed.
;; User opts into redef and actually redefs. Config must be the first form.
(:wat::config::set-redef! true)
(:wat::core::def :battery::x 1)
(:wat::core::def :battery::x 2)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::i64::to-string :battery::x)))
