;; tests/collection/probe_verify_hashset_of_vector_gap.wat — co-located fixture.
;; Historical evidence: (HashSet :- [(Vector :- [T])]) construction (arc 216.5a-d).
;; The gap is closed; this probe confirms it cannot reopen.

(:wat::core::defn :user::verify [] -> :wat::core::i64
  (:wat::core::let
    [v1     (:wat::core::Vector :wat::core::i64 1 2)
     v2     (:wat::core::Vector :wat::core::i64 3 4)
     outer  (:wat::core::HashSet :wat::type::Infer v1 v2)]
    (:wat::hashset::length outer)))
