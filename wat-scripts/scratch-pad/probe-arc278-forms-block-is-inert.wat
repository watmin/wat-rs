;; PROBE — gate: a `(:wat::core::forms …)` block holding two `defrecord`s and a `defrule` that
;; references them must LOAD CLEAN.
;;
;; THE MECHANISM UNDER TEST (DESIGN-STONE-the-expander-reads-the-boundary-door.md). `forms` is
;; `Boundary::AllData` at the door (`src/resolve/boundary.rs:83`) — the RESOLVER never walks into
;; it, because the universe it names is not this one. But `src/macros/expand.rs:441` carried its
;; own hand-rolled three-head data-form set (`quasiquote`/`quote`/`holon::literal`) that OMITTED
;; `forms`. So a `forms` block's arguments fell through to full-Lisp macro dispatch, the `defrule`
;; inside expanded to `(:wat::rete::make-rule …)`, and `src/rete/validate.rs`'s `walk_for_make_rule`
;; — a raw descent that consults no `Boundary` — validated that expansion's fact types against the
;; PARENT's local `TypeEnv`, which does not see facts declared two lines above inside the very same
;; forms block. `forms` was data to the resolver and code to the expander.
;;
;; ⛔ MEASURED 2026-08-12, BEFORE the expander fix (boundary.rs:83's `AllData` arm already had
;; `define` dropped; expand.rs:441's literal set still hand-rolled, `forms` absent) — verbatim
;; `./target/release/wat --check` on this file, exit 1:
;;
;;   #wat.rete/ReteCheckErrors {:message "1 rete rule validation error" ... :errors
;;     [#wat.rete/UnknownFactType {:rule "probe278b::rule-userfn" :fact-type "probe278b::Temp"
;;       :span #wat.core/Span {:file "wat-scripts/scratch-pad/probe-arc278-forms-block-is-inert.wat"
;;       :line 37 :col 14 :end #wat.core.Option/Some [#wat.core/Pos {:line 37 :col 43}]}}]}
;;
;; `:line 37 :col 14`→`:col 43` is `(:probe278b::Temp (?c <- :c))` — the FACT-PATTERN CONDITION
;; inside the defrule's `:when`, three forms below the `defrecord` that declares
;; `:probe278b::Temp` in the very same forms block. That is the design stone's symptom exactly.
;;
;; ⚠ THE LINE NUMBER IS AS-OF-CAPTURE. `:line 37` was true when the capture was taken; this
;; header has grown since, and the form now sits lower (`:when` at line 46 as written). The
;; number is NOT rewritten to match, because a captured measurement is evidence and editing its
;; digits to fit a moved file would be fabricating one. **The anchor is the FORM** —
;; `(:probe278b::Temp (?c <- :c))` in the `:when` — not the line.
;;
;; That fragility is itself the lesson, and it bit twice here. The rider's first header recorded
;; `:line 33` and called the span "the defrule head": stale by four lines AND misdescribed. The
;; orchestrator re-captured it by reverting `expand.rs` alone, rebuilding, and re-running
;; `--check` against this file — and then shifted it again by writing this very correction. A
;; self-describing file cannot cite its own line numbers stably; cite the form.
;;
;; ★ MEASURED 2026-08-12, AFTER (expand.rs:441 replaced with a `quote_boundary` consult matching
;; `Boundary::AllData | Boundary::Quasiquote`) — clean load, no output, exit 0.
;;
;; GREEN = the forms block is genuinely inert to the expander (as it already was to the resolver);
;; RED naming `UnknownFactType` inside the block is the drift this stone closes.

(:wat::core::defn :probe278b::payload [] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::forms
    (:wat::core::defrecord :probe278b::Temp [c <- :wat::core::i64])
    (:wat::core::defrecord :probe278b::Hot  [c <- :wat::core::i64])
    (:wat::rete::defrule :probe278b::rule-userfn
      :when [(:probe278b::Temp (?c <- :c))]
      :then [(:probe278b::Hot :c ?c)])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat "forms payload length="
      (:wat::core::i64::to-string (:wat::core::length (:probe278b::payload))))))
