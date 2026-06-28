;; Co-located fixture for wat_run_sandboxed_ast.rs — slurped via startup_beside(file!()).

(:wat::config::set-capacity-mode! :error)

(:wat::core::defn :my::compute-prints-hello [] -> :wat::core::String
  (:wat::core::let
    [r     (:wat::test::run-hermetic (:wat::kernel::println "hello"))
     lines (:wat::kernel::RunResult/stdout r)
     line  (:wat::core::first lines)]
    line))

(:wat::core::defn :my::compute-assertion-failure [] -> :wat::core::i64
  (:wat::core::let
    [r    (:wat::test::run-thread (:wat::test::assert-eq 1 2))
     fail (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail -> :wat::core::i64
      ((:wat::core::Some _) 1)
      (:wat::core::None    0))))

