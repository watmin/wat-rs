;; BRIEF-cond-the-first-macro-backed-rete-row.md scorecard row 6 — does the RETE-spelled cond
;; compose in a real `defrule`'s `where`, against a bound field, with `fire-rules` selecting
;; correctly? This is a SEPARATE mechanism from ordinary macro-expanded code
;; (probe-cond-rete-scorecard.wat's row 2): `defrule` quotes `:when`/`:then` verbatim
;; (wat/rete.wat:2231) and `eval_test_core` (src/rete/matcher.rs) evaluates that raw,
;; NEVER-macro-expanded AST via `runtime::eval_inner` directly.
;;
;; POSITIVE CONTROL as of arc 278 task #78 (BRIEF-where-bodies-expand-at-compile-time.md). This
;; was a GAP-WITNESS: before task #78, it raised `#wat.runtime/UnknownFunction {:message "unknown
;; function: :wat::rete::core::cond"}` at fire time, because a `where` body was never
;; macro-expanded — `expand_form` returned a `make-rule` call's quoted `:when` vector untouched
;; (the quote-family "carry DATA, not code" check), so the rete-spelled `cond` inside this
;; `where` never got its chance to expand into `:wat::rete::core::if`. Task #78 taught
;; `expand_form` (via `resolve::boundary::Boundary::MakeRule`) to expand ONLY the body of a
;; `(:wat::rete::where …)` form, leaving the surrounding `:probe::Item` fact pattern untouched.
;; Now prints `hits=1` — cond composes correctly in a real `defrule`'s `where`.
(:wat::core::defrecord :probe::Item [tier <- :wat::core::keyword])
(:wat::core::defrecord :probe::Hit [tier <- :wat::core::keyword])

(:wat::rete::defrule :probe::score-rule
  :when
  [(:probe::Item (?tier <- :tier))
   (:wat::rete::where
     (:wat::rete::f64::>
       (:wat::rete::core::cond
         ((:wat::rete::core::keyword::= ?tier :gold)   0.5)
         ((:wat::rete::core::keyword::= ?tier :silver) 0.7)
         (:else                                        0.9))
       0.6))]
  :then
  [(:probe::Hit :tier ?tier)])

(:wat::rete::defquery :probe::q-Hit
  :params []
  :when [(?fact <- :probe::Hit)])


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules  (:wat::core::PersistentVector (:probe::score-rule))
     staged (:wat::rete::insert (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q-Hit))) (:probe::Item :tier :silver))
     fired  (:wat::rete::fire-rules staged)
     hits   (:wat::rete::query fired (:probe::q-Hit))]
    (:wat::kernel::println (:wat::string::concat "hits=" (:wat::core::str (:wat::core::length hits))))))
