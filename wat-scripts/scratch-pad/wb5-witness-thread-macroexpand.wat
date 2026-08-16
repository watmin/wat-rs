;; wb5-witness-thread-macroexpand.wat — 296 Wave B5 rider: macroexpand-first per project
;; doctrine before adjudicating `witness_thread_first_empty_step_panics_at_expansion` and
;; `witness_thread_last_empty_step_desugars_to_call_on_acc`. Dumps the expanded form of both
;; `(-> 5 ())` and `(->> 5 ())` — the empty-list-step witnesses — via
;; `:wat::core::macroexpand` + `write-forms`, pure introspection, never executed.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "=== (:wat::core::-> 5 ()) expansion ===")
    (:wat::kernel::println (:wat::core::write-forms
      (:wat::core::macroexpand (:wat::core::quote
        (:wat::core::-> 5 ())))))
    (:wat::kernel::println "=== (:wat::core::->> 5 ()) expansion ===")
    (:wat::kernel::println (:wat::core::write-forms
      (:wat::core::macroexpand (:wat::core::quote
        (:wat::core::->> 5 ())))))))
