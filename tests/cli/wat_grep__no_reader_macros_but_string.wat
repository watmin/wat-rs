;; wat_grep__no_reader_macros_but_string.wat — a TARGET file with NO reader macros but ONE
;; ordinary, hand-written string literal. Demonstrates the STOP-4 finding in this rider's
;; report: `ast-name` on a StringLit returns the UNQUOTED content (arc 279, documented,
;; `src/edn_shim.rs::eval_ast_name`), while `Span` covers the token INCLUDING both quote
;; characters — so `end-col - col == length(ast-name(node))` can NEVER hold for a string-kind
;; node, written or not. This is NOT the reader-macro phantom class G5/G6 were designed around;
;; it is a second, independent way `Written` under-counts `Named`, over EVERY string literal in
;; the corpus (measured: 10123 of 11534 named-not-written nodes corpus-wide).
(:wat::core::defn :user::greet [] -> :wat::core::String
  "hello")
