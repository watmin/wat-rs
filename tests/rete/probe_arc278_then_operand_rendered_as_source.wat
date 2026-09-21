;; A user's malformed `:then` — an unbound `?var`, about as ordinary a mistake as rete has.
;;
;; The error must name the operand AS WRITTEN. Until 2026-08-27 it rendered the operand with Rust
;; `Debug` and the user was shown:
;;
;;     got wat::core::String "Symbol(Identifier { name: \"?nope\", scopes: {} }, Span { file: … })"
;;
;; — internals, a hygiene scope set, and a nested Span, for a typo. Now it renders `?nope` through
;; the same structural printer `:wat::core::write-forms` uses.
;;
;; The `:location` half was ALREADY correct and is asserted too: this file, line 8, the `?nope`
;; itself. That matters because the ward list still carried a `conformare` finding claiming these
;; sites discard the wat span for `rust_caller_span!()` — stale; the span is real, and it was only
;; the RENDERING that was wrong.
(:wat::core::defrecord :bt::In  [k <- :wat::core::i64])
(:wat::core::defrecord :bt::Out [k <- :wat::core::i64])

(:wat::rete::defrule :bt::r
  :when [(:bt::In (?k :- :k))]
  :then [(:bt::Out :k ?nope)])

(:wat::rete::defquery :bt::q :params [] :when [(?fact :- :bt::Out)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::core::match (:wat::rete::fire-rules
            (:wat::core::match (:wat::rete::insert
              (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector (:bt::r))
                (:wat::core::PersistentVector (:bt::q))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
              (:bt::In :k 1)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
          (:bt::q))))))
