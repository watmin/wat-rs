;; tests/types/probe_arc118_lazy_seq.wat — co-located fixture for probe_arc118_lazy_seq.rs
;;
;; Arc 118 — DISCONFIRMING PROBE for lazy seqs.
;; RED at HEAD: :wat::stream::cons / lazy / empty do not exist.
;; GREEN when the six primitives land in src/seq/.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [s (:wat::stream::cons 1
                           (:wat::stream::lazy
                             (:wat::stream::cons 2
                               (:wat::stream::lazy (:wat::stream::empty)))))]
    (:wat::core::do
      (:wat::kernel::pprintln (:wat::core::first s))
      (:wat::kernel::pprintln (:wat::core::first (:wat::core::rest s)))
      nil)))
