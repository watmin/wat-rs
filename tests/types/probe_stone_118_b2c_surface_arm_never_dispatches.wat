;; Stone 118.B2c — DISCONFIRMING PROBE for door 1: a `defclause` ARM typed with a SURFACE never
;; dispatches at runtime, even though the checker accepts the call.
;;
;; This fixture LOADS and TYPE-CHECKS cleanly — that is the whole point. B1a (`eab12e05`) taught the
;; checker that a concrete instantiation satisfies a parametric surface. The runtime clause selector
;; (`value_matches_type_by_name`, src/runtime.rs:8760) is a SECOND DOOR that never learned it: its
;; `TypeExpr::Parametric` arm resolves the value to a `StreamContainer` and demands the declared head
;; equal that container's canonical name, so `wat::core::Seqable` can never match ANYTHING.
;;
;; ★ THE CONTROL IS THE LOAD-BEARING HALF. `:my::count-via-defn` is the SAME body with the SAME
;; `(Seqable :- [T])` parameter, as a plain `defn` instead of a `defclause` arm. It must SUCCEED. Without
;; it, the four failing rows below would be satisfied by "(Seqable :- [T]) params are broken everywhere" —
;; which is false, and would send the fix at the wrong door.
;; `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

;; ─── the SUBJECT: one clause, one (Seqable :- [T]) arm ───────────────────────────────────────────────
(:wat::core::defclause :my::count-via-clause
  ([c <- (:wat::core::Seqable :- [T])] -> :wat::core::i64
    (:wat::core::length (:wat::core::into [] (:wat::core::Seqable/seq c)))))

;; ─── the CONTROL: identical body + parameter, as a plain `defn` ────────────────────────────────
(:wat::core::defn :my::count-via-defn :- [T]
  [c <- (:wat::core::Seqable :- [T])] -> :wat::core::i64
  (:wat::core::length (:wat::core::into [] (:wat::core::Seqable/seq c))))

;; ─── the four containers, through the CLAUSE (all four must fail today) ────────────────────────
(:wat::core::defn :my::clause-vector [] -> :wat::core::i64
  (:my::count-via-clause (:wat::core::Vector :wat::core::i64 1 2 3)))

(:wat::core::defn :my::clause-list [] -> :wat::core::i64
  (:my::count-via-clause (:wat::core::List/of 1 2 3)))

(:wat::core::defn :my::clause-persistentvector [] -> :wat::core::i64
  (:my::count-via-clause (:wat::core::PersistentVector 1 2 3)))

(:wat::core::defn :my::clause-stream [] -> :wat::core::i64
  (:my::count-via-clause
    (:wat::stream::cons 1
      (:wat::stream::lazy
        (:wat::stream::cons 2
          (:wat::stream::lazy (:wat::stream::empty)))))))

;; ─── the same four through the CONTROL (all four must succeed today) ───────────────────────────
(:wat::core::defn :my::defn-vector [] -> :wat::core::i64
  (:my::count-via-defn (:wat::core::Vector :wat::core::i64 1 2 3)))

(:wat::core::defn :my::defn-list [] -> :wat::core::i64
  (:my::count-via-defn (:wat::core::List/of 1 2 3)))

(:wat::core::defn :my::defn-persistentvector [] -> :wat::core::i64
  (:my::count-via-defn (:wat::core::PersistentVector 1 2 3)))

(:wat::core::defn :my::defn-stream [] -> :wat::core::i64
  (:my::count-via-defn
    (:wat::stream::cons 1
      (:wat::stream::lazy
        (:wat::stream::cons 2
          (:wat::stream::lazy (:wat::stream::empty)))))))
