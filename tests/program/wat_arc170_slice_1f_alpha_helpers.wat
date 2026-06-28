;; tests/program/wat_arc170_slice_1f_alpha_helpers.wat — co-located fixture for probe rows A-L.
;; Contains the canonical main + type-check functions exercised by rows H, I, J.
;; All rows use startup_beside(file!()) to load this world.

;; row H: println accepts any-T (type-checked by freeze; function body calls println on an i64 param).
(:wat::core::defn :test::p [v <- :wat::core::i64] -> :wat::core::nil (:wat::kernel::println v))

;; row I: eprintln accepts any-T (type-checked by freeze; function body calls eprintln on a String param).
(:wat::core::defn :test::p-eprintln [v <- :wat::core::String] -> :wat::core::nil (:wat::kernel::eprintln v))

;; row J: readln returns polymorphic T (type-checked by freeze; return type unifies with :String annotation).
(:wat::core::defn :test::r [] -> :wat::core::String (:wat::kernel::readln -> :wat::core::String))

