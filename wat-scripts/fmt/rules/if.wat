;; `if` layout. A NEW FILE and nothing else. Arc 277.
;;
;; Builder, 2026-09-06: the truthy measurement rides the head line; each branch
;; takes its own line one level in. Three children, and only the first rides.
;;
;; Copied from match.wat — same shape, different head. A `match`'s scrutinee
;; rides; an `if`'s test rides. The arms / branches break from index 2 on.
;; Break names a kind (BreakKind); the emitter computes the rest.

(:wat::rete::defrule :fmt::if-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n "wat.core/if"))]
  :then [(:wat::fmt::Claim :form ?p)])

;; Child 0 is the head, child 1 is the test (stays on the head line),
;; so every child from index 2 on starts its own line.
(:wat::rete::defrule :fmt::if-branch-per-line
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n "wat.core/if"))
         (:wat::grep::Node  (?br <- :id) (?p <- :parent) (?bi <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?bi 1))]
  :then [(:wat::fmt::Break :id ?br :kind (:wat::fmt::BreakKind::Block {}))])
