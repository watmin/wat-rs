;; Prove the PROCESS grant path (peer-pid Some → foldl over grantables) runs end-to-end
;; in a real process bracket, WITHOUT any user record (dodging the pre-existing freeze bug
;; where type_def_to_ast drops a user record's fields). grantables is EMPTY, so the foldl is
;; a no-op — but the peer-pid Some branch + the whole map-worker grant/revoke path EXECUTES.
;;
;; ⚙ arc 255 Stone 1c-0a-ii: the retired combinator `:wat::spawn::process/grants` (called here
;; with an explicit, already-EMPTY `(Vector :- [:wat::capability::Grantable])`) was repointed
;; (`wat-scripts/fixes/repoint-retired-heads-to-live-spellings.wat`) to plain `:wat::spawn::process`
;; — this drops the retired combinator's sole argument, and with it the second dead name it
;; carried, `:wat::capability::Grantable` (renamed to `:wat::capability::Capability` at stone A;
;; this reference was never registered as a call head, so it never showed up in the corpus
;; census). `wat/bracket.wat`'s `map-worker` confirms a plain pool already passes
;; `grant-handles = nil` with a no-op grant-fn, and the `peer-pid → Some pid` GRANT-BOOT branch
;; still fires for a process locus — so this file's claim is unchanged: the same peer-pid Some
;; branch and the same grant/revoke path execute.
(:wat::core::defn :probe::double [n <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* n 2))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [nums (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)
     ;; process/grants with an EMPTY grantable vector → process locus, peer-pid Some, no-op fold
     pr (:wat::bracket::map (:wat::spawn::process) nums :probe::double)
     tr (:wat::bracket::map (:wat::spawn::thread) nums :probe::double)]
    (:wat::kernel::println
      (:wat::string::concat (:wat::edn::write pr)
        (:wat::string::concat " " (:wat::edn::write tr))))))
