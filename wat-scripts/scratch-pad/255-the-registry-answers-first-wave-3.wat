;; 255 — the-registry-answers-first-wave-3: the last five hand-verdicts become registrations
;; (DESIGN-STONE-the-registry-answers-first-wave-3.md). Extends waves 1–2's probe shape
;; (`255-the-registry-answers-first{,-wave-2}.wat`) rather than replacing it.
;;
;; ── WHAT THIS WAVE HOMES, AND WHAT IT DELIBERATELY DOES NOT ─────────────────────────────────────
;;
;; Five verbs get a `#[wat_intrinsic]` registration this stone: `:wat::core::aggregate-new` /
;; `kwargs-construct` (`src/intrinsic/record.rs`), `:wat::core::write-forms` / `with-children`
;; (`src/intrinsic/ast.rs`), and `:wat::core::macro-error` (`src/intrinsic/macro_error.rs`, its own
;; file — the one of the five with no pre-existing named fn to delegate to). The DESIGN's other
;; four candidates — `:wat::verify::string`/`file-path`/`http-path`/`s3-path` — are LOCATOR TAGS
;; matched inside `resolve_verify_payload` (`runtime.rs`), never call heads (0 corpus calls); homing
;; them would register verbs that do not exist, so their guard in `rete/purity.rs` is UNTOUCHED
;; (byte-identical) and the control below uses one of them to prove it.
;;
;; ── metadata-of, BEFORE THIS STONE (measured, pre-existing `target/release/wat`, 2026-08-31) ────
;;
;;   aggregate-new meta:                NONE
;;   kwargs-construct meta:             NONE
;;   write-forms meta:                  NONE
;;   with-children meta:                NONE
;;   macro-error meta:                  NONE
;;   verify::string (control) meta:     NONE
;;
;; ── metadata-of, AFTER THIS STONE (expected — this rider could not build, so this is the
;;    prediction the orchestrator's build/floor either confirms or refutes, not a second
;;    measurement) ───────────────────────────────────────────────────────────────────────────────
;;
;;   aggregate-new meta:      arity=-1 (Variadic) purity=:Pure determinism=:Deterministic totality=:Total
;;   kwargs-construct meta:   arity=-1 (Variadic) purity=:Pure determinism=:Deterministic totality=:Total
;;   write-forms meta:        arity=1            purity=:Pure determinism=:Deterministic totality=:Partial
;;   with-children meta:      arity=2            purity=:Pure determinism=:Deterministic totality=:Partial
;;   macro-error meta:        arity=1            purity=:Pure determinism=:Deterministic totality=:Partial
;;   verify::string (control) meta:               NONE (unchanged — DESIGN's "not a verb", untouched)
;;
;; ── BEHAVIOR, BEFORE THIS STONE (measured, pre-existing `target/release/wat`) ────────────────────
;;
;; All four non-aborting verbs still construct/round-trip exactly as today (no body moves, only the
;; dispatch route changes from a literal `runtime.rs` match arm to a registry lookup that now
;; resolves BEFORE the match is ever reached):
;;
;;   aggregate-new field a=3
;;   aggregate-new field b=4
;;   kwargs-construct field a=5
;;   kwargs-construct field b=6
;;   write-forms text=(1 2 3)
;;   with-children round-trip equal=true
;;
;; `macro-error` is the ONE of the five that can never produce a value — its body unconditionally
;; returns `Err`. Calling it unguarded inside `:user::main` would abort the WHOLE probe before any
;; later line printed (no general try/catch over `EvalBreak::Diagnostic` exists in wat), so — same
;; discipline wave 2 used for its `where`-fence panics — it is demonstrated OUT-OF-TREE, not
;; embedded in this committed, must-LOAD script:
;;
;;   $ cat > /tmp/probe_macro_error.wat <<'EOF'
;;   (:wat::core::defn :user::main [] -> :wat::core::nil
;;     (:wat::core::do
;;       (:wat::core::macro-error "boom")
;;       (:wat::kernel::println "unreachable")))
;;   EOF
;;   $ ./target/release/wat --check /tmp/probe_macro_error.wat   # exit 0 — well-typed, LOADS
;;   $ ./target/release/wat /tmp/probe_macro_error.wat           # exit 1 —
;;   [#wat.kernel.LociDiedError/RuntimeError ["#wat.runtime/MacroAbort {:message \"boom\" ...}"]]
;;
;; This is the empirical anchor for the ★ ruling below: `--check` passes (it "loads"); running it
;; raises a `RuntimeError`/`MacroAbort` that terminates the process with a nonzero exit — a
;; diagnostic surfacing to the user, never a value any wat code `match`es. Body unchanged this
;; stone, so this behavior is unaffected by the registration; the AFTER binary was not run (no
;; build permitted for a rider), so this is the pre-existing binary's measurement, unchanged by
;; the DESIGN's own "no body moves" constraint.
;;
;; ── ★ THE RULING THIS STONE EXISTS FOR: `macro-error` is Partial, not Total ─────────────────────
;;
;; `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`: totality asks whether a verb
;; produces a guaranteed, MATCHABLE outcome. `macro-error`'s body builds
;; `Err(EvalBreak::Diagnostic(Box::new(RuntimeError::new(.., RuntimeErrorKind::MacroAbort{..}))))`
;; — the DECISIVE line is which `EvalBreak` variant that is (`src/value/signal.rs`), not the word
;; "signal" this verb's own arc-258 doc comment happens to use informally:
;;
;;   `EvalBreak::Diagnostic` (signal.rs:70-72): "carries a source location and surfaces to user
;;   code as an error" — the SAME variant an ordinary TypeMismatch/ArityMismatch raise uses.
;;   `EvalBreak::Signal`    (signal.rs:78-81): "Caught at function boundaries; never surfaces to
;;   user code" — what `Option/try`/`Result/try` return, caught by `apply_function` and repackaged
;;   as the ENCLOSING function's own checker-guaranteed Option/Result — a real value the caller
;;   `match`es.
;;
;; `macro-error` builds a `Diagnostic`, never a `Signal`. It is caught nowhere at the wat-value
;; level — only by `macro_eval_pre_validated` (`src/macros/eval.rs`), which repackages it as a Rust
;; `MacroError`, a macro-EXPANSION-time (compile-time) failure, never a `Value` any wat code
;; receives or branches on. `try` and `macro-error` share a family resemblance (both informally
;; "propagate"/"abort") and land on OPPOSITE verdicts — exactly the trap the brief named: the body's
;; Rust TYPE decides, not the family. `Partial`.

(:wat::core::defrecord :probe::AggProbe [a <- :wat::core::i64 b <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    ;; behavior: aggregate-new / kwargs-construct still construct correctly
    (:wat::kernel::println (:wat::string::concat "aggregate-new field a=" (:wat::i64::to-string (:wat::core::Record/field-at (:wat::core::aggregate-new :probe::AggProbe 3 4) 0))))
    (:wat::kernel::println (:wat::string::concat "aggregate-new field b=" (:wat::i64::to-string (:wat::core::Record/field-at (:wat::core::aggregate-new :probe::AggProbe 3 4) 1))))
    (:wat::kernel::println (:wat::string::concat "kwargs-construct field a=" (:wat::i64::to-string (:wat::core::Record/field-at (:wat::core::kwargs-construct :probe::AggProbe :a 5 :b 6) 0))))
    (:wat::kernel::println (:wat::string::concat "kwargs-construct field b=" (:wat::i64::to-string (:wat::core::Record/field-at (:wat::core::kwargs-construct :probe::AggProbe :a 5 :b 6) 1))))
    ;; behavior: write-forms / with-children still round-trip
    (:wat::core::let
      [form         (:wat::core::quote (1 2 3))
       text         (:wat::core::write-forms form)
       kids         (:wat::core::ast->children form)
       rebuilt      (:wat::core::with-children form kids)
       rebuilt-text (:wat::core::write-forms rebuilt)]
      (:wat::kernel::println (:wat::string::concat "write-forms text=" text))
      (:wat::kernel::println (:wat::string::concat "with-children round-trip equal=" (:wat::core::bool::to-string (:wat::core::= text rebuilt-text)))))
    ;; metadata-of: the five, plus a control that is NOT part of this stone
    ;; (:wat::verify::string — DESIGN's "not a verb"; its guard stays untouched, so metadata-of
    ;; must read :None both before AND after, since it is never registered).
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::aggregate-new)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "aggregate-new meta: arity=" (:wat::edn::write (:wat::hashmap::get hm :arity)) " purity=" (:wat::edn::write (:wat::hashmap::get hm :purity)) " determinism=" (:wat::edn::write (:wat::hashmap::get hm :determinism)) " totality=" (:wat::edn::write (:wat::hashmap::get hm :totality)))))
      (:None (:wat::kernel::println "aggregate-new meta: NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::kwargs-construct)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "kwargs-construct meta: arity=" (:wat::edn::write (:wat::hashmap::get hm :arity)) " purity=" (:wat::edn::write (:wat::hashmap::get hm :purity)) " determinism=" (:wat::edn::write (:wat::hashmap::get hm :determinism)) " totality=" (:wat::edn::write (:wat::hashmap::get hm :totality)))))
      (:None (:wat::kernel::println "kwargs-construct meta: NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::write-forms)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "write-forms meta: arity=" (:wat::edn::write (:wat::hashmap::get hm :arity)) " purity=" (:wat::edn::write (:wat::hashmap::get hm :purity)) " determinism=" (:wat::edn::write (:wat::hashmap::get hm :determinism)) " totality=" (:wat::edn::write (:wat::hashmap::get hm :totality)))))
      (:None (:wat::kernel::println "write-forms meta: NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::with-children)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "with-children meta: arity=" (:wat::edn::write (:wat::hashmap::get hm :arity)) " purity=" (:wat::edn::write (:wat::hashmap::get hm :purity)) " determinism=" (:wat::edn::write (:wat::hashmap::get hm :determinism)) " totality=" (:wat::edn::write (:wat::hashmap::get hm :totality)))))
      (:None (:wat::kernel::println "with-children meta: NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::core::macro-error)
      ((:wat::core::Some hm) (:wat::kernel::println (:wat::string::concat "macro-error meta: arity=" (:wat::edn::write (:wat::hashmap::get hm :arity)) " purity=" (:wat::edn::write (:wat::hashmap::get hm :purity)) " determinism=" (:wat::edn::write (:wat::hashmap::get hm :determinism)) " totality=" (:wat::edn::write (:wat::hashmap::get hm :totality)))))
      (:None (:wat::kernel::println "macro-error meta: NONE")))
    (:wat::core::match (:wat::runtime::metadata-of :wat::verify::string)
      ((:wat::core::Some hm) (:wat::kernel::println "verify::string (control) meta: SOME -- unexpected, a finding"))
      (:None (:wat::kernel::println "verify::string (control) meta: NONE")))))
