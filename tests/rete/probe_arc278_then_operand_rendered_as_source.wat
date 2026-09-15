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
  :when [(:bt::In (?k <- :k))]
  :then [(:bt::Out :k ?nope)])

(:wat::rete::defquery :bt::q :params [] :when [(?fact <- :bt::Out)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::rete::fire-rules
            (:wat::rete::insert
              (:wat::rete::compile-all (:wat::core::PersistentVector (:bt::r))
                (:wat::core::PersistentVector (:bt::q)))
              (:bt::In :k 1)))
          (:bt::q))))))
