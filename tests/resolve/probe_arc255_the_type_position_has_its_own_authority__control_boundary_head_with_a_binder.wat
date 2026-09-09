;; ★ GUARD — a BOUNDARY head that also carries a `:-` binder.
;;
;; `normalize`'s one invariant is "never rewrite a symbol in a data position", and a boundary
;; head (`quote`/`forms`/`quasiquote`/`match`/`make-rule`) CAPTURES ITS ARGUMENTS AS DATA.
;; That outranks the binder shape, so the `:-` path must be classified AFTER the Boundary and
;; gated on `Ordinary`.
;;
;; Measured: testing the binder FIRST made this file resolve `my.app/never-defined` from inside
;; QUOTED data and report UnresolvedReferences. Clean main never looks there — it reports the
;; enclosing form's own ArityMismatch, and that is what must survive.
;;
;; No corpus form pairs a boundary head with a binder today. The row keeps the invariant
;; structural rather than incidental.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::show (:wat::core::quote :- [wat.type/i64] (my.app/never-defined 1)))))
