;; R11 — sibling breaking is all-or-nothing. A NEW FILE and nothing else.
;; If any child of a form is on a different line from a sibling, every child
;; (except the head at index 0) starts a line.
;;
;; Default rule: fires only where the parent is unclaimed (`Claim`).
;; A specific rule claims exactly the node it dispatched on; the fallback
;; reaches unruled forms at any depth.
;; Break names a kind ("block" / "align"); the emitter computes the rest.
;;
;; Fallback marks the unruled parent so the Break-application wall accepts
;; these Breaks. R11 cannot assert Claim — that is `not Claim -> Claim` and
;; it races the per-child Breaks.

(:wat::rete::defrule :fmt::siblings-fallback
  :when [(:wat::grep::Node  (?head <- :id) (?p <- :parent) (?hi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::not (:wat::fmt::Claim (?p <- :form)))
         (:wat::grep::Node  (?a <- :id) (?p <- :parent) (?ia <- :index))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?ib <- :index))
         (:wat::grep::Span  (?a <- :id) (?la <- :line))
         (:wat::grep::Span  (?b <- :id) (?lb <- :line))
         (:wat::rete::where (:wat::rete::i64::not= ?la ?lb))]
  :then [(:wat::fmt::Fallback :node ?p)])

(:wat::rete::defrule :fmt::siblings-all-or-nothing
  :when [(:wat::fmt::Fallback (?p <- :node))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ic <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ic 0))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])
