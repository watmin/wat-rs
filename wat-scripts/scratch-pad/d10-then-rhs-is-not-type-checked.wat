;; d10-then-rhs-is-not-type-checked.wat — work-list D10. CURED (grok-rete #344); this file is now
;; the RECORD of the defect, and it no longer reproduces it.
;;
;; ⛔ ON THIS TREE THE STORY HAS TWO DATES. Grok-rete drove D10 live on its own branch (#342,
;; `135b19c37`/`065ed3d91`) and cured it at #344 (`e38b1f46a`, this commit) with a new error kind,
;; `RhsFieldTypeMismatch`. But at replay step #342 — landed on THIS tree one step before this cure
;; arrived — the subject rule below (`:tr::bad`) was ALREADY refused, by a DIFFERENT, independently
;; landed check: `RhsOperandTypeMismatch` (`src/rete/validate/mod.rs`'s `check_rhs_operands`),
;; added by `6df9ba1ab` ("SCORE(277): a rete :then now checks its operand against the field's
;; declared type") — an ANCESTOR of this replay's own base `a3218644d`, so main closed this exact
;; gap on its own, before grok found D10 and before this replay began. #342 therefore banked this
;; file as `.wat.bad` one step early, for the ancestor check's reason rather than this one's. This
;; step's own diff (`RhsFieldTypeMismatch`, `check_then_field_type`) is a SEPARATE, later-landed
;; wall — verified below to be additive, not a duplicate of `RhsOperandTypeMismatch` (D11's nested
;; constructors and D10's non-nested case are not the same site) — see this commit's body for the
;; measurement. Renamed back `.wat.bad` -> `.wat` here because grok's own post-cure content (below)
;; removes the refused subject, so the file loads clean again exactly as grok's comment says it
;; should.
;;
;; THE DEFECT. The rete `:then` RHS did not type-check its field values, while the rest of the
;; language did:
;;
;;   ordinary construction   (:td::Bad :n "x")  ->  #wat.check/TypeMismatch
;;                              ":td::Bad: parameter #1 expects :wat::core::i64; got :wat::core::String"
;;   the SAME construction inside `:then`       ->  compiled, fired, and the derived fact was
;;                              #tr/Bad {:n "not-an-i64"}
;;
;; Driven 2026-09-02 at HEAD `135b19c37` for BOTH a bound `?var` and a literal. The RHS walls that
;; existed were RhsArityMismatch / RhsMissingFields / RhsPositionalConstructionRetired /
;; RhsUnresolvableOperand — every one structural. None typed a value.
;;
;; THE CURE. `RhsFieldTypeMismatch` (`src/rete/validate/error.rs`), produced by
;; `check_then_field_type` (`src/rete/validate/typing.rs`) from both branches of
;; `validate_then_form`. The SUBJECT rule that used to live here — `(:tr::Bad :n ?s)`, a String
;; binding into an i64 field — is now a rule-compile refusal, so KEEPING IT HERE WOULD RED the
;; `every_wat_scripts_file_loads` gate. It moved, with its literal / positional / computed
;; siblings and their `.edn` goldens, to `tests/rete/probe_arc278_D10_then_field_types.rs`.
;;
;; ⛔ NOT a parametric-record problem. `:tr::Box.s` is concretely `:wat::core::String`. The `:when`
;; side DID reason about types — a comparison on an erased `:T` is refused with
;; ConstraintTypeNotComparable — so this was the `:then` surface specifically.
;;
;; ⛔ THE CONTROL IS LOAD-BEARING AND WAS ADDED AFTER THREE VACUOUS PROBES. `FireOutcome::Fired`
;; means the fire COMPLETED, not that a rule produced anything — a probe whose `collect-rules`
;; names an empty namespace compiles ZERO rules and still reports Fired. `Good count: 1` is what
;; proves this probe is live; without it every number below is meaningless.
;; ANTI-VACUITY: the well-typed rule `ok` must derive, or this probe proves nothing. WHAT IT NOW
;; PROVES is the other half of the cure — that a well-typed `:then` still compiles and fires.
(:wat::core::defrecord :tr::Box [k <- :wat::core::i64  s <- :wat::core::String])
(:wat::core::defrecord :tr::Good [n <- :wat::core::i64])
(:wat::rete::defrule :tr::ok
  :when [(:tr::Box (?k <- :k))]
  :then [(:tr::Good :n ?k)])                    ;; i64 into i64 — the CONTROL

;; THE SUBJECT, as it stood, now a rule-compile refusal — see
;; `tests/rete/probe_arc278_D10_then_field_types_bound_var.wat.bad`:
;;
;;   (:wat::rete::defrule :tr::bad
;;     :when [(:tr::Box (?k <- :k) (?s <- :s))]
;;     :then [(:tr::Bad :n ?s)])                  ;; String binding into i64
;;
;;   #wat.rete/RhsFieldTypeMismatch — "defrule `tr::bad`: `:then` insert of `:tr::Bad` fills field
;;   `:n`, declared `:wat::core::i64` (rete `i64`), with operand `?s`, whose type is `string`"

(:wat::rete::defquery :tr::qg :params [] :when [(?f <- :tr::Good)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :tr)
                             (:wat::core::PersistentVector (:tr::qg)))
          [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
          [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f} (:wat::kernel::assertion-failed! :message "mnt")])
     s1 (:wat::core::match (:wat::rete::insert s0 (:tr::Box :k 7 :s "not-an-i64"))
          [:wat::rete::InsertOutcome.Inserted {:session __x} __x]
          [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __a :used __b :staged __c} (:wat::kernel::assertion-failed! :message "c")])]
    (:wat::core::match (:wat::rete::fire-rules s1)
      [:wat::rete::FireOutcome.Fired {:value __f}
        (:wat::core::do
          (:wat::kernel::println "CONTROL Good count:")
          (:wat::kernel::println (:wat::core::length (:wat::rete::query __f (:tr::qg))))
          (:wat::kernel::println "SUBJECT (was `:tr::bad`) is now refused at rule-compile — see the header"))]
      [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __a :used __b :rounds __c} (:wat::kernel::println "ceil")]
      [:wat::rete::FireOutcome.RoundCapExceeded {:cap __a :still-deriving __b} (:wat::kernel::println "roundcap")])))
