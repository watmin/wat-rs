;; Fixture: test 16 — set-redef! true + same-type redef succeeds; :a == 2 at runtime.
(:wat::config::set-redef! true)
(:wat::core::def :t::a 1)
(:wat::core::def :t::a 2)
(:wat::core::defn :t::compute-a [] -> :wat::core::i64 :t::a)
