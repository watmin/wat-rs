;; `cond` layout. A NEW FILE and nothing else. Arc 277.
;;
;; Builder, 2026-09-06: one clause per line; the body rides its test; bodies
;; ALIGN across the clauses of one cond. `cond` has NO subject — clauses only,
;; terminal `(:else body)` required. A rule that binds a term after the head
;; is a rule for `match`, which already has one.
;;
;; AlignPairs cannot do this. It pads a broken child so the NEXT sibling rides,
;; and every clause is broken, so the pad never fires. TableRow is the existing
;; fact that shares first-token width across sibling lists; the emitter already
;; withholds the pad when the aligned line does not fit. This file names no
;; budget. Claim each clause so an unaligned fallback is still one line
;; (row 13: `:else` and a call-test take the same shape).

(:wat::rete::defrule :fmt::cond-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::cond"))]
  :then [(:wat::fmt::Claim :form ?p)])

;; one CLAUSE per line. Child 0 is the head; every child from index 1 on
;; is a clause.
(:wat::rete::defrule :fmt::cond-clause-per-line
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::cond"))
         (:wat::grep::Node  (?cl <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))]
  :then [(:wat::fmt::Break :id ?cl :kind (:wat::fmt::BreakKind::Block))])

;; Claim the clause (body rides; R11 does not explode a call-test) and
;; join every clause into one table group so the emitter pads tests to
;; the widest, subject to its existing fit test.
(:wat::rete::defrule :fmt::cond-clause-table
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::cond"))
         (:wat::grep::Node  (?first <- :id) (?p <- :parent) (?fi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?fi 1))
         (:wat::grep::Node  (?cl <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))]
  :then [(:wat::fmt::Claim :form ?cl)
         (:wat::fmt::TableRow :form ?cl :group ?first)])
