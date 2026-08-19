;; tests/types/probe_arc118_lazy_seq.wat — co-located fixture for probe_arc118_lazy_seq.rs
;;
;; Arc 118 — DISCONFIRMING PROBE for lazy seqs.
;; RED at HEAD: :wat::stream::cons / lazy / empty do not exist.
;; GREEN when the six primitives land in src/seq/.
;;
;; Stone 118.B4-iii — THE WALL (2026-08-18): the sibling `.rs` test's name is
;; `lazy_seq_cons_first_rest_traverses` and its assertion is "a lazy seq builds and TRAVERSES" —
;; that the cons cell holds together and walking it in order (element 1, then element 2) works.
;; `first`/`rest` no longer accept a Stream; `:wat::stream::next` is the one door now. Rewritten
;; onto it below, preserving exactly what the test measures — the traversal order (1 then 2) and
;; that each step is a genuine force (laziness) — not how it used to spell the walk.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [s (:wat::stream::cons 1
                           (:wat::stream::lazy
                             (:wat::stream::cons 2
                               (:wat::stream::lazy (:wat::stream::empty)))))]
    (:wat::core::match (:wat::stream::next s)
      ((:wat::stream::NextOutcome::Item first-value rest)
        (:wat::core::do
          (:wat::kernel::pprintln first-value)
          (:wat::core::match (:wat::stream::next rest)
            ((:wat::stream::NextOutcome::Item second-value _rest2)
              (:wat::core::do (:wat::kernel::pprintln second-value) nil))
            (:wat::stream::NextOutcome::Exhausted
              (:wat::kernel::assertion-failed! "expected a second element, stream exhausted" :wat::core::None :wat::core::None)))))
      (:wat::stream::NextOutcome::Exhausted
        (:wat::kernel::assertion-failed! "expected a first element, stream exhausted" :wat::core::None :wat::core::None)))))
