;; tests/collection/probe_arc278_0b_persistent_vector.wat — co-located fixture for
;; the sibling probe (.rs), slurped via startup_beside(file!()). Named zero-arg
;; defns, one per assertion in `persistent_vector_core_behavior`.
;;
;; :wat::core::PersistentVector/{length,get,conj} carry no registered TypeSchemes
;; (runtime-dispatched intrinsics, arc-278-0b) — same class as `metadata-of`
;; (docs/CONVENTIONS.md gotcha); each defn below annotates the documented/observed
;; return shape, matching the original bare-world eval.

;; 1. ctor + length
(:wat::core::defn :t::p1-ctor-length [] -> :wat::core::i64
  (:wat::vector::length (:wat::core::PersistentVector 10 20 30)))

;; 2. get by index
(:wat::core::defn :t::p2-get-by-index [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::vector::get (:wat::core::PersistentVector 10 20 30) 1))

;; 3. IMMUTABILITY / structural sharing — conj does not mutate the original.
(:wat::core::defn :t::p3-conj-immutable-original [] -> :wat::core::i64
  (:wat::core::let [pv  (:wat::core::PersistentVector 1 2)
                     _pv2 (:wat::vector::conj pv 3)]
    (:wat::vector::length pv)))

;; 3. conj returns the extended vector
(:wat::core::defn :t::p4-conj-extended [] -> :wat::core::i64
  (:wat::vector::length
    (:wat::vector::conj (:wat::core::PersistentVector 1 2) 3)))

;; 4. LAYER-1 polymorphism — generic get dispatches on PersistentVector.
(:wat::core::defn :t::p5-generic-get [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::get (:wat::core::PersistentVector 10 20 30) 2))

;; 4. LAYER-1 polymorphism — generic conj dispatches on PersistentVector.
(:wat::core::defn :t::p6-generic-conj [] -> :wat::core::i64
  (:wat::vector::length (:wat::core::conj (:wat::core::PersistentVector 1) 2)))
