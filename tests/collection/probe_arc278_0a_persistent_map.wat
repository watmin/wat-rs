;; tests/collection/probe_arc278_0a_persistent_map.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()). Named zero-arg defns,
;; one per assertion in `persistent_map_core_behavior`.
;;
;; :wat::core::PersistentMap/{length,contains-key?,assoc,dissoc} carry no registered
;; TypeSchemes (runtime-dispatched intrinsics, arc-278-0a) — same class as
;; `metadata-of` (docs/CONVENTIONS.md gotcha); each defn below annotates the
;; documented/observed return shape, matching the original bare-world eval.

;; 1. ctor + length
(:wat::core::defn :t::p1-ctor-length [] -> :wat::core::i64
  (:wat::map::length (:wat::core::PersistentMap :a 1 :b 2)))

;; 2. contains-key? hit
(:wat::core::defn :t::p2-contains-hit [] -> :wat::core::bool
  (:wat::map::contains-key? (:wat::core::PersistentMap :a 1) :a))

;; 2. contains-key? miss
(:wat::core::defn :t::p3-contains-miss [] -> :wat::core::bool
  (:wat::map::contains-key? (:wat::core::PersistentMap :a 1) :z))

;; 3. IMMUTABILITY / structural sharing — assoc does not mutate the original.
(:wat::core::defn :t::p4-assoc-immutable-original [] -> :wat::core::i64
  (:wat::core::let [pm  (:wat::core::PersistentMap :a 1)
                     _pm2 (:wat::map::assoc pm :b 2)]
    (:wat::map::length pm)))

;; 3. assoc returns the extended map
(:wat::core::defn :t::p5-assoc-extended [] -> :wat::core::i64
  (:wat::map::length
    (:wat::map::assoc (:wat::core::PersistentMap :a 1) :b 2)))

;; 4. dissoc removes the key
(:wat::core::defn :t::p6-dissoc-removes [] -> :wat::core::bool
  (:wat::map::contains-key?
    (:wat::map::dissoc (:wat::core::PersistentMap :a 1) :a) :a))

;; 5. LAYER-1 polymorphism — generic contains? dispatches on PersistentMap.
(:wat::core::defn :t::p7-generic-contains [] -> :wat::core::bool
  (:wat::core::contains? (:wat::core::PersistentMap :a 1) :a))

;; 5. LAYER-1 polymorphism — generic assoc dispatches on PersistentMap.
(:wat::core::defn :t::p8-generic-assoc [] -> :wat::core::i64
  (:wat::map::length (:wat::core::assoc (:wat::core::PersistentMap :a 1) :b 2)))
