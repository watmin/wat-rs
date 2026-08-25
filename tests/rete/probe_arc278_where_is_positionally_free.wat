;; Fixture BESIDE probe_arc278_where_is_positionally_free.rs.
;;
;; THE CONTRACT: a `(:wat::rete::where …)` guard is POSITIONALLY FREE. A rule means
;; the same thing wherever the guard sits, once its variables are bound.
;;
;; WHY THIS EXISTS — a SILENT WRONG ANSWER, filed 2026-08-24
;; (NOTE-rete-a-where-before-a-fact-condition-silently-matches-nothing.md). A guard
;; followed by TWO OR MORE fact conditions made the rule match NOTHING. It compiled,
;; ran, exited 0, and returned an empty result set. For a search tool that is
;; indistinguishable from "nothing matched" — the filer only caught it because a
;; positive-control fixture with a deliberate duplicate ALSO returned 0.
;;
;; BOTH REFERENCES DISAGREED WITH NATIVE, measured: the wat `$oracle` and Clara 0.24.0
;; each matched correctly with the guard mid-chain. So this was never a question of
;; whether `where`s must be trailing — the engine's own definition of correct says
;; they need not be, and refusing them would have made the fence contradict the oracle.
;;
;; THE TRIGGER IS EXACTLY "two or more FACT conditions after the guard", not the
;; guard's absolute position: a guard at slot 3 of 5 failed while the same guard at
;; slot 4 of 5 worked. Not the join shape either — the report's unjoined `Source`
;; condition was a red herring; a fully-joined third fact failed identically.
;;
;; Mechanism: `filter_after_join` walks the frontier through FILTER children only, so
;; `:where → HashJoin(a) → HashJoin(b)` stalled at (a). See `left_activate_join`.
;;
;; ROWS. `one-after` and `trailing` are the shapes that ALREADY worked — they are here
;; so a regression on the working path is visible in the same place. `two-after` is the
;; reported bug. `four-after` is depth: the frontier must loop, not just take one extra
;; step, or a fix that special-cases two would pass while three still silently failed.

(:wat::core::defrecord :wpf::A   [id <- :wat::core::i64  k <- :wat::core::String])
(:wat::core::defrecord :wpf::B   [id <- :wat::core::i64])
(:wat::core::defrecord :wpf::C   [id <- :wat::core::i64])
(:wat::core::defrecord :wpf::D   [id <- :wat::core::i64])
(:wat::core::defrecord :wpf::E   [id <- :wat::core::i64])
(:wat::core::defrecord :wpf::One   [x <- :wat::core::i64])
(:wat::core::defrecord :wpf::Two   [x <- :wat::core::i64])
(:wat::core::defrecord :wpf::Four  [x <- :wat::core::i64])
(:wat::core::defrecord :wpf::Trail [x <- :wat::core::i64])

(:wat::rete::defrule :wpf::r-one
  :when [(:wpf::A (?id <- :id) (?k <- :k))
         (:wat::rete::where (:wat::rete::core::string::= ?k "yes"))
         (:wpf::B (?id <- :id))]
  :then [(:wpf::One :x ?id)])

(:wat::rete::defrule :wpf::r-two
  :when [(:wpf::A (?id <- :id) (?k <- :k))
         (:wat::rete::where (:wat::rete::core::string::= ?k "yes"))
         (:wpf::B (?id <- :id)) (:wpf::C (?id <- :id))]
  :then [(:wpf::Two :x ?id)])

(:wat::rete::defrule :wpf::r-four
  :when [(:wpf::A (?id <- :id) (?k <- :k))
         (:wat::rete::where (:wat::rete::core::string::= ?k "yes"))
         (:wpf::B (?id <- :id)) (:wpf::C (?id <- :id))
         (:wpf::D (?id <- :id)) (:wpf::E (?id <- :id))]
  :then [(:wpf::Four :x ?id)])

(:wat::rete::defrule :wpf::r-trail
  :when [(:wpf::A (?id <- :id) (?k <- :k))
         (:wpf::B (?id <- :id)) (:wpf::C (?id <- :id))
         (:wat::rete::where (:wat::rete::core::string::= ?k "yes"))]
  :then [(:wpf::Trail :x ?id)])

(:wat::rete::defquery :wpf::q1 :params [] :when [(?fact <- :wpf::One)])
(:wat::rete::defquery :wpf::q2 :params [] :when [(?fact <- :wpf::Two)])
(:wat::rete::defquery :wpf::q4 :params [] :when [(?fact <- :wpf::Four)])
(:wat::rete::defquery :wpf::qt :params [] :when [(?fact <- :wpf::Trail)])

(:wat::core::defn :wpf::staged [] -> :wat::rete::Session
  (:wat::rete::insert-all
    (:wat::rete::insert-all
      (:wat::rete::insert-all
        (:wat::rete::insert-all
          (:wat::rete::insert-all
            (:wat::rete::compile-all (:wat::rete::collect-rules :wpf)
              (:wat::core::PersistentVector (:wpf::q1) (:wpf::q2) (:wpf::q4) (:wpf::qt)))
            (:wat::core::PersistentVector (:wpf::A :id 1 :k "yes") (:wpf::A :id 2 :k "no")))
          (:wat::core::PersistentVector (:wpf::B :id 1) (:wpf::B :id 2)))
        (:wat::core::PersistentVector (:wpf::C :id 1) (:wpf::C :id 2)))
      (:wat::core::PersistentVector (:wpf::D :id 1) (:wpf::D :id 2)))
    (:wat::core::PersistentVector (:wpf::E :id 1) (:wpf::E :id 2))))

(:wat::core::defn :wpf::counts [s <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:wat::core::length (:wat::rete::query s (:wpf::q1)))
    (:wat::core::length (:wat::rete::query s (:wpf::q2)))
    (:wat::core::length (:wat::rete::query s (:wpf::q4)))
    (:wat::core::length (:wat::rete::query s (:wpf::qt)))))

;; [one, two, four, trailing] x [native, oracle] — every slot must be 1.
(:wat::core::defn :user::native-and-oracle [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::mapv
    (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
    (:wat::core::PersistentVector/concat
      (:wpf::counts (:wat::rete::fire-rules (:wpf::staged)))
      (:wpf::counts (:wat::rete::fire-rules$oracle (:wpf::staged))))))
