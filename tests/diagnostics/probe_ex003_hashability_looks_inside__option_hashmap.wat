;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E). Key: `(Option.Some <handle>)` — direct; the checker admits it.
;; Verb: HashMap assoc must refuse with a wat TypeMismatch — never a panic.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "handle")
     k (:wat::core::Option.Some {:value h})
     m  (:wat::core::HashMap :- [(:wat::core::Option :- [(:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64])]) :wat::core::i64])
     m2 (:wat::hashmap::assoc m k 1)]
    (:wat::kernel::println "INSERTED")))
