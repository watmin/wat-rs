;; Stone 255.22 — an extend-type declares its type parameters (arc 255).
;; The four `wat/seq.wat` Seqable edges, in binder form, with the child spelling its element
;; `Elem` while the surface spells it `T`: they check and run. `seq`'s own name is taken on each
;; container (`<Type>/<method>` is global, and wat/seq.wat already registers `Vector/seq` …), so
;; this is the same surface under another name — `:t22::Elems`, method `elems` — not a second
;; binding of `:wat::core::Seqable`.
(wat.core/defsurface t22/Elems :- [T] :nature wat.type/Struct
  :features [(elems [self :- (t22/Elems :- [T])] :- (wat.stream/Stream :- [T]))])

(wat.core/extend-type :- [Elem] (wat.type/Vector :- [Elem]) (t22/Elems :- [Elem])
  (elems [self] :- (wat.stream/Stream :- [Elem]) (wat.core/seqable->stream self)))

(wat.core/extend-type :- [Elem] (wat.type/PersistentVector :- [Elem]) (t22/Elems :- [Elem])
  (elems [self] :- (wat.stream/Stream :- [Elem]) (wat.core/seqable->stream self)))

(wat.core/extend-type :- [Elem] (wat.type/List :- [Elem]) (t22/Elems :- [Elem])
  (elems [self] :- (wat.stream/Stream :- [Elem]) (wat.core/seqable->stream self)))

(wat.core/extend-type :- [Elem] (wat.stream/Stream :- [Elem]) (t22/Elems :- [Elem])
  (elems [self] :- (wat.stream/Stream :- [Elem]) (wat.core/seqable->stream self)))

;; a generic fn over ANY (Elems :- [T]) — parametric satisfaction.
(wat.core/defn t22/count-of :- [T] [s :- (t22/Elems :- [T])] :- wat.type/i64
  (wat.core/length (wat.core/into [] (t22.Elems/elems s))))

;; a CONCRETE (Elems :- [i64]) bound — the instantiated target unifies, and the element type
;; reaches the body.
(wat.core/defn t22/sum-of [s :- (t22/Elems :- [wat.type/i64])] :- wat.type/i64
  (wat.core/foldl (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64
                       (wat.core/+ acc x))
                     0
                     (wat.core/into [] (t22.Elems/elems s))))

;; a DIRECT method call on a concrete container — the method's result is instantiated by the
;; edge's own binder (`Elem := i64`), so it feeds an i64 consumer.
(wat.core/defn t22/direct-sum [] :- wat.type/i64
  (wat.core/foldl (wat.core/fn [acc :- wat.type/i64 x :- wat.type/i64] :- wat.type/i64
                       (wat.core/+ acc x))
                     0
                     (wat.core/into [] (t22.Elems/elems (wat.type/Vector :- [wat.type/i64] 4 5)))))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println
    (wat.string/join ","
      (wat.type/Vector :- [wat.type/String]
        (wat.core/str (t22/count-of (wat.type/Vector :- [wat.type/i64] 1 2 3)))
        (wat.core/str (t22/count-of (wat.type/PersistentVector :- [wat.type/i64] 1 2 3 4)))
        (wat.core/str (t22/count-of (wat.type/List :- [wat.type/i64] 1 2 3 4 5)))
        (wat.core/str (t22/count-of (wat.stream/cons 1
                                           (wat.stream/lazy
                                             (wat.stream/cons 2
                                               (wat.stream/lazy (wat.stream/empty)))))))
        (wat.core/str (t22/sum-of (wat.type/Vector :- [wat.type/i64] 1 2 3)))
        (wat.core/str (t22/sum-of (wat.type/PersistentVector :- [wat.type/i64] 1 2 3 4)))
        (wat.core/str (t22/sum-of (wat.type/List :- [wat.type/i64] 1 2 3 4 5)))
        (wat.core/str (t22/sum-of (wat.stream/cons 10
                                         (wat.stream/lazy
                                           (wat.stream/cons 20
                                             (wat.stream/lazy (wat.stream/empty)))))))
        (wat.core/str (t22/direct-sum))))))
