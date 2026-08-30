;; THE GUARDED COUNTER — refused, AND it terminates. This fixture exists to hold that pair.
;;
;; `N(k+1) :- N(k), (where (< ?k 500))` is the "hello world" of recursive Datalog-with-arithmetic
;; and it is the FIRST thing a user writes. It halts at k=500. The verifier refuses it anyway,
;; because its cyclicity test is purely structural — reachability over fact-type edges — and never
;; reads the `where` fence. That refusal is CORRECT by the verifier's own stated claim (it proves
;; the absence of ONE unbounded-derivation shape, not the presence of divergence), and the class is
;; named in `stratify.rs`'s "WHAT IT CANNOT SEE" block.
;;
;; ── WHY IT IS A FIXTURE AND NOT A COMMENT ────────────────────────────────────────────────────
;;
;; An earlier version of `probe_arc278_fixpoint_round_cap_deep.wat` WAS this shape. It was refused,
;; correctly, and rewritten into a range-restricted transitive closure — and the only record that
;; the class exists lived in that file's header, in prose, in a file nobody greps. Reported
;; 2026-08-28 by claude-compute; weighed against this tree and confirmed by driving.
;;
;; A comment cannot go red. This can. If someone later teaches the verifier to read the fence, THIS
;; FILE STOPS BEING REFUSED and its test fails — which is exactly the notification that the
;; narrowing closed, and the honest place to re-decide what the diagnostic should say.
;;
;; ⛔ It is NOT here to argue for an escape hatch. Two were already refused by builder ruling (a
;; `rune:` marker — "no magic comments"; and `Termination::Asserted [why <- String]` — "their
;; strings are their reason for themselves"). An author's string is not a proof.

(:wat::core::defrecord :gc::N [k <- :wat::core::i64])

(:wat::rete::defrule :gc::count-up
  :when
  [(:gc::N (?k <- :k))
   (:wat::rete::where (:wat::rete::core::i64::< ?k 500))]
  :then
  [(:gc::N :k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])

(:wat::rete::defquery :gc::q :params [] :when [(?fact <- :gc::N)])

;; No println before compile-all — the same lesson the fn-head fixture records: announcing
;; "compiled" first prints whether or not the compile then fails.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::core::match (:wat::rete::fire-rules
            (:wat::core::match (:wat::rete::insert
              (:wat::rete::compile-all (:wat::core::PersistentVector (:gc::count-up))
                (:wat::core::PersistentVector (:gc::q)))
              (:gc::N :k 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
          (:gc::q))))))
