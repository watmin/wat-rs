;; probe-118B1-seqable.wat — stone 118.B1's gate rows 2–7, as a runnable program.
;;
;; `(:wat::core::Seqable :- [T])` is the type the seven `-stream` twins were a workaround for.
;; This probe proves it is REAL — not merely declared.
;;
;; ★ ROW 6 IS THE STONE. Rows 2–5 exercise `Seqable/seq` on each container directly, which only
;; shows the four `extend-type`s registered. Row 6 CALLS a generic fn whose parameter type is
;; `(Seqable :- [T])` with all four containers — and that is the thing 118.3-B had to fix, because a
;; concrete builtin did not unify against a PARAMETRIC surface parameter. A probe that only
;; DECLARES such a fn proves nothing: that exact mistake was made on 2026-08-17 and reported as
;; "the full design type-checks" when adding call sites made it RED four times over.
;; `[[feedback_a_green_test_can_prove_nothing]]`
;;
;; ROW 7 is the laziness guard: `seq` on an INFINITE stream must not force the chain. If it
;; materialised, this program would never terminate — so termination itself is the assertion.

;; An unbounded source — row 7's subject.
(:wat::core::defn :probe::nat
  [i <- :wat::core::i64] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::stream::lazy
    (:wat::stream::cons i (:probe::nat (:wat::core::+ i 1)))))

;; ★ ROW 6 — ONE generic fn over ANY (Seqable :- [T]). This is the shape the whole route exists for:
;; after B2 every sequence verb in the stdlib looks like this, and so does a user's.
(:wat::core::defn :probe::count-via-seq :- [T]
  [s <- (:wat::core::Seqable :- [T])] -> :wat::core::i64
  (:wat::core::length (:wat::core::into [] (:wat::core::Seqable/seq s))))

;; Rows 2–5 — `seq` on each container drains to the same elements, in order. Joined so the ORDER
;; is asserted, not just the count: a coercion that reversed or shuffled would pass a length check.
(:wat::core::defn :probe::elems-of :- [T]
  [s <- (:wat::core::Seqable :- [T])] -> :wat::core::String
  (:wat::core::string::join "," (:wat::core::into [] (:wat::core::Seqable/seq s))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; rows 2-5 — order-preserving drain through the surface, one line per container.
    (:wat::kernel::println
      (:wat::core::string::join " | "
        (:wat::core::Vector :wat::core::String
          (:probe::elems-of (:wat::core::Vector :wat::core::i64 1 2 3))
          (:probe::elems-of (:wat::core::PersistentVector 1 2 3 4))
          (:probe::elems-of (:wat::core::List/of 1 2 3 4 5))
          (:probe::elems-of (:wat::stream::cons 7
                              (:wat::stream::lazy
                                (:wat::stream::cons 8
                                  (:wat::stream::lazy (:wat::stream::empty)))))))))
    ;; ★ row 6 — the generic fn CALLED with all four. Expect 3,4,5,2.
    (:wat::kernel::println
      (:wat::core::string::join ","
        (:wat::core::Vector :wat::core::i64
          (:probe::count-via-seq (:wat::core::Vector :wat::core::i64 1 2 3))
          (:probe::count-via-seq (:wat::core::PersistentVector 1 2 3 4))
          (:probe::count-via-seq (:wat::core::List/of 1 2 3 4 5))
          (:probe::count-via-seq (:wat::stream::cons 1
                                   (:wat::stream::lazy
                                     (:wat::stream::cons 2
                                       (:wat::stream::lazy (:wat::stream::empty)))))))))
    ;; row 7 — LAZINESS. `seq` over an INFINITE source, bounded by `take`. Termination IS the
    ;; assertion; a materialising `seq` would hang here rather than print.
    (:wat::kernel::println
      (:wat::core::string::join ","
        (:wat::core::into []
          (:wat::core::take (:wat::core::Seqable/seq (:probe::nat 0)) 3))))))
