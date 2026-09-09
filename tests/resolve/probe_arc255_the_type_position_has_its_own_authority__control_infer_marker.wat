;; GREEN — `:wat::type::Infer` is a live type-position MARKER (src/types.rs:74), not a
;; declared type: TypeEnv::contains answers false for it. Any fix that validates a `:-`
;; argument against the type registry refuses this, and it has 46 corpus occurrences.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [s (:wat::core::HashSet :- [:wat::type::Infer] "list" "vector")]
    (:wat::kernel::println "y")))
