;; Co-located fixture for wat_dispatch_e3_result.rs — slurped via startup_beside(file!()).

(:wat::core::use! :rust::test::Fallible)

(:wat::core::defn :my::compute-ok-matched [] -> :wat::core::i64
  (:wat::core::match (:rust::test::Fallible::non_negative 42) 
    [:wat::core::Ok {:value v} v]
    [:wat::core::Err {:error _} -1]))

(:wat::core::defn :my::compute-err-matched [] -> :wat::core::i64
  (:wat::core::match (:rust::test::Fallible::non_negative -1) 
    [:wat::core::Ok {:value _} 0]
    [:wat::core::Err {:error _} 99]))

(:wat::core::defn :my::compute-user-ok [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Ok 7) 
    [:wat::core::Ok {:value v} v]
    [:wat::core::Err {:error _} -1]))

(:wat::core::defn :my::compute-user-err [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Err "x") 
    [:wat::core::Ok {:value _} 0]
    [:wat::core::Err {:error _} 11]))

