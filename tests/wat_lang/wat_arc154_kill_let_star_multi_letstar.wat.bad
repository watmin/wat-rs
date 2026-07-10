;; Negative fixture: multiple let* sites fire BareLegacyLetStar.
;; Used by test: multiple_let_star_sites_post_retirement_silently_alias

(:wat::core::defn :t::a [] -> :wat::core::i64
  (:wat::core::let* (((x :wat::core::i64) 1)) x))
(:wat::core::defn :t::b [] -> :wat::core::i64
  (:wat::core::let* (((y :wat::core::i64) 2)) y))
