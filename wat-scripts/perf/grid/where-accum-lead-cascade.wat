;; wat-scripts/perf/grid/where-accum-lead-cascade.wat — a leading accumulate in a QUERY, under a
;; cascade.
;;
;; Twin of where-accum-lead-cascade.clj. THE SHAPE NO OTHER AXIS REACHED, named precisely:
;;
;;   where-accum-lead   leading accumulate, in a RULE, no cascade   — covered, agrees
;;   leading-exists     leading `:exists`, in a RULE, WITH a cascade — covered, agrees
;;   THIS AXIS          leading accumulate, in a QUERY, WITH a cascade
;;
;; Found by the rete differential fuzzer (`wat-tests/rete/differential-fuzz.wat`), family A,
;; 2026-08-25: the row count tracked the FIXPOINT ROUND COUNT exactly — a 1-rule inert chain gave
;; 2 rows and a 2-rule chain gave 3, where both references give 1. Tracked in
;; docs/arc/2026/06/278-rules-engine/RETE-FIX-LIST.md.
;;
;; THE CASCADE IS INERT AND THAT IS THE WHOLE DESIGN. S1 -> S2 -> S3 derives facts the query never
;; reads; its only job is to make the fixpoint iterate more rounds. So rows 1-3 differ in NOTHING
;; the query can see, and any spread between them is the engine leaking its own round count into
;; an answer. This is the same instrument `leading-exists` uses for the `:exists` form of the
;; defect ("the row count must be independent of the cascade") — one axis per non-monotonic
;; leading condition, because a fix for one did not reach the other: the leading `:not`/`:exists`
;; fix (2026-08-24, `71d0e700e`) left the accumulate form live for a further two days.
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-accum-lead-cascade.wat
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-accum-lead-cascade.clj

(:wat::core::defrecord :walc::W  [k <- :wat::core::i64])
(:wat::core::defrecord :walc::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :walc::S2 [k <- :wat::core::i64])
(:wat::core::defrecord :walc::S3 [k <- :wat::core::i64])

;; The inert chain. Neither rule reads or writes W.
(:wat::rete::defrule :walc::r1 :when [(:walc::S1 (?k <- :k))] :then [(:walc::S2 :k ?k)])
(:wat::rete::defrule :walc::r2 :when [(:walc::S2 (?k <- :k))] :then [(:walc::S3 :k ?k)])

(:wat::rete::defquery :walc::q-lead :params []
  :when [(?n <- (:wat::rete::acc::count) :from (:walc::W))
         (:wat::rete::where (:wat::rete::core::i64::>= ?n 2))])

(:wat::rete::defquery :walc::q-W :params [] :when [(?fact <- :walc::W)])

(:wat::core::defn :walc::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

(:wat::core::defn :walc::run
  [rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])
   q     <- :wat::rete::Query]
  -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query
      (:wat::rete::fire-rules
        (:wat::rete::insert
          (:wat::rete::compile-all rules
            (:wat::core::PersistentVector (:walc::q-lead) (:walc::q-W)))
          (:walc::W :k 7) (:walc::W :k 7) (:walc::S1 :k 1)))
      q)))

;; Two W facts, so the count is 2 and the `>= 2` predicate holds in every row. Rows 1-3 must all
;; print n=1; row 4 proves the accumulate still had its facts, so a "fix" that empties the world
;; cannot read as agreement.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [none (:wat::core::PersistentVector)
     d1   (:wat::core::PersistentVector (:walc::r1))
     d2   (:wat::core::PersistentVector (:walc::r1) (:walc::r2))]
    (:walc::line 1 "no-cascade"  (:walc::run none (:walc::q-lead)))
    (:walc::line 2 "cascade-d1"  (:walc::run d1   (:walc::q-lead)))
    (:walc::line 3 "cascade-d2"  (:walc::run d2   (:walc::q-lead)))
    (:walc::line 4 "control-W"   (:walc::run d2   (:walc::q-W)))))
