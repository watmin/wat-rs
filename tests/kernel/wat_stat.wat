;; Co-located fixture for wat_stat.rs — slurped via startup_beside(file!()).

(:wat::core::defn :my::compute-mean-known [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::f64 1.0 2.0 3.0 4.0 5.0)
     m  (:wat::stat::mean xs)
     v  (:wat::core::match m 
           ((:wat::core::Some x) x) (:wat::core::None -1.0))]
    (:wat::f64::to-string v)))

(:wat::core::defn :my::compute-mean-empty [] -> :wat::core::String
  (:wat::core::let
    [xs    (:wat::core::Vector :wat::core::f64)
     m     (:wat::stat::mean xs)
     label (:wat::core::match m 
              ((:wat::core::Some _) "some") (:wat::core::None "none"))]
    label))

(:wat::core::defn :my::compute-variance-known [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::f64 1.0 2.0 3.0 4.0 5.0)
     v  (:wat::core::match (:wat::stat::variance xs) 
           ((:wat::core::Some x) x) (:wat::core::None -1.0))]
    (:wat::f64::to-string v)))

(:wat::core::defn :my::compute-variance-single [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::f64 7.0)
     v  (:wat::core::match (:wat::stat::variance xs) 
           ((:wat::core::Some x) x) (:wat::core::None -1.0))]
    (:wat::f64::to-string v)))

(:wat::core::defn :my::compute-stddev-known [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::f64 1.0 2.0 3.0 4.0 5.0)
     sd (:wat::core::match (:wat::stat::stddev xs) 
           ((:wat::core::Some x) x) (:wat::core::None -1.0))]
    (:wat::core::if (:wat::core::> sd 1.41)  "ok" "bad")))

