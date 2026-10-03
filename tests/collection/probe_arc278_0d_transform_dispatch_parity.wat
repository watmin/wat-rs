;; tests/collection/probe_arc278_0d_transform_dispatch_parity.wat — co-located fixture,
;; slurped via startup_beside(file!()), asserting startup (type-check) succeeds.
;;
;; Contains the 8 transform-op defns over PersistentVector (foldl/reduce-over-reverse/map/filter/
;; reverse/take/drop/concat) and the 3 bare-typed-container defns — all must type-check clean
;; (arc 278 stone 0d).
;;
;; Arc 118.2a note: `map`/`filter`/`take`/`drop` flipped LAZY (return `(Stream :- [T])`, not the
;; container-preserving contract this probe originally proved parity for). The test's PURPOSE —
;; "does this op accept a PersistentVector INPUT at check time" — still holds and is still
;; asserted here; only the OUTER fold changed from `foldl` (Vector/List/PersistentVector-only,
;; would reject the new Stream output) to `:wat::core::reduce` (same 3-arg shape, Stream-aware).
;; `foldl`/`reverse`/`concat` are untouched by 118.2a and keep their original `foldl` wrapping
;; unchanged.
;;
;; Arc 118.B6b: `foldr` retired — it was `reverse`+`foldl` wearing a name borrowed from Haskell,
;; where the verb is distinct only because it is LAZY, a property strict wat cannot have.
;; `p-foldr` below is renamed `p-fold-reverse` and its body is now spelled
;; `(reduce f init (reverse coll))` — the replacement composition, still checked over a
;; PersistentVector at check time, same as every other slot in this file.

(wat.core/defn user/p-foldl [] :- wat.type/i64
  (wat.core/foldl
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    (wat.type/PersistentVector :- [wat.type/i64] 1 2 3)))

(wat.core/defn user/p-fold-reverse [] :- wat.type/i64
  (wat.core/reduce
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    (wat.core/reverse (wat.type/PersistentVector :- [wat.type/i64] 1 2 3))))

(wat.core/defn user/p-map [] :- wat.type/i64
  (wat.core/reduce
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    (wat.core/map
      (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 (wat.i64/* x 2))
      (wat.type/PersistentVector :- [wat.type/i64] 1 2 3))))

(wat.core/defn user/p-filter [] :- wat.type/i64
  (wat.core/reduce
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    (wat.core/filter
      (wat.core/fn [x :- wat.type/i64] :- wat.type/bool (wat.i64/> x 1))
      (wat.type/PersistentVector :- [wat.type/i64] 1 2 3))))

(wat.core/defn user/p-rev [] :- wat.type/i64
  (wat.core/foldl
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    (wat.core/reverse (wat.type/PersistentVector :- [wat.type/i64] 1 2 3))))

(wat.core/defn user/p-take [] :- wat.type/i64
  (wat.core/reduce
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    (wat.core/take (wat.type/PersistentVector :- [wat.type/i64] 1 2 3) 2)))

(wat.core/defn user/p-drop [] :- wat.type/i64
  (wat.core/reduce
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    (wat.core/drop (wat.type/PersistentVector :- [wat.type/i64] 1 2 3) 1)))

(wat.core/defn user/p-concat [] :- wat.type/i64
  (wat.core/foldl
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    (wat.core/concat (wat.type/PersistentVector :- [wat.type/i64] 1 2 3) (wat.type/PersistentVector :- [wat.type/i64] 1 2 3))))

(wat.core/defn user/fold-bare-pv [xs :- wat.type/PersistentVector] :- wat.type/i64
  (wat.core/foldl
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    xs))

(wat.core/defn user/fold-bare-vec [xs :- wat.type/Vector] :- wat.type/i64
  (wat.core/foldl
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    xs))

(wat.core/defn user/map-bare-pv [xs :- wat.type/PersistentVector] :- wat.type/i64
  (wat.core/reduce
    (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64 (wat.i64/+ acc x))
    0
    (wat.core/map
      (wat.core/fn [x :- wat.type/i64] :- wat.type/i64 (wat.i64/* x 2))
      xs)))

