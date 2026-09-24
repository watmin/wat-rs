;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E). Key: a record field — LAUNDERED through a generic `T`: a record declared with the handle type directly is refused by the containment rule (`ImpureFieldInPureAggregate`), the generic one is not.
;; Verb: HashMap assoc must refuse with a wat TypeMismatch — never a panic.
(:wat::core::defrecord :user::Holder :- [T] [inner <- :T])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "handle")
     k (:user::Holder :inner h)
     m  (:wat::core::HashMap :- [(:user::Holder :- [(:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64])]) :wat::core::i64])
     m2 (:wat::hashmap::assoc m k 1)]
    (:wat::kernel::println "INSERTED")))
