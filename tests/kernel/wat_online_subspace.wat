;; Co-located fixture for wat_online_subspace.rs — slurped via startup_beside(file!()).

(:wat::core::defn :my::compute-construct [] -> :wat::core::String
  (:wat::core::let
    [s (:wat::holon::OnlineSubspace/new 10000 16)
     d (:wat::holon::OnlineSubspace/dim s)
     k (:wat::holon::OnlineSubspace/k s)
     n (:wat::holon::OnlineSubspace/n s)]
    (:wat::core::if
      (:wat::core::and (:wat::core::= d 10000)
        (:wat::core::and (:wat::core::= k 16) (:wat::core::= n 0))) -> :wat::core::String
      "ok" "wrong")))

(:wat::core::defn :my::compute-update [] -> :wat::core::String
  (:wat::core::let
    [s       (:wat::holon::OnlineSubspace/new 10000 4)
     v       (:wat::holon::encode (:wat::holon::to-holon "x"))
     residual (:wat::holon::OnlineSubspace/update s v)
     n       (:wat::holon::OnlineSubspace/n s)]
    (:wat::core::if (:wat::core::= n 1) -> :wat::core::String "incremented" "stuck")))

(:wat::core::defn :my::compute-eigenvalues [] -> :wat::core::String
  (:wat::core::let
    [s    (:wat::holon::OnlineSubspace/new 10000 8)
     eigs (:wat::holon::OnlineSubspace/eigenvalues s)
     len  (:wat::core::length eigs)]
    (:wat::core::if (:wat::core::= len 8) -> :wat::core::String "k-eigs" "wrong-len")))

