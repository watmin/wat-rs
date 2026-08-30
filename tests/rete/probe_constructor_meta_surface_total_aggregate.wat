;; tests/rete/probe_constructor_meta_surface_total_aggregate.wat — BRIEF-constructor-meta-audit.md
;; (d6c32cf5), RE-POINTED by BRIEF-construction-total-three-walls.md #1.
;;
;; WHAT THIS FORM USED TO DO (`d6c32cf5`): a nested surface aggregate constructor — `(:cg::Inner
;; :x 5)` written as a FIELD VALUE inside `(:cg::Outer :inner …)`, not as the `:then` item's own
;; head — compiled CLEAN (pure ∧ deterministic both held) and then died at FIRE time with
;; `RuntimeErrorKind::UnknownFunction`, unconditionally, regardless of arity:
;; `dispatch_keyword_head_value`'s generic evaluator had NO arm recognizing a bare aggregate-type
;; keyword as a constructor outside `build_insert_fact`'s special-cased TOP-level path. This kept
;; `constructor_meta`'s aggregate site `total: false`.
;;
;; WHAT IT DOES NOW (BRIEF-construction-total-three-walls.md #1 — "the one wall that gets WIRED,
;; not tightened"): nothing about this form was ever illegal — it was simply never wired.
;; `dispatch_keyword_head_value`'s fallback now recognizes a bare keyword resolving to
;; `TypeDef::Aggregate` and delegates to the SAME `eval_kwargs_construct` dispatch the
;; macro-expanded `:wat::core::kwargs-construct` verb already used. This rule now compiles AND
;; FIRES, through both the oracle and the native kernel — the fixture that used to prove a
;; measured gap now proves the fix. `constructor_meta`'s aggregate site is `total: true`.

(:wat::core::defrecord :cg::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :cg::Inner  [x <- :wat::core::i64])
(:wat::core::defrecord :cg::Outer  [inner <- :cg::Inner])

(:wat::rete::defrule :cg::gather
  :when [(:cg::Anchor (?x <- :x))]
  :then [(:cg::Outer :inner (:cg::Inner :x 5))])

(:wat::rete::defquery :cg::q-Outer
  :params []
  :when [(:cg::Outer (?inner <- :inner))])


;; Fires via the WAT ORACLE.
(:wat::core::defn :user::run-oracle [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cg::q-Outer)))
     session (:wat::core::match (:wat::rete::insert session (:cg::Anchor :x 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules$oracle session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     derived (:wat::rete::query fired (:cg::q-Outer))
     r       (:wat::core::first derived)]
    (:cg::Inner/x
      (:wat::core::Option/expect
        (:wat::core::PersistentMap/get r "?inner")
        "q-Outer: ?inner"))))

;; Fires via the NATIVE KERNEL — same rule, same expected value, through the compiled RHS path
;; (`insert`/`fire-rules`) instead of the interpreted oracle.
(:wat::core::defn :user::run-native [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cg::q-Outer)))
     session (:wat::core::match (:wat::rete::insert session (:cg::Anchor :x 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     derived (:wat::rete::query fired (:cg::q-Outer))
     r       (:wat::core::first derived)]
    (:cg::Inner/x
      (:wat::core::Option/expect
        (:wat::core::PersistentMap/get r "?inner")
        "q-Outer: ?inner"))))
