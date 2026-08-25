;; Prove the PROCESS grant path (peer-pid Some → foldl over grantables) runs end-to-end
;; in a real process bracket, WITHOUT any user record (dodging the pre-existing freeze bug
;; where type_def_to_ast drops a user record's fields). grantables is EMPTY, so the foldl is
;; a no-op — but the peer-pid Some branch + the whole map-worker grant/revoke path EXECUTES.
(:wat::core::defn :probe::double [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* n 2))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [nums (:wat::core::Vector :wat::core::i64 1 2 3 4 5)
     ;; process/grants with an EMPTY grantable vector → process locus, peer-pid Some, no-op fold
     pr (:wat::bracket::map (:wat::spawn::process/grants (:wat::core::Vector :wat::capability::Grantable)) nums :probe::double)
     tr (:wat::bracket::map (:wat::spawn::thread) nums :probe::double)]
    (:wat::kernel::println
      (:wat::string::concat (:wat::edn::write pr)
        (:wat::string::concat " " (:wat::edn::write tr))))))
