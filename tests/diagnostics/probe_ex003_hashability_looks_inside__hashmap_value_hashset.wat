;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E). Key: a HashMap whose VALUE is the handle, `{1 <handle>}` — direct; the checker admits it.
;; Verb: HashSet conj must refuse with a wat TypeMismatch — never a panic.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "handle")
     k (:wat::core::HashMap :- [:wat::core::i64 (:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64])] 1 h)
     s  (:wat::core::HashSet :- [(:wat::core::HashMap :- [:wat::core::i64 (:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64])])])
     s2 (:wat::hashset::conj s k)]
    (:wat::kernel::println "INSERTED")))
