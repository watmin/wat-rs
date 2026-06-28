;; Co-located fixture for wat_dispatch_e4_shared.rs — slurped via startup_beside(file!()).

(:wat::core::use! :rust::test::Greeting)

(:wat::core::defn :my::compute-message [] -> :wat::core::String
  (:wat::core::let
    [g (:rust::test::Greeting::new "hello" 2026)]
    (:rust::test::Greeting::message g)))

(:wat::core::defn :my::compute-year [] -> :wat::core::i64
  (:wat::core::let
    [g (:rust::test::Greeting::new "any" 2026)]
    (:rust::test::Greeting::year g)))

(:wat::core::defn :my::compute-crossing [] -> :rust::test::Greeting
  (:rust::test::Greeting::new "crossed" 1999))

