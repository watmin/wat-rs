;; tests/value/wat_arc220_char_supplementary_plane.wat — NEGATIVE fixture.
;; startup_from_file must return Err.
;; A supplementary-plane char literal (\😀 = U+1F600) is
;; rejected by the WAT lexer at lex time with a BMP-only diagnostic.
(:wat::core::defn :t::main [] -> :wat::core::nil \😀)
