;; SPECIMEN — the-little-wat F-091. A name ending in `<` is a lex error, and the error names no
;; file, no line and no column of THIS file — only a byte offset — while locating itself inside
;; crates/wat-reader/src/parser.rs.
;; ⛔ Do not "fix" the name — the name being illegal is the point.
(:wat::core::defn :u::o< [a <- :wat::core::i64] -> :wat::core::i64 a)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:u::o< 1)))
