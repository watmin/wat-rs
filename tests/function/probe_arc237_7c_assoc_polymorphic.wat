;; tests/function/probe_arc237_7c_assoc_polymorphic.wat
;; Arc 237 Stone 237.7c — polymorphic assoc (HashMap arm regression; Record arms in siblings).
;; Co-located fixture, slurped via startup_beside(file!()).
;; Negative cases are in sibling *.wat.bad files.
;; Record-arm cases (#[ignore]'d until Stone 237.7c ships) are in separate fixtures.

;; HashMap arm — regression contract (works today via alias; works post via intrinsic)
(:wat::core::defn :user::assoc-hashmap [] -> :wat::core::i64
  (:wat::core::length
    (:wat::hashmap::keys
      (:wat::core::assoc (:wat::core::HashMap :- [:wat::core::String :wat::core::i64]) "k" 1))))
