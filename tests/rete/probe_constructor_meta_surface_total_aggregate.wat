;; tests/rete/probe_constructor_meta_surface_total_aggregate.wat — BRIEF-constructor-meta-audit.md.
;;
;; TOTAL STAYS FALSE (aggregate site), MEASURED: `constructor_meta`'s aggregate branch must NOT
;; become `total: true`, because the only freeze-time wall covering a surface aggregate
;; constructor (`validate_and_reorder_then`, src/rete/validate.rs) covers ONLY a `:then` item's
;; own TOP-level `(:Type arg…)` shape — never a nested operand that is itself a call form. This
;; rule nests `(:cg::Inner :x 1)` as a FIELD VALUE (an operand), not as the `:then` item's own
;; head — Stone B's own widening explicitly stopped flagging a nested call-form operand as
;; categorically illegal (`rhs_operand_can_never_resolve`), but nothing validates its shape
;; ahead of time either.
;;
;; Proof: this rule compiles CLEAN today (pure ∧ deterministic both hold — the pure fix in this
;; same audit is what makes it reach this far at all) and then dies at FIRE time with
;; `RuntimeErrorKind::UnknownFunction` — the generic evaluator `dispatch_keyword_head_value`
;; reaches for a nested surface aggregate call (via `resolve_rhs_value` -> `eval_rhs_expr` ->
;; `eval_inner` -> `eval_list`) has no arm recognizing a bare aggregate-type keyword as a
;; constructor (unlike the TOP-level `:then` item, which `build_insert_fact` special-cases before
;; ever reaching the generic evaluator). If `total?` were armed and this site said `true`, THIS
;; rule would compile clean and abort `fire-rules` — the exact failure mode `total` exists to
;; keep out of a compiled rule.

(:wat::core::defrecord :cg::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :cg::Inner  [x <- :wat::core::i64])
(:wat::core::defrecord :cg::Outer  [inner <- :cg::Inner])

(:wat::rete::defrule :cg::gather
  :when [(:cg::Anchor (?x <- :x))]
  :then [(:cg::Outer :inner (:cg::Inner :x 5))])

;; Returns a String tag so the harness can assert WHICH failure fired without depending on the
;; error's exact rendered text (only the discriminating substring "UnknownFunction").
(:wat::core::defn :user::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::rete::compile rules)
     session (:wat::rete::insert session (:cg::Anchor :x 0))
     fired   (:wat::rete::fire-rules-spec session)
     derived (:wat::rete::query-by-type-string fired "cg::Outer")
     r       (:wat::core::first derived)]
    (:cg::Inner/x (:cg::Outer/inner r))))
