;; probe: a set-redef! that is NOT the leading form (defn comes first) —
;; does collect_entry_file_inner's "every setter precedes every non-setter"
;; discipline reject this, or does it fall through as an ordinary top-level
;; form processed later by register_runtime_defs_form?
(:wat::core::defn :user::helper [] -> :wat::core::i64 42)
(:wat::config::set-redef! true)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "reached main"))
