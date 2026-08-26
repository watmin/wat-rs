;; tests/diagnostics/probe_arc296_s7_ensure_reason_enum.wat — 296 S7 RED fixture.
;; A defclause whose :ensure :fn arg type (:String) ≠ the clause return type (:i64).
;; That triggers CheckErrorKind::EnsureFnInvalid with the ArgTypeMismatch reason —
;; the one that today flattens a {arg-type, clause-return-type} type PAIR into a prose
;; String via format!(). S7 makes `reason` a structural enum, so the EDN :reason becomes
;; #wat.kernel/ArgTypeMismatch {:arg-type "…" :clause-return-type "…"} instead of a String.
;; startup MUST fail (check error).
(:wat::core::defclause :my::bad
  ([x <- :wat::core::i64] -> :wat::core::i64
    :ensure (:wat::core::fn [result <- :wat::core::String] -> :wat::core::bool
              (:wat::i64::> 1 0))
    x))
