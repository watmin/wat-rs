;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E). Key: a record field — LAUNDERED through a generic `T`: a record declared with the handle type directly is refused by the containment rule (`ImpureFieldInPureAggregate`), the generic one is not.
;; Verb: HashSet conj must refuse with a wat TypeMismatch — never a panic.
(:wat::core::defrecord :user::Holder :- [T] [inner <- :T])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "handle")
     k (:user::Holder :inner h)
     s  (:wat::core::HashSet :- [(:user::Holder :- [(:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64])])])
     s2 (:wat::hashset::conj s k)]
    (:wat::kernel::println "INSERTED")))
