;; BRIEF-cond-the-first-macro-backed-rete-row.md scorecard row 6 — does the RETE-spelled cond
;; compose in a real `defrule`'s `where`, against a bound field, with `fire-rules` selecting
;; correctly? This is a SEPARATE mechanism from ordinary macro-expanded code
;; (probe-cond-rete-scorecard.wat's row 2): `defrule` quotes `:when`/`:then` verbatim
;; (wat/rete.wat:2231) and `eval_test_core` (src/rete/matcher.rs) evaluates that raw,
;; NEVER-macro-expanded AST via `runtime::eval_inner` directly.
(:wat::core::defrecord :probe::Item [tier <- :wat::core::keyword])
(:wat::core::defrecord :probe::Hit [tier <- :wat::core::keyword])

(:wat::rete::defrule :probe::score-rule
  :when
  [(:probe::Item (?tier <- :tier))
   (:wat::rete::where
     (:wat::core::f64::>
       (:wat::rete::core::cond
         ((:wat::rete::core::keyword::= ?tier :gold)   0.5)
         ((:wat::rete::core::keyword::= ?tier :silver) 0.7)
         (:else                                        0.9))
       0.6))]
  :then
  [(:probe::Hit :tier ?tier)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules  (:wat::core::PersistentVector (:probe::score-rule))
     staged (:wat::rete::insert (:wat::rete::compile rules) (:probe::Item :tier :silver))
     fired  (:wat::rete::fire-rules staged)
     hits   (:wat::rete::query fired :probe::Hit)]
    (:wat::kernel::println (:wat::core::string::concat "hits=" (:wat::core::str (:wat::core::length hits))))))
