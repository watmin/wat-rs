;; arc 255 Stone 1a-gamma-i — verify the exact @example text for the six doc-only
;; structs BEFORE writing it into a doc comment (STOP-2: confirm per form with the
;; binary). Scratch, per holon/CLAUDE.md's `.wat` scratch convention.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "quote write-forms:")
    (:wat::kernel::println (:wat::core::write-forms (:wat::core::quote (f x))))
    (:wat::kernel::println "quote ast-kind:")
    (:wat::kernel::println (:wat::core::ast-kind (:wat::core::quote (f x))))

    (:wat::kernel::println "forms length:")
    (:wat::kernel::println (:wat::core::length (:wat::core::forms 1 2 3)))

    (:wat::kernel::println "struct->form write-forms:")
    (:wat::kernel::println (:wat::core::write-forms (:wat::core::struct->form (:wat::holon::CapacityExceeded :cost 7 :budget 3))))

    (:wat::kernel::println "quasiquote write-forms:")
    (:wat::kernel::println (:wat::core::write-forms (:wat::core::quasiquote (:foo (:wat::core::unquote (:wat::i64::+ 1 2))))))

    (:wat::kernel::println "macroexpand-1 write-forms (probe::twice 5 defined below):")
    (:wat::kernel::println (:wat::core::write-forms (:wat::core::macroexpand-1 (:wat::core::quote (:probe::twice2 5)))))

    (:wat::kernel::println "macroexpand write-forms:")
    (:wat::kernel::println (:wat::core::write-forms (:wat::core::macroexpand (:wat::core::quote (:probe::twice2 5)))))
  ))

(:wat::core::defmacro :probe::twice2 [x <- :wat::WatAST] -> :wat::WatAST
  `(:wat::i64::+ ~x ~x))
