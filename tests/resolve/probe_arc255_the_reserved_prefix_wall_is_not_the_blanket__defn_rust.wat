;; A USER `defn` claiming a name under `:rust::*`. Rust modules express their own names;
;; userland wat may not mint them. MUST be refused.
(:wat::core::defn :rust::x::sneaky [] -> :wat::core::i64 1)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
