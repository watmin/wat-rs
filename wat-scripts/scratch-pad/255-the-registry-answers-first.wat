;; 255 — the-registry-answers-first: the eleven facts the retired prefix guesses carried, now
;; answered by the REGISTRY (each verb's own `@Totality`) instead of by
;; `src/rete/purity.rs::intrinsic_meta`'s `:wat::string::`/`:wat::regex::` and `:wat::edn::`
;; prefix guesses (DESIGN-STONE-the-registry-answers-first.md).
;;
;; ── WHY THIS PROBE USES `:wat::rete::total?`, NOT `:wat::rete::compile-all` ─────────────────────
;;
;; The brief's own sketch was a `where` using `(:wat::string::concat …)`, expected ADMITTED via
;; `compile-all`. Measured against the PRE-EXISTING binary (before any Rust change in this stone),
;; that is not what happens — for ANY of the eleven, nested or top-level:
;;
;;   (:wat::rete::where (:wat::string::concat ?tag "!") …nested inside a rete op…)
;;     => "compile-condition: where expr is not a rete primitive — ':wat::string::concat'
;;         is not a rete primitive; a where admits only :wat::rete:: ops"
;;   (:wat::rete::where (:wat::string::contains? ?tag "cat"))         [top-level]
;;     => "compile-condition: where expr is not a rete primitive — ':wat::string::contains?'
;;         is not a rete primitive; a where admits only :wat::rete:: ops"
;;
;; `wat/rete/compile.wat`'s `compile-condition`/`then-item-fence` both require
;; `(and is-pure is-det is-total is-rete)` — LAW A (`is-rete`, `:wat::rete::primitive?`) refuses
;; ANY core-spelled computation unconditionally, independent of purity/determinism/totality
;; (`src/rete/purity.rs::classify_expr`'s own doc: *"being pure, deterministic and total does not
;; make an op rete … `:wat::core::>` is all three and is still refused"*). None of the eleven are
;; rete-vocabulary members (constructors/accessors/rete-namespaced ops), so `where`/`then` refuse
;; all eleven BOTH before and after this stone — that surface cannot demonstrate the fact-move at
;; all; every `where`/`then` case is REFUSED, unconditionally, with or without this stone's change.
;;
;; The consumer that DOES read `intrinsic_meta`'s totality axis in isolation (no Law A conjunct)
;; is `:wat::rete::total?`/`pure?`/`deterministic?` — the standalone introspection predicates
;; (`src/rete/purity.rs`, ruled Pure∧Deterministic∧Total themselves, arc 255 Stone P6-c-W5a),
;; which classify a quoted expression against exactly ONE axis. This file uses those.
;;
;; ── BEFORE THIS STONE (measured, pre-existing `target/release/wat`, 2026-08-31) ─────────────────
;;
;;   concat pure?             true
;;   concat deterministic?    true
;;   concat total?            true      <- the guess's claim (Arc 255 Stone F's hand-list)
;;   contains? total?         true
;;   edn/read-foreign total?  true
;;   split total?             false     <- Unreviewed default-deny, unaffected by the guess
;;
;; ── AFTER THIS STONE (expected, once the registry answers) ──────────────────────────────────────
;;
;; Ten of the eleven move IN UNCHANGED: `length`/`trim`/`to-lowercase`/`contains?`/
;; `starts-with?`/`ends-with?`/`empty?` and `edn::{read-foreign,ForeignRecord/get,
;; ForeignRecord/class}` are `@Totality Total`, re-derived from each body (see
;; `src/intrinsic/string.rs` / `src/intrinsic/edn.rs`), same verdict the guess argued.
;;
;; ⛔ `:wat::string::concat` is the ELEVENTH and it does NOT move in unchanged — re-reading its own
;; body (`eval_string_concat`, `src/intrinsic/string.rs`) shows it raises `ArityMismatch` on a
;; zero-arg call, and `check.rs::infer_string_concat`'s own comment confirms the checker ACCEPTS
;; that call as well-typed (*"the checker accepts arity 0 … so the runtime owns the diagnostic"*).
;; Per `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md` — *"A raise or a panic
;; is NOT an outcome … The test is matchability, not whether the failure is … well-diagnosed"* —
;; that makes concat `@Totality Partial`, not the guess's `Total`. So:
;;
;;   concat pure?             true      (unchanged — Purity untouched, per this stone's own rule)
;;   concat deterministic?    true      (unchanged — Determinism untouched)
;;   concat total?            FALSE     <- CHANGES. The guess's "always return for any two
;;                                          strings" quietly assumed away concat's own zero-arg
;;                                          call, which its variadic signature admits.
;;   contains? total?         true      (unchanged)
;;   edn/read-foreign total?  true      (unchanged)
;;   split total?             false     (unchanged — still Unreviewed, a different stone's work)
;;
;; No `where`/`then` fence in the corpus is affected either way: the corpus's actual fenced usage
;; of concat is via the independently-declared rete alias `:wat::rete::string::concat`
;; (`src/rete/vocabulary.rs`, its OWN fixed-arity-2 `OpMeta{total:true}`, genuinely total by
;; construction — untouched, STOP-5), not the variadic core spelling this stone re-measured.
;;
;; Run: target/release/wat wat-scripts/scratch-pad/255-the-registry-answers-first.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [concat-pure  (:wat::rete::pure?
                    (:wat::core::quote
                      (:wat::core::fn [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String
                        (:wat::string::concat a b))))
     concat-det   (:wat::rete::deterministic?
                    (:wat::core::quote
                      (:wat::core::fn [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String
                        (:wat::string::concat a b))))
     concat-total (:wat::rete::total?
                    (:wat::core::quote
                      (:wat::core::fn [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String
                        (:wat::string::concat a b))))
     contains-total (:wat::rete::total?
                    (:wat::core::quote
                      (:wat::core::fn [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::bool
                        (:wat::string::contains? a b))))
     read-foreign-total (:wat::rete::total?
                    (:wat::core::quote
                      (:wat::core::fn [s <- :wat::core::String] -> (:wat::edn::ReadForeignOutcome :- [:wat::core::i64])
                        (:wat::edn::read-foreign s))))
     split-total (:wat::rete::total?
                    (:wat::core::quote
                      (:wat::core::fn [a <- :wat::core::String b <- :wat::core::String] -> (:wat::core::Vector :- [:wat::core::String])
                        (:wat::string::split a b))))]
    (:wat::kernel::println (:wat::string::concat "concat pure?             " (:wat::core::bool::to-string concat-pure)))
    (:wat::kernel::println (:wat::string::concat "concat deterministic?    " (:wat::core::bool::to-string concat-det)))
    (:wat::kernel::println (:wat::string::concat "concat total?            " (:wat::core::bool::to-string concat-total)))
    (:wat::kernel::println (:wat::string::concat "contains? total?         " (:wat::core::bool::to-string contains-total)))
    (:wat::kernel::println (:wat::string::concat "edn/read-foreign total?  " (:wat::core::bool::to-string read-foreign-total)))
    (:wat::kernel::println (:wat::string::concat "split total?             " (:wat::core::bool::to-string split-total)))))
