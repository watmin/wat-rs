;; Co-located fixture for wat_reckoner.rs — slurped via startup_beside(file!()).

(:wat::core::defn :my::compute-discrete-dims-labels [] -> :wat::core::String
  (:wat::core::let
    [labels
      (:wat::core::Vector :- [:wat::holon::HolonAST]
        (:wat::holon::to-holon "up")
        (:wat::holon::to-holon "down"))
     r
      (:wat::holon::Reckoner/new-discrete "test-rec" 10000 100 labels)
     d       (:wat::holon::Reckoner/dims r)
     label-list (:wat::holon::Reckoner/labels r)
     nlabels (:wat::core::length label-list)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= d 10000) (:wat::core::= nlabels 2))
       "ok" "wrong")))

(:wat::core::defn :my::compute-observe-predict [] -> :wat::core::String
  (:wat::core::let
    [labels
      (:wat::core::Vector :- [:wat::holon::HolonAST]
        (:wat::holon::to-holon "up")
        (:wat::holon::to-holon "down"))
     r
      (:wat::holon::Reckoner/new-discrete "rec" 10000 1 labels)
     v          (:wat::holon::encode (:wat::holon::to-holon "x"))
     u1         (:wat::holon::Reckoner/observe r v 0 1.0)
     u2         (:wat::holon::Reckoner/observe r v 1 1.0)
     pred       (:wat::holon::Reckoner/predict r v)
     conviction (:wat::core::third pred)]
    (:wat::core::if (:wat::core::>= conviction 0.0)  "ok" "wrong")))

(:wat::core::defn :my::compute-continuous-construct [] -> :wat::core::String
  (:wat::core::let
    [r (:wat::holon::Reckoner/new-continuous "cont" 10000 100 0.0 16)
     d (:wat::holon::Reckoner/dims r)]
    (:wat::core::if (:wat::core::= d 10000)  "ok" "wrong")))

