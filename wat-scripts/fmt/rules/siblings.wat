;; R11 — exploded default. A leading run of ATOM children rides the head line.
;; The first COMPOUND child (list/vector/map/set), and every child after it,
;; starts its own line. The head (index 0) never breaks.
;;
;; ⛔ NOT "break every child" — that would put a leading atom like `m` in
;; `(assoc m (f b) (g b))` on its own line and contradict the ruling.
;;
;; Default rule: fires only where the parent is unclaimed (`Claim`).
;; Fallback marks the unruled parent so the Break-application wall accepts
;; these Breaks. Four kind-rules because rete `or` is not legal inside `:where`.

(:wat::rete::defrule :fmt::siblings-fallback-list
  :when [(:wat::grep::Node  (?head <- :id) (?p <- :parent) (?hi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::not (:wat::fmt::Claim (?p <- :form)))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "list"))]
  :then [(:wat::fmt::Fallback :node ?p)])

(:wat::rete::defrule :fmt::siblings-fallback-vector
  :when [(:wat::grep::Node  (?head <- :id) (?p <- :parent) (?hi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::not (:wat::fmt::Claim (?p <- :form)))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))]
  :then [(:wat::fmt::Fallback :node ?p)])

(:wat::rete::defrule :fmt::siblings-fallback-map
  :when [(:wat::grep::Node  (?head <- :id) (?p <- :parent) (?hi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::not (:wat::fmt::Claim (?p <- :form)))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "map"))]
  :then [(:wat::fmt::Fallback :node ?p)])

(:wat::rete::defrule :fmt::siblings-fallback-set
  :when [(:wat::grep::Node  (?head <- :id) (?p <- :parent) (?hi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::not (:wat::fmt::Claim (?p <- :form)))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "set"))]
  :then [(:wat::fmt::Fallback :node ?p)])

(:wat::rete::defrule :fmt::siblings-explode-list
  :when [(:wat::fmt::Fallback (?p <- :node))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?fi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "list"))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?fi))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])

(:wat::rete::defrule :fmt::siblings-explode-vector
  :when [(:wat::fmt::Fallback (?p <- :node))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?fi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "vector"))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?fi))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])

(:wat::rete::defrule :fmt::siblings-explode-map
  :when [(:wat::fmt::Fallback (?p <- :node))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?fi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "map"))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?fi))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])

(:wat::rete::defrule :fmt::siblings-explode-set
  :when [(:wat::fmt::Fallback (?p <- :node))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?fi <- :index) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "set"))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?fi))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])
