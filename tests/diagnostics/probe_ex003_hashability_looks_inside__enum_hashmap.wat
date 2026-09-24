;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E). Key: an enum variant field — an `:wat::enum::Impure` enum, direct (a `:Pure` one is refused by `ImpureVariantFieldInPureEnum`).
;; Verb: HashMap assoc must refuse with a wat TypeMismatch — never a panic.
(:wat::core::defenum :user::Boxed :wat::enum::Impure
  :Full  [inside <- (:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64])]
  :Empty [])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "handle")
     k (:user::Boxed.Full {:inside h})
     m  (:wat::core::HashMap :- [:user::Boxed :wat::core::i64])
     m2 (:wat::hashmap::assoc m k 1)]
    (:wat::kernel::println "INSERTED")))
