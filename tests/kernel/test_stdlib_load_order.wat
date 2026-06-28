;; Co-located fixture for test_stdlib_load_order.rs — slurped via startup_beside(file!()).

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::length (:wat::deporder::verify-stdlib)))

