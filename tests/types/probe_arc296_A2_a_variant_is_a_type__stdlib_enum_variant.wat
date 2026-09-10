;; ⛔ THE ROW THAT CATCHES A SCOPE CUT. A STDLIB enum's variant — the population
;; excluded by `!is_reserved_prefix` last time, and the one the capability is FOR.
;; Every other fixture spells :usr::Box, so a stone that excludes :wat::* passes them all.
(:wat::core::defn :user::takes-some [s <- (:wat::core::Option.Some :- [:wat::core::i64])] -> :wat::core::i64 7)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [o (:wat::core::Option.Some {:value 42})]
    (:wat::kernel::println (:user::takes-some o))))
