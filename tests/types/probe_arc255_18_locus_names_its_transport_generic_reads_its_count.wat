;; Stone 255.18 row, rewritten at 255.19 — was `_generic_narrowing.wat.bad`, the pin of a GAP: a
;; generic `(:wat::spawn::Locus :- [T])` could not reach `runner-count`, a defclause keyed on the
;; CONCRETE loci (the rigid T could not unify with a clause's Shared/Wire binding).
;; 255.19 closed the gap by REMOVAL, not by the checker: `runner-count` is now a `Locus` surface
;; method, so a generic locus reads its runner count directly.
;; POSITIVE: one generic reader, called with a process locus (Wire) and a thread locus (Shared).
(:wat::core::defn :probe::count :- [T] [l <- (:wat::spawn::Locus :- [T])] -> :wat::core::i64
  (:wat::spawn::Locus/runner-count l))
(:wat::core::defn :probe::via-process [] -> :wat::core::i64
  (:probe::count (:wat::spawn::process::runner-count 8)))
(:wat::core::defn :probe::via-thread [] -> :wat::core::i64
  (:probe::count (:wat::spawn::thread::runner-count 3)))
