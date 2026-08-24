;; tests/types/probe_arc294a_edn_measures_directly_vec.wat — co-located fixture
;;
;; Arc 294.a — a plain EDN VECTOR measures directly via :wat::holon::cosine.
;; RED at HEAD: type-check rejects (Vector :- [i64]) at parameter #1 of :wat::holon::cosine.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::pprintln (:wat::holon::cosine [1 2 3] [1 2 4]))
    nil))
