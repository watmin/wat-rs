;; wat-scripts/perf/grid/where-accum-where-chain.wat — TWO `:where`s after an accumulate.
;;
;; Twin of where-accum-where-chain.clj. THE GAP THIS AXIS EXISTS TO HOLD, named precisely:
;;
;;   where-accum-where   accumulate + ONE `:where`            — already covered, agrees
;;   where-test-chain    Test -> Join -> Test -> Test         — already covered, agrees
;;   THIS AXIS          accumulate + `:where` -> `:where`     — the shape neither reaches
;;
;; Rows 1 and 2 differ by exactly ONE trailing, trivially-true `:where`. They MUST print the
;; same n=. Clara 0.24.0 does — a `:test` after a `:test` after an accumulator is an ordinary
;; chain there, and adding a tautology cannot subtract a match.
;;
;; Found by the rete differential fuzzer (`wat-tests/rete/differential-fuzz.wat`), family B,
;; 2026-08-25. Tracked in docs/arc/2026/06/278-rules-engine/RETE-FIX-LIST.md.
;;
;; WHY THE ONE-WHERE ROW IS IN THIS FILE AND NOT ONLY IN ITS OWN AXIS: the two rows share every
;; fact, every accumulator and the first predicate, so a diff between them isolates the trailing
;; `:where` and nothing else. A fix that "succeeds" by breaking row 1 fails visibly here, in the
;; same output, rather than in a different axis someone might not run.
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-accum-where-chain.wat
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-accum-where-chain.clj

(:wat::core::defn :wawc::row-count [] -> :wat::core::i64 2)

(:wat::core::defrecord :wawc::Station [loc <- :wat::core::String])
(:wat::core::defrecord :wawc::Reading [loc <- :wat::core::String v <- :wat::core::i64])
(:wat::core::defrecord :wawc::Busy    [loc <- :wat::core::String n <- :wat::core::i64])

;; ROW 1 — the AGREEING CONTROL: station, accumulate, ONE where.
(:wat::rete::defrule :wawc::one-where
  :when [(:wawc::Station (?loc <- :loc))
         (?n <- (:wat::rete::acc::count) :from (:wawc::Reading (?loc <- :loc)))
         (:wat::rete::where (:wat::rete::core::i64::>= ?n 2))]
  :then [(:wawc::Busy :loc ?loc :n ?n)])

;; ROW 2 — the same rule plus ONE trailing tautology. A `:where` that is true for every token
;; cannot remove a match, so this must derive exactly what row 1 derives.
(:wat::rete::defrule :wawc::two-wheres
  :when [(:wawc::Station (?loc <- :loc))
         (?n <- (:wat::rete::acc::count) :from (:wawc::Reading (?loc <- :loc)))
         (:wat::rete::where (:wat::rete::core::i64::>= ?n 2))
         (:wat::rete::where (:wat::rete::core::i64::> 1 0))]
  :then [(:wawc::Busy :loc ?loc :n ?n)])

(:wat::rete::defquery :wawc::q-Busy
  :params []
  :when [(?fact <- :wawc::Busy)])

(:wat::core::defn :wawc::sum-n [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64
                     p   <- :wat::core::PersistentMap]
      -> :wat::core::i64
      (:wat::core::let [f (:wat::core::Option/expect
                             (:wat::core::PersistentMap/get p "?fact")
                             "query: ?fact")]
        (:wat::core::i64::+ acc (:wawc::Busy/n f))))
    0
    (:wat::rete::query s (:wawc::q-Busy))))

(:wat::core::defn :wawc::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
      (:wat::core::String/concat
        (:wat::core::String/concat " " name)
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))))))

;; Three Readings at MCI, so the accumulate counts 3 and the `>= 2` predicate holds. Both rows
;; must report n=3. Native reports 3 and 0 — the trailing tautology erases the match.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [one  (:wat::rete::compile-all
            (:wat::core::PersistentVector (:wawc::one-where))
            (:wat::core::PersistentVector (:wawc::q-Busy)))
     two  (:wat::rete::compile-all
            (:wat::core::PersistentVector (:wawc::two-wheres))
            (:wat::core::PersistentVector (:wawc::q-Busy)))
     facts (:wat::core::fn [s <- :wat::rete::Session] -> :wat::rete::Session
             (:wat::rete::insert s
               (:wawc::Station :loc "MCI")
               (:wawc::Reading :loc "MCI" :v 1)
               (:wawc::Reading :loc "MCI" :v 2)
               (:wawc::Reading :loc "MCI" :v 3)))]
    (:wawc::line 1 "one-where"
      (:wawc::sum-n (:wat::core::match (:wat::rete::fire-rules (facts one)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))
    (:wawc::line 2 "two-wheres"
      (:wawc::sum-n (:wat::core::match (:wat::rete::fire-rules (facts two)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
