;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E). Key: a record field — LAUNDERED through a generic `T`: a record declared with the handle type directly is refused by the containment rule (`ImpureFieldInPureAggregate`), the generic one is not.
;; Verbs: Lru/put must answer an Err value, Lru/get must miss — never a panic.
(:wat::core::defrecord :user::Holder :- [T] [inner <- :T])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::Result/expect (:wat::cache::Lru/new :- [:wat::core::keyword :wat::core::i64] 2) "handle")
     k (:user::Holder :inner h)
     ;; No handle-free value of this type exists, so this cache can never be non-empty: every
     ;; put is refused, and an empty lru/hashbrown table answers get without hashing the key.
     cache (:wat::core::Result/expect (:wat::cache::Lru/new :- [(:user::Holder :- [(:wat::cache::Lru :- [:wat::core::keyword :wat::core::i64])]) :wat::core::i64] 2) "cache")
     _put  (:wat::core::match (:wat::cache::Lru/put cache k 1)
             [:wat::core::Result.Ok {:value _d} (:wat::kernel::println "UNREFUSED")]
             [:wat::core::Result.Err {:error f} (:wat::kernel::println (:wat::cache::Fault/message f))])]
    (:wat::core::match (:wat::cache::Lru/get cache k)
      [:wat::core::Option.Some {:value _v} (:wat::kernel::println "HIT")]
      [:wat::core::Option.None {} (:wat::kernel::println "MISS")])))
