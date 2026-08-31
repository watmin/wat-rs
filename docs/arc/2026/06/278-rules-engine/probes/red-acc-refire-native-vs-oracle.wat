;; red-acc-refire-native-vs-oracle.wat — THE ORACLE IS WRONG. Clara settled it.
;;
;; Driven 2026-08-31 at HEAD 2733b9bd9. Prints [Out, Tally, Stale(n=0)] for native then oracle:
;;
;;     native  [1 1 0]        oracle  [1 2 1]
;;
;; `C` is derived in round 1, `Out` from `C` in round 2, so the accumulate's count goes 0 -> 1
;; across the fixpoint. The ORACLE emits a Tally for EACH value and KEEPS BOTH — the `Stale`
;; column is a rule matching `Tally(n = 0)`, and the oracle has one. **A fact asserting the count
;; is zero, left standing while the count is one.** Native emits only the final Tally.
;;
;; ⛔⛔ THE EXTERNAL REFERENCE SETTLES IT — the builder's call, and it inverted the assumption.
;; The same rule set in Clara 0.24.0 (`red-acc-refire-clara-reference.clj`, run exactly as the
;; grid runs it: `clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}'`):
;;
;;     clara [Out Tally] = [1 1]        clara Tally values = [1]
;;
;; **Clara agrees with NATIVE.** It does truth maintenance: when an accumulator's result changes,
;; the fact derived from the old result is RETRACTED. The oracle does not, so it accumulates
;; intermediate fixpoint states as permanent facts.
;;
;; ⚠ THE ORACLE IS THIS ARC'S DIFFERENTIAL REFERENCE. Every `fire-rules$oracle` comparison over a
;; shape where an accumulate's count CHANGES mid-fixpoint is measuring against a reference that
;; over-emits. That is the finding worth carrying, not the two-line diff.
;;
;; ⚠ AND THE TREE'S ONLY DIFFERENTIAL FOR THIS SHAPE CANNOT SEE IT.
;; `probe_arc278_derived_exists_acc.wat`'s tally rule carries `(where (i64::= ?n 1))`, which
;; filters the intermediate emission out. Proven by adding that same fence here:
;;
;;     fenced:    native [1 1]   oracle [1 1]   AGREE
;;     unfenced:  native [1 1]   oracle [1 2]   DISAGREE
;;
;; So `derived_exists_and_acc_spec_matches_native` passes because of its fixture's shape, not
;; because the engines agree.
;;
;; ⚠ THIS IS NOT CLASS D2, which predicts a double-index on `filter -> HashJoin(a) -> HashJoin(b)`.
;; Removing the join chain entirely leaves this divergence unchanged, so the chain is not
;; implicated. **D2 REMAINS UNTESTED, NOT DISPROVEN** — a fixture provably carrying two chained
;; HashJoins is still owed.
;;
;; rune:lint(red-by-design) — this file exists to PRINT two disagreeing lines; it loads and runs
;;    cleanly, and a reader comparing the two printed vectors is meant to find them different.

(:wat::core::defrecord :d2::A    [x <- :wat::core::i64])
(:wat::core::defrecord :d2::B    [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :d2::Seed [y <- :wat::core::i64])
(:wat::core::defrecord :d2::C    [y <- :wat::core::i64])
(:wat::core::defrecord :d2::Out  [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :d2::Tally [n <- :wat::core::i64])
(:wat::core::defrecord :d2::Stale [n <- :wat::core::i64])

;; round 1 derives C — so the LAST join's right side is empty during round 1
(:wat::rete::defrule :d2::mk-c
  :when [(:d2::Seed (?y <- :y))]
  :then [(:d2::C :y ?y)])

;; filter -> join a -> join b
(:wat::rete::defrule :d2::chain
  :when [(:d2::C (?y <- :y))]
  :then [(:d2::Out :x 1 :y ?y)])

;; an accumulate over the derived Out — a COUNT sees doubled tokens where dedup hides doubled facts
(:wat::rete::defrule :d2::tally
  :when [(:d2::Seed (?y <- :y))
         (?n <- (:wat::rete::acc::count) :from (:d2::Out))]
  :then [(:d2::Tally :n ?n)])

(:wat::rete::defrule :d2::stale
  :when [(:d2::Tally (?n <- :n))
         (:wat::rete::where (:wat::rete::core::i64::= ?n 0))]
  :then [(:d2::Stale :n ?n)])

(:wat::rete::defquery :d2::q-stale :params [] :when [(?f <- :d2::Stale)])
(:wat::rete::defquery :d2::q-out   :params [] :when [(?f <- :d2::Out)])
(:wat::rete::defquery :d2::q-tally :params [] :when [(?f <- :d2::Tally)])

(:wat::core::defn :d2::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s
      (:d2::A :x 1) (:d2::B :x 1 :y 7) (:d2::Seed :y 7))
    ((:wat::rete::InsertOutcome::Inserted __st) __st)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c)
      (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None))))

(:wat::core::defn :d2::compile [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :d2)
      (:wat::core::PersistentVector (:d2::q-out) (:d2::q-tally) (:d2::q-stale)))
    ((:wat::rete::CompileOutcome::Compiled __s) __s)
    ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
      (:wat::kernel::assertion-failed! "terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :d2::counts [fired <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:wat::core::length (:wat::rete::query fired (:d2::q-out)))
    (:wat::core::length (:wat::rete::query fired (:d2::q-tally)))
    (:wat::core::length (:wat::rete::query fired (:d2::q-stale)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:d2::counts
      (:wat::core::match (:wat::rete::fire-rules (:d2::seed (:d2::compile)))
        ((:wat::rete::FireOutcome::Fired __f) __f)
        ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r) (:wat::kernel::assertion-failed! "mc" :wat::core::None :wat::core::None))
        ((:wat::rete::FireOutcome::RoundCapExceeded __c __s) (:wat::kernel::assertion-failed! "rc" :wat::core::None :wat::core::None)))))
    (:wat::kernel::println (:d2::counts
      (:wat::core::match (:wat::rete::fire-rules$oracle (:d2::seed (:d2::compile)))
        ((:wat::rete::FireOutcome::Fired __f) __f)
        ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r) (:wat::kernel::assertion-failed! "mc" :wat::core::None :wat::core::None))
        ((:wat::rete::FireOutcome::RoundCapExceeded __c __s) (:wat::kernel::assertion-failed! "rc" :wat::core::None :wat::core::None)))))))
