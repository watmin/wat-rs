;; Blank line before a binder whose PREVIOUS binder's value is COMPLEX.
;; Complex = the value is a form with at least one compound child — the same
;; test Part 1 uses to explode it. Derived from structure, never from the
;; previous pass's whitespace, so blanks cannot accumulate.
;;
;; Only BETWEEN binders: even index > 0. The first binder never gets a blank.

(:wat::rete::defrule :fmt::let-blank-after-list
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::let"))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?bi 1))
         (:wat::rete::where (:wat::rete::string::= ?bk "vector"))
         (:wat::grep::Node  (?bind <- :id) (?b <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem ?ci 2 :undefined 1) 0))
         (:wat::grep::Node  (?val <- :id) (?b <- :parent) (?vi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::- ?ci 1 :undefined 0)))
         (:wat::grep::Node  (?kid <- :id) (?val <- :parent) (?kk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?kk "list"))]
  :then [(:wat::fmt::BlankBefore :id ?bind)])

(:wat::rete::defrule :fmt::let-blank-after-vector
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::let"))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?bi 1))
         (:wat::rete::where (:wat::rete::string::= ?bk "vector"))
         (:wat::grep::Node  (?bind <- :id) (?b <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem ?ci 2 :undefined 1) 0))
         (:wat::grep::Node  (?val <- :id) (?b <- :parent) (?vi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::- ?ci 1 :undefined 0)))
         (:wat::grep::Node  (?kid <- :id) (?val <- :parent) (?kk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?kk "vector"))]
  :then [(:wat::fmt::BlankBefore :id ?bind)])

(:wat::rete::defrule :fmt::let-blank-after-map
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::let"))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?bi 1))
         (:wat::rete::where (:wat::rete::string::= ?bk "vector"))
         (:wat::grep::Node  (?bind <- :id) (?b <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem ?ci 2 :undefined 1) 0))
         (:wat::grep::Node  (?val <- :id) (?b <- :parent) (?vi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::- ?ci 1 :undefined 0)))
         (:wat::grep::Node  (?kid <- :id) (?val <- :parent) (?kk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?kk "map"))]
  :then [(:wat::fmt::BlankBefore :id ?bind)])

(:wat::rete::defrule :fmt::let-blank-after-set
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::let"))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?bi 1))
         (:wat::rete::where (:wat::rete::string::= ?bk "vector"))
         (:wat::grep::Node  (?bind <- :id) (?b <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem ?ci 2 :undefined 1) 0))
         (:wat::grep::Node  (?val <- :id) (?b <- :parent) (?vi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::- ?ci 1 :undefined 0)))
         (:wat::grep::Node  (?kid <- :id) (?val <- :parent) (?kk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?kk "set"))]
  :then [(:wat::fmt::BlankBefore :id ?bind)])
