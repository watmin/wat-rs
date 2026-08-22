;; tests/resolve/arc109_type_reference_must_resolve_row4_parametric.wat — row 4.
;;
;; A phantom type as a PARAMETRIC form's HEAD, `(:wat::cache::NoSuchType :- [:wat::core::i64])`
;; — the `:-` binder-marker spelling `defservice`/`defsurface` annotations emit. The arg
;; (`:wat::core::i64`) is a real, resolvable type deliberately: a walk that checks only
;; `args` and never the `Parametric.head` itself would pass this fixture wrongly.
(:wat::core::defn :user::f [x <- (:wat::cache::NoSuchType :- [:wat::core::i64])] -> :wat::core::i64 0)
