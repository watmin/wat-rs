;; Stone 118.B2d — door 2, the POSITIVE half. MUST CHECK CLEAN.
;;
;; ★ THIS IS THE LOAD-BEARING FIXTURE. The `_neg` sibling failing proves nothing on its own —
;; "Seqable<T> is broken" would explain it equally, and would send the fix at the wrong door. These
;; two rows bound the defect precisely:
;;
;;   ROW 1 (direct)      — a concrete container satisfies a CONCRETE surface instantiation.
;;                         This is stone B1a (`eab12e05`) and it still works. So the SURFACE and the
;;                         CHECKER's satisfaction rule are both fine.
;;   ROW 2 (polymorphic) — the surface method's `Stream<T>` result IS usable, as long as the consumer
;;                         is equally polymorphic. This is WHY nothing caught door 2 for a month:
;;                         `core-seqable.wat` only ever fed `Seqable/seq` into `into`, whose Stream
;;                         clause is itself `Stream<T>`, so a free `T` unified happily and the row
;;                         went green.
;;
;; Together with `_neg`: the loss is specific to a surface METHOD'S RETURN meeting a CONCRETE
;; consumer. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

(:wat::core::defn :my::eats-concrete
  [c <- :wat::core::Seqable<wat::core::i64>] -> :wat::core::i64
  (:wat::core::length (:wat::core::into [] (:wat::core::Seqable/seq c))))

;; ROW 1 — the container fed DIRECTLY. B1a's guarantee.
(:wat::core::defn :my::direct [] -> :wat::core::i64
  (:my::eats-concrete (:wat::core::Vector :wat::core::i64 1 2 3)))

;; ROW 2 — a POLYMORPHIC consumer swallows the `Stream<T>` result without complaint.
(:wat::core::defn :my::eats-polymorphic<T>
  [s <- :wat::stream::Stream<T>] -> :wat::core::i64
  (:wat::core::length (:wat::core::into [] s)))

(:wat::core::defn :my::via-surface-method-into-polymorphic [] -> :wat::core::i64
  (:my::eats-polymorphic (:wat::core::Seqable/seq (:wat::core::Vector :wat::core::i64 1 2 3))))
