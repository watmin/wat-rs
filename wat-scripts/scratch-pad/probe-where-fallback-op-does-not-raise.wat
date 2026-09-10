;; probe-where-fallback-op-does-not-raise.wat — regrounding runtime.rs:5371.
;;
;; THE QUESTION strike-no-rule-that-cannot-compile's OPEN QUESTION asks: does a `where`'s expr
;; still traverse `dispatch_keyword_head_value` (runtime.rs:5360-onward), the claim
;; `probe-stop-a-where-arith-path.wat` (deleted by this strike; its own header dated its result
;; 2026-08-02) once proved for the pre-totality-gate runtime? That probe is now VOID: it fences a
;; raw `:wat::core::i64::+`, which the now-armed totality axis refuses at compile
;; (`wat/rete/compile.wat:463`), so it never fires and proves nothing today.
;;
;; THE READING (verified against source, not asserted): `wat/rete/compile.wat`'s totality axis
;; forces a `where`'s arithmetic through the FALLBACK-carrying rete variant
;; (`:wat::rete::core::i64::+ a b :undefined fallback`, `src/rete/vocabulary.rs:326-339`,
;; `total: true`). At fire time this lowers to `Expr::CallFallback`
;; (`src/rete/expr_ir/mod.rs:758`) and executes at `src/rete/expr_ir/eval.rs:363-380`:
;; `apply_op` (self-contained dispatch table, `eval.rs:890-…`, `apply_core_kind`'s `OpExec::I64Add`
;; arm at `eval.rs:977`, using `a.checked_add(b)`) is called DIRECTLY — no call anywhere in
;; `expr_ir/eval.rs` or `expr_ir/mod.rs` reaches `runtime::dispatch_keyword_head_value` or
;; `runtime::dispatch_keyword_head` (grepped both files: the only `crate::runtime::eval_inner`
;; call in this module is `eval_lower`, the COMPILE-time `:wat::rete::lower` primitive, not the
;; fire-time exec path). A raised `IntegerOverflow` from that `checked_add` is caught by
;; `classify_fallback_outcome` right there in `eval.rs:369` and turned into `UseFallback` —
;; the fallback expr runs INSTEAD, silently. There is no path left by which an armed-total
;; `where` arithmetic op can raise to a user at all, which is why this probe cannot replicate
;; STOP-A's methodology (crash + read `:location`) — the crash it depended on is now
;; categorically impossible for a total op. The proof here is instead: force an overflow and show
;; the FALLBACK value is what actually flows to `:then`, in the rete-driven session — since a
;; classic `eval_inner`/`dispatch_keyword_head_value` walk over `:wat::core::i64::+` (the old
;; `:4829` inline arm this strike's citation names) has no `:undefined`/fallback vocabulary at
;; all and could not have produced this behavior.
;;
;; ⇒ THE CLAIM AT runtime.rs:5371 ("a `where` traverses `dispatch_keyword_head_value`") IS FALSE
;;   TODAY. It was true when written (2026-08-02, commit 9b57ded784) — `where` execution was the
;;   plain AST interpreter then. `30725034f` (2026-08-17, "compile `where` — one Expr DAG, stash
;;   the Program") moved `where` execution onto the compiled `expr_ir::Program` path, and that
;;   path never calls back into `runtime.rs`'s giant keyword match for op execution. The two
;;   sites the ORIGINAL probe distinguished (`:4829` inline, `:9753` substrate-apply) are both
;;   still real, but NEITHER is where a `where` op executes now — a third site
;;   (`expr_ir::apply_core_kind`) is, and it was not one of the two candidates STOP-A considered
;;   because it did not exist yet.
;;
;; RUN: ./target/release/wat wat-scripts/scratch-pad/probe-where-fallback-op-does-not-raise.wat
;; EXPECT: "before-fire", then the query returns exactly one :pfo::Hit with k=2 and sum=-999 (the
;; FALLBACK value, not a crash, not the true overflowed sum) — proof the fallback path fired.

(:wat::core::defrecord :pfo::Big [k <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :pfo::Hit [k <- :wat::core::i64  sum <- :wat::core::i64])

;; ?n is i64::MAX for k=2; (?n + 1) overflows. The TOTAL fence variant must not raise — it must
;; substitute :undefined's fallback (-999) and let the rule fire normally.
(:wat::rete::defrule :pfo::add-in-where
  :when
  [(:pfo::Big (?k <- :k) (?n <- :n))
   (:wat::rete::where (:wat::rete::core::i64::> (:wat::rete::core::i64::+ ?n 1 :undefined -999) -1000000))]
  :then
  [(:pfo::Hit ?k (:wat::rete::core::i64::+ ?n 1 :undefined -999))])

(:wat::rete::defquery :pfo::q-Hit
  :params []
  :when [(?fact <- :pfo::Hit)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [session (:wat::core::match (:wat::rete::insert-all
               (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector (:pfo::add-in-where)) (:wat::core::PersistentVector (:pfo::q-Hit))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
               (:wat::core::PersistentVector
                 (:pfo::Big :k 1 :n 1)
                 (:pfo::Big :k 2 :n 9223372036854775807))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     _       (:wat::kernel::println "before-fire")
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     _       (:wat::kernel::println "after-fire")
     hits    (:wat::rete::query fired (:pfo::q-Hit))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::nil  h <- :wat::core::PersistentMap] -> :wat::core::nil
        (:wat::kernel::println h))
      nil
      hits)))
