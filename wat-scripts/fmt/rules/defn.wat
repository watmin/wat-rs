;; R1 — defn layout. ONE file, nothing else. Arc 277.
;; Head + name + param-spec (`:- [P…]`) if present stay on line 1 (no Break).
;; Arg-spec on its own line (empty `[]` included); one argument per line after the first;
;; `->` (ret-type) on its own line; body on its own line.

(:wat::rete::defrule :fmt::defn-argspec-break
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
  :then [(:wat::fmt::Break :id ?args :indent 2)])

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
  :then [(:wat::fmt::Break :id ?ch :indent 3)])

(:wat::rete::defrule :fmt::defn-ret-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defn"))
         (:wat::grep::Node  (?arrow <- :id) (?p <- :parent))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "->"))]
  :then [(:wat::fmt::Break :id ?arrow :indent 2)])

(:wat::rete::defrule :fmt::defn-body-break
  :when [(:wat::grep::Node  (?h <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::defn"))
         (:wat::grep::Node  (?arrow <- :id) (?p <- :parent) (?ari <- :index))
         (:wat::grep::Named (?arrow <- :id) (?an <- :name))
         (:wat::rete::where (:wat::rete::string::= ?an "->"))
         (:wat::grep::Node  (?body <- :id) (?p <- :parent) (?bi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?bi (:wat::rete::i64::+ ?ari 2 :undefined 0)))]
  :then [(:wat::fmt::Break :id ?body :indent 2)])
