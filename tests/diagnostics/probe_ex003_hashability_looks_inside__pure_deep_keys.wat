;; Fixture for probe_ex003_hashability_looks_inside.rs (excursus 003, stone E) — the POSITIVE control.
;; Deep keys of pure data must still insert and be found in all three hashed containers, and the
;; `ExcludedByDesign` shapes that have REAL `Hash` arms — `List`, `PersistentMap` — must stay legal
;; keys: the deep check mirrors `impl Hash`, not `is_atomizable`.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [k     (:wat::core::Option.Some {:value (:wat::core::Tuple 1 "a")})
     l     (:wat::core::List 1 2)
     pm    (:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64] "a" 1)
     m     (:wat::hashmap::assoc (:wat::core::HashMap :- [(:wat::core::Option :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])]) :wat::core::i64]) k 1)
     s     (:wat::hashset::conj (:wat::core::HashSet :- [(:wat::core::Option :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])])]) k)
     sl    (:wat::hashset::conj (:wat::core::HashSet :- [(:wat::core::List :- [:wat::core::i64])]) l)
     spm   (:wat::hashset::conj (:wat::core::HashSet :- [(:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])]) pm)
     cache (:wat::core::Result/expect (:wat::cache::Lru/new :- [(:wat::core::Option :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])]) :wat::core::i64] 2) "cache")
     _put  (:wat::core::match (:wat::cache::Lru/put cache k 42)
             [:wat::core::Result.Ok {:value _d} (:wat::kernel::println "PUT OK")]
             [:wat::core::Result.Err {:error f} (:wat::kernel::println (:wat::cache::Fault/message f))])]
    (:wat::core::do
      (:wat::kernel::println (:wat::hashmap::contains-key? m k))
      (:wat::kernel::println (:wat::hashset::contains? s k))
      (:wat::kernel::println (:wat::hashset::contains? sl l))
      (:wat::kernel::println (:wat::hashset::contains? spm pm))
      (:wat::core::match (:wat::cache::Lru/get cache k)
        [:wat::core::Option.Some {:value v} (:wat::kernel::println v)]
        [:wat::core::Option.None {} (:wat::kernel::println "MISS")]))))
