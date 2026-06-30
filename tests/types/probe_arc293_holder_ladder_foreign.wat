;; tests/types/probe_arc293_holder_ladder_foreign.wat — co-located fixture (arc 293 K1b)
;;
;; The foreign half of the holder ladder. A foreign type satisfies a surface via its `extend-type`
;; subtype edge (assignable arms at check.rs:14633/14641) — and that edge must ALSO honor a holder
;; bound: the foreign type's DERIVED holder (is_holon_or_vector -> HolonRecord, is_portable_type ->
;; Record, else Struct) must clear the surface's floor via `rank() >=`.
;;
;; This fixture is the NEGATIVE case (it must FAIL to type-check after K1b): `:wat::core::String` is
;; edn-repr (Record-capable, rank 0) but NOT a holon (rank +1), so it must NOT satisfy a
;; `:holder :wat::holon::Record` surface — even though extend-type supplies the method structurally.
;;
;; RED at HEAD: the edge is holder-EXEMPT (option (b)), so the String wrongly satisfies and the world
;; type-checks. GREEN after K1b: the derived holder (Record) < the floor (HolonRecord) -> rejected.

(:wat::core::defsurface :k1b::Vsa :holder :wat::holon::Record
  :features [(measure [self] -> :wat::core::f64)])

(:wat::core::extend-type :wat::core::String :k1b::Vsa
  (measure [self] -> :wat::core::f64 0.0))

(:wat::core::defn :k1b::use [x <- :k1b::Vsa] -> :wat::core::f64 (:k1b::Vsa/measure x))

(:wat::core::defn :k1b::demo [] -> :wat::core::f64
  (:k1b::use "hello"))     ; a String in a :holder :HolonRecord slot — MUST be rejected (String is not a holon)
