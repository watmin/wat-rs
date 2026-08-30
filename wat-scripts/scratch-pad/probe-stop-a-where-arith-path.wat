;; probe-stop-a-where-arith-path.wat — STOP-A, the DISCONFIRMING probe.
;;
;; THE QUESTION (DESIGN-STONE-where-admits-only-rete-ops.md, "⛔ STOP-A"): TWO sites in
;; `runtime.rs` raise `IntegerOverflow`, and the rete surface must hang on whichever one a
;; `where` expression actually traverses. A grep cannot tell a duplicate from a fast path.
;;
;;   runtime.rs:4829  `dispatch_keyword_head` -> `eval_i64_arith(head, args, list_span, …)`
;;                    the INLINE arm. It has the AST, so it threads the OPERAND's span
;;                    (`b_span`) into the error.
;;   runtime.rs:9753  `dispatch_substrate_impl` -> `arith_i64_i64_inner(impl_name, vals, …)`
;;                    the PRE-EVALUATED substrate table (reached from `runtime.rs:8925`, the
;;                    `apply` path; `purity.rs:1153` names it "the apply-reachable substrate
;;                    table"). It has NO AST, so every arm uses `rust_caller_span!()`.
;;
;; ⇒ THE TWO PATHS ARE DISTINGUISHABLE BY THE ERROR'S `:location`, and by nothing else — both
;;   raise the same `RuntimeErrorKind::IntegerOverflow` with the same op/a/b fields.
;;
;;     `:location` naming THIS .wat file, at the overflowing operand  ⇒ the :4829 INLINE path
;;     `:location` naming a Rust source file (src/runtime.rs)          ⇒ the :9753 SUBSTRATE path
;;
;; This is the "confirm with a probe first" the seam demanded: the call chain was traced by
;; READING (`matcher.rs:1139` eval_test_core -> `runtime::eval_inner` -> `eval_list:4330` ->
;; `dispatch_keyword_head:4390` -> `:4829`), and a read is a hypothesis until a run agrees.
;;
;; ── HOW IT RUNS (it is DESIGNED TO CRASH — that is the measurement) ──────────────────────────
;;
;;     ./target/release/wat wat-scripts/scratch-pad/probe-stop-a-where-arith-path.wat
;;
;; Expect: "before-fire" on stdout, then a non-zero exit with the raise on stderr. Read the
;; `:location` out of the `#wat.runtime/IntegerOverflow` in that cascade. "after-fire" must
;; NEVER print — a `where` that raises unwinds the whole `fire-rules` call (already established
;; by `wat-scripts/perf/grid/where-numeric.wat`'s row 11; not re-derived here).
;;
;; NON-VACUITY: `before-fire` printing and `after-fire` NOT printing is the control. If BOTH
;; print, the predicate never overflowed and the probe measured nothing — do not read a
;; location off a run that did not raise.
;;
;; ── RESULT, RUN 2026-08-02 — STOP-A IS CLOSED, AND THE READ WAS RIGHT ────────────────────────
;;
;;     "before-fire"                                        <- the control fired
;;     #wat.runtime/IntegerOverflow
;;       {:message "i64 overflow: 9223372036854775807 :wat::core::i64::+ 1 does not fit in 64 bits"
;;        :location #wat.core/Span {:file "…/probe-stop-a-where-arith-path.wat"
;;                                  :line <the `where` line> :col 66 :end {… :col 67}}
;;        :op ":wat::core::i64::+" :a 9223372036854775807 :b 1}
;;     (no "after-fire")                                    <- the raise unwound the whole fire
;;
;; THE LOAD-BEARING FACT IS THE FILE, NOT THE NUMBER: the span names **this .wat file**, at
;; col 66-67 of the `where` line — the literal `1`, the SECOND OPERAND of the `i64::+`. (The
;; line number drifts if this header is edited; the column tracks the operand. Re-run to see
;; the current pair.) That is `b_span`, which only the INLINE arm has. `arith_i64_i64_inner`
;; carries no AST and would have reported a `rust_caller_span!()` naming `src/runtime.rs`.
;;
;; ⇒ A `where` TRAVERSES runtime.rs:4829 (the inline `eval_i64_arith` arm), NOT :9753.
;;
;; THE CONSEQUENCE FOR #55 (S3b+S4): the clean `arith_i64_i64_inner` / `I64ArithErr` factoring
;; the design stone points at lives on the OTHER path. `:wat::rete::core::i64::+` therefore cannot
;; simply substitute a second terminal handler at `:9753` and expect a `where` to reach it —
;; `:4829` must be refactored onto that shared kernel FIRST, or the rete surface works only
;; through `apply`. The "shared kernel, two surfaces" law is intact; the kernel just is not
;; yet shared by the path that matters.

(:wat::core::defrecord :stopa::Big [k <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :stopa::Hit [k <- :wat::core::i64])

;; The overflow is on a BOUND VARIABLE, inside a `where`, so it can only be reached through the
;; rete test path — not through a literal the checker could fold. `n` is i64::MAX for the second
;; fact, so `(+ ?n 1)` overflows exactly once the TestNode reaches it.
(:wat::rete::defrule :stopa::overflow-in-where
  :when
  [(:stopa::Big (?k <- :k) (?n <- :n))
   (:wat::rete::where (:wat::rete::core::i64::> (:wat::core::i64::+ ?n 1) 0))]
  :then
  [(:stopa::Hit ?k)])

(:wat::rete::defquery :stopa::q-Hit
  :params []
  :when [(?fact <- :stopa::Hit)])


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [session (:wat::core::match (:wat::rete::insert-all
               (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector (:stopa::overflow-in-where)) (:wat::core::PersistentVector (:stopa::q-Hit))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
               (:wat::core::PersistentVector
                 (:stopa::Big :k 1 :n 1)
                 (:stopa::Big :k 2 :n 9223372036854775807))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     _       (:wat::kernel::println "before-fire")
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     _       (:wat::kernel::println "after-fire")]
    (:wat::kernel::println
      (:wat::core::i64::to-string
        (:wat::core::PersistentVector/length
          (:wat::rete::query fired (:stopa::q-Hit)))))))
