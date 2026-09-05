;; R4 — `match` layout. A NEW FILE and nothing else. Arc 277.
;;
;; Builder, 2026-09-05:
;;   (:wat::core::match x   ;; term is bound on the first line
;;     (n n)                ;; one arm per line
;;     (_ 0))               ;; closed
;;
;; ⛔ `_` IS LEGAL, and it is the sanctioned close — not a tolerated hack. Measured: 105 uses
;; across 59 corpus files, and the CHECKER is built on it — `src/check.rs:6251`
;; (`MatchShape::Open(_) => wildcard_seen`) drives exhaustiveness from the wildcard, and the
;; non-exhaustive error text at `:6257` literally recommends *"add a fallback `_` arm."*
;;
;; The scrutinee rides the head line — like a `defn`'s NAME and unlike a `let`, whose head line
;; carries nothing. Three forms, three answers: a rule is never copied between them.

(:wat::rete::defrule :fmt::match-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::match"))]
  :then [(:wat::fmt::Claim :form ?p)])

;; one ARM per line. Child 0 is the head, child 1 is the scrutinee (stays on the head line),
;; so every child from index 2 on starts its own line.
(:wat::rete::defrule :fmt::match-arm-per-line
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::match"))
         (:wat::grep::Node  (?arm <- :id) (?p <- :parent) (?ai <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ai 1))
         (:wat::grep::Span  (?p <- :id) (?pc <- :col))]
  :then [(:wat::fmt::Break :id ?arm :indent (:wat::rete::i64::+ ?pc 1 :undefined 2))])
