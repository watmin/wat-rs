;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E). Key: an enum variant field — an `:wat::enum::Impure` enum, direct (a `:Pure` one is refused by `ImpureVariantFieldInPureEnum`).
;; Verbs: Lru/put must answer an Err value, Lru/get must miss — never a panic.
(:wat::core::defenum :user::Boxed :wat::enum::Impure
  :Full  [inside <- (:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64])]
  :Empty [])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "handle")
     k (:user::Boxed.Full {:inside h})
     cache (:wat::core::Result/expect (:wat::cache::Lru/new :- [:user::Boxed :wat::core::i64] 2) "cache")
     _put  (:wat::core::match (:wat::cache::Lru/put cache k 1)
             [:wat::core::Result.Ok {:value _d} (:wat::kernel::println "UNREFUSED")]
             [:wat::core::Result.Err {:error f} (:wat::kernel::println (:wat::cache::Fault/message f))])
     ;; A pure sibling of the same type makes the cache NON-EMPTY, so get must hash k.
     ;; (An empty lru/hashbrown table answers get without hashing — that miss would prove nothing.)
     _pop  (:wat::core::match (:wat::cache::Lru/put cache (:user::Boxed.Empty {}) 7)
             [:wat::core::Result.Ok {:value _d} (:wat::kernel::println "SIBLING STORED")]
             [:wat::core::Result.Err {:error _f} (:wat::kernel::println "SIBLING REFUSED")])]
    (:wat::core::match (:wat::cache::Lru/get cache k)
      [:wat::core::Option.Some {:value _v} (:wat::kernel::println "HIT")]
      [:wat::core::Option.None {} (:wat::kernel::println "MISS")])))
