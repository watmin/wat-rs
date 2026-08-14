;; Arc 296 — the GREEN half of the nature-roots gate.
;;
;; Both Record umbrellas used to be registered `nature: Nature::Struct` — which said
;; "a record may hold impure values", the inverse of what a record IS. Two fields here,
;; one per umbrella, both of which a PURE aggregate must be able to hold.
;;
;; Before the fix, `holds-holon` RAISED `ImpureFieldInPureAggregate` while `holds-core`
;; passed — because `is_pure_type` carried a hand-written short-circuit for the core
;; umbrella and NOBODY GAVE IT TO THE SIBLING. That asymmetry is the whole finding: one
;; special case, two identical umbrellas.

(:wat::core::defrecord :t::holds-core  [r <- :wat::core::Record])
(:wat::core::defrecord :t::holds-holon [r <- :wat::holon::Record])
