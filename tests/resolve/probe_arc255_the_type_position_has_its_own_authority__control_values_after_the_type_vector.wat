;; ★★★ THE GUARD THAT FAILS UNDER SABOTAGE — and only when RUN.
;;
;; A `:-` form is NOT all types. `(:wat::core::Vector :- [T] v1 v2)` carries a type-argument
;; vector AND live VALUE arguments, and `(wat.core/str 1)` is a namespaced symbol that
;; `normalize` MUST rewrite to `:wat::core::str`.
;;
;; An implementation that treats the whole `:-` subtree as opaque data leaves the symbol
;; un-rewritten. Measured: that version `--check`s at EXIT 0 and DIES AT RUNTIME with
;; `#wat.runtime/UnboundSymbol {:name "wat.core/str"}`.
;;
;; A Vector, not a HashSet: the rendering must be ORDER-DETERMINISTIC so the row can assert
;; the whole value exactly instead of a loose `contains`.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [v (:wat::core::Vector :- [:wat::type::Infer] (wat.core/str 1) "b")]
    (:wat::kernel::println (:wat::core::show v))))
