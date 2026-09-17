;; Formerly a negative fixture (cross-numeric `=` a check error, THE DECISION).
;; Arc 300 C5 retired 237.8a's comparison-side reject — mixed-numeric `=` now
;; type-checks (still evals to `false`, category-aware). Renamed .wat.bad -> .wat, arc 278 C18: it starts up clean, so the `.bad` claim was false.
(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::= 1 2.0))
