;; wat/eval.wat — arc 296 J: `:wat::eval::*` step/walk/form enums, declared in wat.
;;
;; Eval-control sum types formerly registered as hand-written `EnumDef`
;; literals in `src/types.rs`. wat is the source of truth; Rust consumes
;; them via `wat_enum_register_from!`.
;;
;; Load order: after `wat/core.wat`. Payloads include `:wat::WatAST` (builtin).

;; :wat::eval::StepResult — populated in the Ok slot of the :Result
;; returned by :wat::eval-step! (arc 068). Two variants distinguish
;; "one rewrite happened, here's the next form" from "this is the
;; terminal value." Both arms carry a payload — the next form as
;; wat::WatAST, the terminal value as wat::holon::HolonAST. The
;; consumer drives the loop, feeding StepNext.form back in until
;; StepTerminal arrives.
;; PURITY Impure: in-locus eval-step control; carries WatAST forms — Impure (never crosses)
(:wat::core::defenum :wat::eval::StepResult :wat::enum::Impure
  :StepNext [form <- :wat::WatAST]
  :StepTerminal [value <- :wat::WatAST]
;; Arc 070 — distinguishes "input was already a value; no
;; work happened" from "this step reduced a redex." Fires
;; on holon-value-shape WatASTs (`to-watast(holon)` round-
;; trips like Bundle's bare-list lift, holon-constructor
;; forms with all-canonical args, primitive literals).
;; Walkers and tracers care about chain-length 0 vs ≥ 1.
  :AlreadyTerminal [value <- :wat::WatAST])

;; Arc 070 — (:wat::eval::WalkStep :- [A]) — what the visitor passed to
;; :wat::eval::walk returns. Two variants:
;;
;;   Continue(acc')        — keep walking; acc' is the new
;;                           accumulator. If the current
;;                           step-result was StepNext, walk
;;                           recurses on the next form. If it
;;                           was StepTerminal/AlreadyTerminal,
;;                           walk returns (terminal, acc').
;;   Skip(terminal, acc')  — caller has its own answer for this
;;                           coordinate (cache hit, etc.).
;;                           Walk stops here and returns
;;                           (terminal, acc').
;;
;; Generic over A so the consumer's accumulator can be any
;; type — cache, trace, counter, tier, etc.
;; PURITY Impure: in-locus walk control — Impure (never crosses)
(:wat::core::defenum :wat::eval::WalkStep :- [A] :wat::enum::Impure
  :Continue [acc <- :A]
  :Skip [terminal <- :wat::WatAST  acc <- :A])

;; Arc 170 — (:wat::eval::FormOutcome :- [T]) — what `:wat::eval-with-defs!` returns:
;; the outcome of handing ONE form to a world built from a definition set.
;;
;; Why FOUR variants and not a Result. A caller submitting a line (a REPL's `E`
;; phase is the motivating consumer) can receive four genuinely different answers,
;; and the substrate CANNOT let them infer which from an error:
;;
;;   :Declared    — the form was a declaration; the world took it. Nullary on
;;                  purpose — the caller already holds `defs` and `form`, so the
;;                  `conj` is theirs; returning a grown set would mint a second
;;                  authority for one fact. (Shape precedent: SendOutcome::Sent.)
;;   :Evaluated   — the form was an expression; here is its value.
;;   :CheckFailed — the form did not survive the freeze (parse / macro-expand /
;;                  type-check). A STATIC failure: nothing ran. The cause is the
;;                  freeze error's OWN `error_edn()` floor record — a navigable
;;                  tagged value (`#wat.check/CheckErrors {…}`,
;;                  `#wat.resolve/UnresolvedReferences {…}`, …), NEVER that tree
;;                  flattened into a String. `:wat::core::Error` is a structural
;;                  surface (message/location/causes, wat/core.wat:1816) and
;;                  `error_edn()`'s whole contract is to place exactly those three
;;                  floor keys ahead of the variant fields — so every freeze error
;;                  satisfies it as-is, with its causes chain intact and walkable.
;;   :Raised      — the form type-checked, ran, and unwound. A DYNAMIC failure.
;;                  (A form that RETURNS an Err value is `:Evaluated`, not `:Raised`.)
;;
;; The static/dynamic split is not stylistic — collapsing both into one `cause`
;; slot is the overloaded-bucket Ruling A forbids (DESIGN-service-io-budgets.md),
;; and the two carriers are genuinely different types: every `EvalError.kind` is a
;; dynamic-eval kind (see the EvalError doc above — "unknown-function",
;; "type-mismatch", "runtime-error"), none of which can describe a freeze
;; rejection. `StartupError` is what the freeze itself returns (freeze.rs) — it is
;; reused rather than duplicated. Its single `message` field is thin for a REPL
;; (which wants the location); growing it is that type's own follow-up, exactly
;; the "extensible … if a real consumer surfaces" its comment invites.
;;
;; `:Declared` cannot be inferred from a refusal, which is why it is a first-class
;; answer: MEASURED (wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat),
;; `defn` and `defrecord` fail eval with `unknown-function` — byte-identical to a
;; TYPO — because both are macros with no runtime verb to find.
;;
;; Impure, for (RecvOutcome :- [O])'s reason exactly: the caller's live `Environment` is
;; threaded through unchanged so impure bindings survive, so `T` may be a live
;; resource, and a Pure marking would lie the moment it is. `T` is phantom in three
;; of the four variants — the precedent is `(:wat::service::Outcome :- [S R O])`, whose
;; `O` is phantom for Reply/Stop/NoReply.
;;
;; Named by an intueri cast (2026-07-28), which also ruled the verb
;; `:wat::eval-with-defs!` and this `:wat::eval::` home — the namespace the eval
;; TYPES already use (StepResult, WalkStep above); `:wat::core::` would have been
;; drift, and a bare `Outcome` would read ambiguously beside
;; `:wat::service::Outcome` in the defservice handler that is its first consumer.
;; PURITY Impure: T may be a live resource — see above
(:wat::core::defenum :wat::eval::FormOutcome :- [T] :wat::enum::Impure
  :Declared
  :Evaluated [value <- :T]
  :CheckFailed [cause <- :wat::core::Error]
  :Raised [cause <- :wat::core::EvalError])
