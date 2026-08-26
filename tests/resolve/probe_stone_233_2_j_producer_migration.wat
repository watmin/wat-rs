;; tests/resolve/probe_stone_233_2_j_producer_migration.wat
;; just-eval fixture for probe_stone_233_2_j_producer_migration.rs — probes 1+2 (the
;; behavioral guard: a producer-tagged TrackedValue survives eval, provenance intact).
;; The Rust driver fetches this defn's OWN body AST (Function::body) and evals it via
;; eval_in_frozen directly, to get back the raw TrackedValue (provenance included) —
;; call_beside/apply_function only ever return the unwrapped Value.
(:wat::core::defn :user::probe [] -> :wat::core::keyword
  (:wat::keyword::from-string "wat::core::nil"))
