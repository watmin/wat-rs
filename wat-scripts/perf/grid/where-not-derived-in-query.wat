;; wat-scripts/perf/grid/where-not-derived-in-query.wat — `:not` over a DERIVED class, in a QUERY.
;;
;; Twin of where-not-derived-in-query.clj. THE SHAPE NO OTHER AXIS REACHED, named precisely:
;;
;;   where-query-compat   query-side `:not`, over an INSERTED class    — covered, agrees
;;   strat-neg/negation   `:not` over a DERIVED class, in a RULE       — covered, agrees
;;   THIS AXIS            query-side `:not`, over a DERIVED class      — the crossing of the two
;;
;; Found by the rete differential fuzzer (`wat-tests/rete/differential-fuzz.wat`), family C,
;; 2026-08-25: 54 divergences, ALL at depth >= 1 and never at depth 0. Tracked in
;; docs/arc/2026/06/278-rules-engine/RETE-FIX-LIST.md.
;;
;; WHY IT WAS WRONG, AND WHY THAT IS A QUERY PROBLEM RATHER THAN A NEGATION PROBLEM. A
;; constrained query is harvested from the fixpoint's ACCUMULATED beta, which by the semi-naive
;; contract is never cleared. The negation propagated its token in round 0, when S2 did not yet
;; exist; a later round derived S2 and nothing retracted the token already in beta. The query
;; passed while the engine's own `q-S2` confirmed the fact was present — which is exactly what
;; row 5 is here to prove.
;;
;; ⚠ THE ONE ASYMMETRY WITH THE TWIN, STATED RATHER THAN HIDDEN. wat's own C fixture writes the
;; negation as `(:wat::rete::not (:user::S2 (?s <- :k)))` — a variable bound INSIDE the negation
;; and used nowhere. Clara 0.24.0 REFUSES that form at compile time ("Using variable that is not
;; previously bound ... variables used in negations are not bound for subsequent rules"), while
;; wat compiles it silently. Both sides here negate BARE so the rows stay comparable; the
;; acceptance divergence is its own RETE-FIX-LIST entry, not this axis's subject.
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-not-derived-in-query.wat
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-not-derived-in-query.clj

(:wat::core::defrecord :wndq::S1  [k <- :wat::core::i64])
(:wat::core::defrecord :wndq::S2  [k <- :wat::core::i64])
(:wat::core::defrecord :wndq::S3  [k <- :wat::core::i64])
(:wat::core::defrecord :wndq::Hit [k <- :wat::core::i64])

;; S2 and S3 exist ONLY by derivation. Nothing inserts either.
(:wat::rete::defrule :wndq::r1 :when [(:wndq::S1 (?k <- :k))] :then [(:wndq::S2 :k ?k)])
(:wat::rete::defrule :wndq::r2 :when [(:wndq::S2 (?k <- :k))] :then [(:wndq::S3 :k ?k)])

;; The same negation in a RULE — the contrast that answers "is a query stratified the way a rule
;; is?" in the output itself rather than in a comment.
(:wat::rete::defrule :wndq::r-not-S2
  :when [(:wat::rete::not (:wndq::S2))]
  :then [(:wndq::Hit :k 1)])

(:wat::rete::defquery :wndq::q-not-S2 :params [] :when [(:wat::rete::not (:wndq::S2))])
(:wat::rete::defquery :wndq::q-not-S3 :params [] :when [(:wat::rete::not (:wndq::S3))])
(:wat::rete::defquery :wndq::q-Hit    :params [] :when [(?fact <- :wndq::Hit)])
(:wat::rete::defquery :wndq::q-S2     :params [] :when [(?fact <- :wndq::S2)])

(:wat::core::defn :wndq::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :wndq::run
  [rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   q     <- :wat::rete::Query]
  -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query
      (:wat::core::match (:wat::rete::fire-rules
        (:wat::core::match (:wat::rete::insert
          (:wat::rete::compile-all rules
            (:wat::core::PersistentVector
              (:wndq::q-not-S2) (:wndq::q-not-S3) (:wndq::q-Hit) (:wndq::q-S2)))
          (:wndq::S1 :k 1)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
      q)))

;; Row 1 is the agreeing control and is here, not in a separate file, so a "fix" that achieves
;; agreement by breaking the ABSENT case fails visibly in the same output.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [none  (:wat::core::PersistentVector)
     d1    (:wat::core::PersistentVector (:wndq::r1))
     d2    (:wat::core::PersistentVector (:wndq::r1) (:wndq::r2))
     ruled (:wat::core::PersistentVector (:wndq::r1) (:wndq::r-not-S2))]
    (:wndq::line 1 "query-no-chain" (:wndq::run none  (:wndq::q-not-S2)))
    (:wndq::line 2 "query-chain-d1" (:wndq::run d1    (:wndq::q-not-S2)))
    (:wndq::line 3 "query-chain-d2" (:wndq::run d2    (:wndq::q-not-S3)))
    (:wndq::line 4 "rule-chain"     (:wndq::run ruled (:wndq::q-Hit)))
    (:wndq::line 5 "control-S2"     (:wndq::run d1    (:wndq::q-S2)))))
