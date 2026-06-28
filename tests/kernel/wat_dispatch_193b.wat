;; Co-located fixture for wat_dispatch_193b.rs — slurped via startup_beside(file!()).

(:wat::core::use! :rust::test::Counter)

(:wat::core::defn :my::compute-increment [] -> :wat::core::i64
  (:wat::core::let
    [c (:rust::test::Counter::new 10)
     _ (:rust::test::Counter::increment c)
     _ (:rust::test::Counter::increment c)
     _ (:rust::test::Counter::increment c)]
    (:rust::test::Counter::read c)))

(:wat::core::defn :my::compute-read [] -> :wat::core::i64
  (:wat::core::let
    [c (:rust::test::Counter::new 42)]
    (:rust::test::Counter::read c)))

