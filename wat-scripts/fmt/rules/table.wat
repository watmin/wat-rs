;; Adjacent same-typed instances form a table. Same HEAD, same KEY SEQUENCE
;; (positional: same arity, encoded as "#N"), consecutive :index under one
;; parent, two or more. group is the first member's id. Claim each member
;; so R11 and the pair-run do not explode it. The emitter pads; this file
;; names no width.

(:wat::rete::defrule :fmt::table-start
  :when [(:wat::grep::Node  (?a <- :id) (?p <- :parent) (?i <- :index) (?ak <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?ak "list"))
         (:wat::fmt::FormSig (?a <- :form) (?h <- :head) (?k <- :keys))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?j <- :index) (?bk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?bk "list"))
         (:wat::rete::where (:wat::rete::i64::= ?j (:wat::rete::i64::+ ?i 1 :undefined 0)))
         (:wat::fmt::FormSig (?b <- :form) (?h <- :head) (?k <- :keys))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?pred <- :id) (?p <- :parent) (?pi <- :index) (?pk <- :kind))
             (:wat::rete::where (:wat::rete::string::= ?pk "list"))
             (:wat::rete::where (:wat::rete::i64::= ?pi (:wat::rete::i64::- ?i 1 :undefined 0)))
             (:wat::fmt::FormSig (?pred <- :form) (?h <- :head) (?k <- :keys))))]
  :then [(:wat::fmt::TableRow :form ?a :group ?a)
         (:wat::fmt::TableRow :form ?b :group ?a)
         (:wat::fmt::Claim :form ?a)
         (:wat::fmt::Claim :form ?b)])

(:wat::rete::defrule :fmt::table-extend
  :when [(:wat::fmt::TableRow (?b <- :form) (?g <- :group))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?i <- :index))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?j <- :index) (?ck <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?ck "list"))
         (:wat::rete::where (:wat::rete::i64::= ?j (:wat::rete::i64::+ ?i 1 :undefined 0)))
         (:wat::fmt::FormSig (?b <- :form) (?h <- :head) (?k <- :keys))
         (:wat::fmt::FormSig (?c <- :form) (?h <- :head) (?k <- :keys))]
  :then [(:wat::fmt::TableRow :form ?c :group ?g)
         (:wat::fmt::Claim :form ?c)])
