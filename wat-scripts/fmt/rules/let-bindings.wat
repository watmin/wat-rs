;; R3 binding VECTOR — a dispatch target because it decides its children's layout.
;; Claims the vector; positions ITS children (one binder pair per line).
;; The `let` rule does not reach in here.

(:wat::rete::defrule :fmt::let-bindings-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::let"))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?bi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?bi 1))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))]
  :then [(:wat::fmt::Claim :form ?b)])

;; one BINDER per line. A binder is a PAIR — name at an even index, value at the odd one after
;; it — so every even child of the binding vector past the first starts a line, aligned under
;; the first binder (one space inside the opening bracket).
(:wat::rete::defrule :fmt::let-binder-per-line
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::let"))
         (:wat::grep::Node  (?b <- :id) (?p <- :parent) (?bi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?bi 1))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))
         (:wat::grep::Node  (?bind <- :id) (?b <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem ?ci 2 :undefined 1) 0))]
  :then [(:wat::fmt::Break :id ?bind :kind "align")])
