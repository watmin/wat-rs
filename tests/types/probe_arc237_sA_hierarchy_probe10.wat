;; Fixture probe 10: (subtype? :my::Nonexistent :wat::core::Record) → Err (unknown type name).
;; The error may fire at startup (type check) or at eval (runtime); either satisfies the contract.
(wat.core/defn user/probe10 [] :- wat.type/bool
  (wat.core/subtype? my/Nonexistent wat.type/Record))
