;; tests/collection/probe_arc278_0d_transform_dispatch_parity_bad.wat — NEGATIVE fixture.
;; Loaded via startup_from_file, asserting startup (type-check) FAILS.
;;
;; A String reducer folded over an i64 PersistentVector must be rejected (element type mismatch).
(:wat::core::defn :user::bad [] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String x <- :wat::core::String] -> :wat::core::String
      (:wat::core::string::concat acc x))
    ""
    (:wat::core::PersistentVector 1 2 3)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
