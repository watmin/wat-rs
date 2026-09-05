;; R11 — sibling breaking is all-or-nothing. A NEW FILE and nothing else.
;; If any child of a form is on a different line from a sibling, every child
;; (except the head at index 0) starts a line.
;;
;; Default rule: fires only where no Claim exists. Specific rules (R1, later R3/R4)
;; assert Claim on the forms they own; R11 is never a competing exclusion list.

(:wat::rete::defrule :fmt::siblings-all-or-nothing
  :when [(:wat::grep::Node  (?head <- :id) (?p <- :parent) (?hi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::not (:wat::fmt::Claim (?p <- :form)))
         (:wat::grep::Node  (?a <- :id) (?p <- :parent) (?ia <- :index))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?ib <- :index))
         (:wat::grep::Span  (?a <- :id) (?la <- :line))
         (:wat::grep::Span  (?b <- :id) (?lb <- :line))
         (:wat::rete::where (:wat::rete::i64::not= ?la ?lb))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ic <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ic 0))
         (:wat::grep::Span  (?p <- :id) (?pc <- :col))]
  :then [(:wat::fmt::Break :id ?c :indent (:wat::rete::i64::+ ?pc 1 :undefined 2))])
