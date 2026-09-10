;; TARGET — Option is an ordinary parametric enum (296 H-3) and must take the same map ctor.
;; ⚠ THE SLOT IS TYPED ON PURPOSE. In an untyped slot this form ALREADY "works" — it builds
;; Option<HashMap<keyword,i64>>, the map swallowed as the PAYLOAD. Only a typed slot can tell
;; the target apart from that.
(:wat::core::defn :user::pick [o <- (:wat::core::Option :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::match o [:wat::core::Option.Some {:value v} v] [:wat::core::Option.None {} -1]))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::show (:user::pick (:wat::core::Option.Some {:value 1})))))
