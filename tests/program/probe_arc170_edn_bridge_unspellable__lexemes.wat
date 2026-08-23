;; Fixture for probe_arc170_edn_bridge_unspellable — one form per class of wat
;; lexeme that EDN cannot spell natively. Each head/annotation below is ONE
;; token to wat-reader; none survives as a native EDN keyword or symbol, so
;; each must cross the wire carried VERBATIM in a tagged record.
;;
;; This file is PARSE-ONLY input to the bridge round trip — it is never frozen
;; or evaluated. The subject under test is the crossing, not the semantics.

;; 1. Type/method accessor — `Vector/length` puts a second `/` in the keyword.
(:wat::core::defn :u::acc [] -> :wat::core::i64 (:wat::core::Vector/length xs))

;; 2. generic type — `<` `>` in a keyword body.
(:wat::core::defn :u::gen [xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64 1)

;; 3. tuple type — the keyword body OPENS with `(`.
(:wat::core::defn :u::tup [] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]) (:wat::core::Tuple 1 "a"))

;; 4. function type — parens AND `->` inside one keyword token.
(:wat::core::defn :u::fnty [g <- :wat::core::Fn(wat::core::i64)->wat::core::i64] -> :wat::core::i64 (g 1))

;; 5. namespace-prefix marker — a TRAILING `::`, so the EDN name is empty.
(:wat::core::defn :my::kernel::pfx {:restricted-to [:my::kernel::]} [] -> :wat::core::i64 1)

;; 6. generic method head — arc 109 stone "the last comma lives in a symbol"
;; retired the comma-carrying `mk<S,R>` spelling (a comma can never enter a
;; symbol body, at any depth); the multi-param binder form below is the
;; replacement. `:-` is an ordinary EDN-spellable keyword and `[S R]` an
;; ordinary vector, so this class no longer exercises an unspellable lexeme —
;; kept in place as corpus (C03) rather than deleted, since the file is a
;; population, not a single-purpose fixture.
(:wat::core::defsurface :u::S :nature :wat::kernel::Peer
  :messages []
  :features [(mk :- [S R] [self <- :u::S] -> :wat::core::i64)])
