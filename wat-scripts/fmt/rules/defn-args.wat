;; R1 arg-spec VECTOR — a dispatch target because it decides its children's layout.
;; Claims the vector; positions ITS children (one arg per line after the first).
;; The `defn` rule does not reach in here.

(:wat::rete::defrule :fmt::defn-args-claim
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defn"))
         (:wat::grep::Node  (?args <- :id) (?p <- :parent) (?ai <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))
         (:wat::grep::Node  (?arrow <- :id) (?p <- :parent) (?ari <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?ari (:wat::rete::i64::+ ?ai 1 :undefined 0)))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "->"))]
  :then [(:wat::fmt::Claim :form ?args)])

(:wat::rete::defrule :fmt::defn-arg-per-line
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defn"))
         (:wat::grep::Node  (?args <- :id) (?p <- :parent) (?ai <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))
         (:wat::grep::Node  (?arrow <- :id) (?p <- :parent) (?ari <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?ari (:wat::rete::i64::+ ?ai 1 :undefined 0)))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "->"))
         (:wat::grep::Node  (?ch <- :id) (?args <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem ?ci 3 :undefined 1) 0))]
  :then [(:wat::fmt::Break :id ?ch :kind "align")])
