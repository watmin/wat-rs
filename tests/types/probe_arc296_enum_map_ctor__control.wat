;; CONTROL — a well-formed program with no enum construction at all. Establishes what
;; "the checker accepted this" looks like on this binary/fixture path. If this ever goes
;; non-zero, no verdict in this probe means anything.
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "control"))
