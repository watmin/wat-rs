;; tests/program/wat_arc170_slice_1f_alpha_helpers.wat — co-located fixture for probe rows A-L.
;; Contains the canonical main + type-check functions exercised by rows H, I, J.
;; All rows use startup_beside(file!()) to load this world.

;; row H: println accepts any-T (type-checked by freeze; function body calls println on an i64 param).
(:wat::core::defn :test::p [v <- :wat::core::i64] -> :wat::core::nil (:wat::kernel::println v))

;; row I: eprintln accepts any-T (type-checked by freeze; function body calls eprintln on a String param).
(:wat::core::defn :test::p-eprintln [v <- :wat::core::String] -> :wat::core::nil (:wat::kernel::eprintln v))

;; row J: readln returns polymorphic T (type-checked by freeze; return type unifies with :String annotation).
(:wat::core::defn :test::r [] -> :wat::core::String (:wat::kernel::readln -> :wat::core::String))

;; just-eval probes (rubric) — rows A/B/C/D/E/F/G drive these named zero-arg fns via
;; world.symbols().get(...) + apply_function, instead of an ad-hoc parse_one! literal.
(:wat::core::defn :probe::println-42 [] -> :wat::core::nil (:wat::kernel::println 42))
(:wat::core::defn :probe::println-hello [] -> :wat::core::nil (:wat::kernel::println "hello"))
(:wat::core::defn :probe::println-true [] -> :wat::core::nil (:wat::kernel::println true))
(:wat::core::defn :probe::println-false [] -> :wat::core::nil (:wat::kernel::println false))
(:wat::core::defn :probe::println-tuple [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::Tuple 1 2)))
(:wat::core::defn :probe::eprintln-42 [] -> :wat::core::nil (:wat::kernel::eprintln 42))
(:wat::core::defn :probe::eprintln-hello [] -> :wat::core::nil (:wat::kernel::eprintln "hello"))
(:wat::core::defn :probe::readln-string [] -> :wat::core::String
  (:wat::kernel::readln' 524288 -> :wat::core::String))

