;; Stone 118.B2d — door 2, the POSITIVE fixture. MUST CHECK CLEAN.
;;
;; ⛔ THIS FILE WAS `_neg.wat.bad` AND IT INVERTED. It was committed as the WITNESS of the defect,
;; asserting that a `Vector<i64>` routed through `Seqable/seq` loses its element type. Stone 118.B2d
;; landed and it now checks clean — that inversion IS the stone's acceptance.
;;
;; `Seqable/seq` is declared `[self <- Seqable<T>] -> Stream<T>`. Called on a `Vector<i64>` it must
;; yield `Stream<i64>`. Before B2d it yielded `Stream<T>` with `T` FREE, so the result could not be
;; handed to any consumer wanting a concrete element type — the ONE method `Seqable<T>` has could not
;; have its result typed.
;;
;; Mechanism (src/check.rs:4926-4948, path (1)): the resolution looks up the satisfier's registered
;; `<ConcreteType>/<method>` scheme, on the documented assumption that `extend-type` already
;; substituted the surface's `<T>` to a CONCRETE binding ("e.g. T=i64 for
;; (extend-type :IntBox :Holds<i64>)"). `Seqable<T>` is satisfied by GENERIC CONTAINERS —
;; `(extend-type :wat::core::Vector :wat::core::Seqable<T>)` binds `T -> T`, a VARIABLE — so the
;; stored scheme's return stays `Stream<T>` and nothing ever instantiates it from the receiver.
;;
;; THE FIX: path (1) now binds the surface's params from the RECEIVER's args when the arities line
;; up. No new state — a satisfier that bound CONCRETELY leaves no surface param in its scheme, so the
;; `rename` is the identity there and those schemes are byte-identical. The safety is structural.
;;
;; Its sibling `_pos.wat` still carries the two rows that bound the defect: a concrete container fed
;; DIRECTLY (B1a, always worked) and a POLYMORPHIC consumer swallowing the result (which is why
;; nothing caught this for a month).

(:wat::core::defn :my::eats-concrete
  [c <- (:wat::core::Seqable :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::length (:wat::core::into [] (:wat::core::Seqable/seq c))))

;; THE ROW THAT WAS RED — a Vector<i64> routed through the surface method. Now yields Stream<i64>.
(:wat::core::defn :my::via-surface-method [] -> :wat::core::i64
  (:my::eats-concrete (:wat::core::Seqable/seq (:wat::core::Vector :wat::core::i64 1 2 3))))
