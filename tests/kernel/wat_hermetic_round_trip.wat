;; Co-located fixture for wat_hermetic_round_trip.rs — slurped via startup_beside(file!()).

(:wat::core::defn :my::compute-stdout-count [] -> :wat::core::i64
  (:wat::core::let
    [result (:wat::test::run-hermetic (:wat::kernel::println "tada!"))
     lines  (:wat::kernel::RunResult/stdout result)]
    (:wat::core::length lines)))

(:wat::core::defn :my::compute-eval-in-outer [] -> :wat::core::Result<wat::holon::HolonAST,wat::core::EvalError>
  (:wat::core::let
    [hermetic-result (:wat::test::run-hermetic (:wat::kernel::println 42))
     lines           (:wat::kernel::RunResult/stdout hermetic-result)
     captured-src    (:wat::core::first lines)]
    (:wat::eval-edn! captured-src)))

