;; tests/types/probe_arc294a_edn_measures_directly_map.wat — co-located fixture
;;
;; Arc 294.a — a plain EDN MAP measures directly via :wat::holon::cosine (no manual to-holon).
;; RED at HEAD: type-check rejects (HashMap :- [keyword i64]) at parameter #1 of :wat::holon::cosine.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::pprintln (:wat::holon::cosine {:a 1 :b 2} {:a 1 :b 3}))
    nil))
