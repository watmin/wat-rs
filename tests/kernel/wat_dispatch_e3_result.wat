;; Co-located fixture for wat_dispatch_e3_result.rs — slurped via startup_beside(file!()).

(:wat::core::use! :rust::test::Fallible)

(:wat::core::defn :my::compute-ok-matched [] -> :wat::core::i64
  (:wat::core::match (:rust::test::Fallible::non_negative 42) 
    [:wat::core::Result::Ok {:value v} v]
    [:wat::core::Result::Err {:error _} -1]))

(:wat::core::defn :my::compute-err-matched [] -> :wat::core::i64
  (:wat::core::match (:rust::test::Fallible::non_negative -1) 
    [:wat::core::Result::Ok {:value _} 0]
    [:wat::core::Result::Err {:error _} 99]))

(:wat::core::defn :my::compute-user-ok [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Result::Ok {:value 7}) 
    [:wat::core::Result::Ok {:value v} v]
    [:wat::core::Result::Err {:error _} -1]))

(:wat::core::defn :my::compute-user-err [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Result::Err {:error "x"}) 
    [:wat::core::Result::Ok {:value _} 0]
    [:wat::core::Result::Err {:error _} 11]))

