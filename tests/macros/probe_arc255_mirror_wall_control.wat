;; arc 255 Stone expand-only-the-mirror-wall — CASE A, the control.
;; macro-error unconditionally as the ENTIRE program body of a defmacro (same shape
;; :wat::core::cond ships for its non-exhaustive-clause abort, wat/core.wat:1455-1464).
;; Never invoked, so startup + compute must succeed unchanged — if the wall ever fires
;; here, macro-error is dead at its only legitimate call site.
(:wat::core::defmacro :probe::always-boom [] -> :wat::WatAST
  (:wat::core::macro-error "boom — control: legal call site, never invoked"))
(:wat::core::defn :user::compute [] -> :wat::core::bool true)
