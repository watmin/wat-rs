;; Co-located fixture for wat_engram_library.rs — slurped via startup_beside(file!()).
;; Each fn returns a String label rather than println-ing it.

(:wat::core::defn :my::compute-empty [] -> :wat::core::String
  (:wat::core::let
    [lib (:wat::holon::EngramLibrary/new 10000)
     n   (:wat::holon::EngramLibrary/len lib)]
    (:wat::core::if (:wat::core::= n 0) -> :wat::core::String "empty" "non-empty")))

(:wat::core::defn :my::compute-add-count [] -> :wat::core::String
  (:wat::core::let
    [lib     (:wat::holon::EngramLibrary/new 10000)
     sub     (:wat::holon::OnlineSubspace/new 10000 4)
     v       (:wat::holon::encode (:wat::holon::to-holon "x"))
     r       (:wat::holon::OnlineSubspace/update sub v)
     u       (:wat::holon::EngramLibrary/add lib "pattern-a" sub)
     n       (:wat::holon::EngramLibrary/len lib)
     found   (:wat::holon::EngramLibrary/contains lib "pattern-a")
     missing (:wat::holon::EngramLibrary/contains lib "absent")]
    (:wat::core::if
      (:wat::core::and (:wat::core::= n 1)
        (:wat::core::and found (:wat::core::not missing))) -> :wat::core::String
      "ok" "wrong")))

(:wat::core::defn :my::compute-match [] -> :wat::core::String
  (:wat::core::let
    [lib      (:wat::holon::EngramLibrary/new 10000)
     sub      (:wat::holon::OnlineSubspace/new 10000 4)
     v        (:wat::holon::encode (:wat::holon::to-holon "x"))
     r        (:wat::holon::OnlineSubspace/update sub v)
     u        (:wat::holon::EngramLibrary/add lib "alpha" sub)
     matches  (:wat::holon::EngramLibrary/match-vec lib v 5 5)
     nmatches (:wat::core::length matches)]
    (:wat::core::if (:wat::core::= nmatches 1) -> :wat::core::String "one-match" "wrong")))

