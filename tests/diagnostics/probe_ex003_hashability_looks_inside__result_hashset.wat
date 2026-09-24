;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E). Key: `(Result.Ok <handle>)` — direct; the checker admits it.
;; Verb: HashSet conj must refuse with a wat TypeMismatch — never a panic. One cell per extra
;; recursive arm of `value_is_hashable`, so mutation (a) reddens each arm, not only the six
;; containers the brief named.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "handle")
     k (:wat::core::Result.Ok {:value h})
     s  (:wat::core::HashSet :- [(:wat::core::Result :- [(:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64]) :wat::core::String])])
     s2 (:wat::hashset::conj s k)]
    (:wat::kernel::println "INSERTED")))
