;; tests/rete/probe_arc278_ast_to_source.wat — co-located fixture for the sibling .rs, slurped
;; via call_beside(file!()). Arc 278 Stone 1: `:wat::core::ast->source` must print VERBATIM
;; `::`-source (does NOT dial `::`→`.` the way `write-forms` does), so `read-string` re-reads it
;; back to the identical form.

;; round-trip: read-string(ast->source(form)) reproduces the SAME form. `read-string` wraps its
;; parsed top-level forms in a program-List, so `(first (ast->children ...))` unwraps back to
;; the single quoted form for comparison. `form` exercises List + Vector + Keyword + Symbol + a
;; literal (the sift predicate shape).
(:wat::core::defn :user::ast-to-source-round-trips [] -> :wat::core::bool
  (:wat::core::let
    [form (:wat::core::quote
            (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::+ x 1)))]
    (:wat::core::=
      form
      (:wat::core::first
        (:wat::core::ast->children
          (:wat::core::match (:wat::core::read-string (:wat::core::ast->source form)) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))))

;; GUARD (anti-write-forms): ast->source must print the raw `::` token text, never the
;; `.`-dialed write-forms notation — the form's head keyword is `:wat::core::fn`, so its
;; printed source must still contain `::`.
(:wat::core::defn :user::ast-to-source-is-verbatim-colon-colon [] -> :wat::core::bool
  (:wat::core::let
    [form (:wat::core::quote
            (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::+ x 1)))]
    (:wat::string::contains? (:wat::core::ast->source form) "::")))
