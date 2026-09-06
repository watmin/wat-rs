;; Keyword-argument runs. A run is a maximal tail of children whose even-offset
;; members (from the start of the tail) are keywords. Claim the parent so R11
;; does not compete. Break each key; the value rides with it. AlignPairs asks
;; the emitter to pad first tokens so the second tokens line up. No rule names
;; a width.
;;
;; Positionals before the first key get no Break — they ride the head line.
;; A start whose previous sibling is `->` or `:-` is refused (ret-specs are not
;; argument runs). Parent must be a list.
;;
;; Claim is its own rule (same shape as defn-claim) so `not Claim` in R11 is
;; stratified after it.

(:wat::rete::defrule :fmt::kwargs-claim-from-1
  :when [(:wat::grep::Node  (?p <- :id) (?pk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?pk "list"))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi 1))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))]
  :then [(:wat::fmt::Claim :form ?p)
         (:wat::fmt::AlignPairs :form ?p)])

(:wat::rete::defrule :fmt::kwargs-claim-later
  :when [(:wat::grep::Node  (?p <- :id) (?pk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?pk "list"))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::> ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?prev <- :id) (?p <- :parent) (?pi <- :index))
             (:wat::grep::Named (?prev <- :id) (?pn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?pn "->"))
             (:wat::rete::where (:wat::rete::i64::= ?pi (:wat::rete::i64::- ?s 1 :undefined 0)))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dash <- :id) (?p <- :parent) (?di <- :index))
             (:wat::grep::Named (?dash <- :id) (?dn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?di (:wat::rete::i64::- ?s 1 :undefined 0)))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))
         (:wat::grep::Node  (?wit <- :id) (?p <- :parent) (?wi <- :index) (?wk <- :kind))
         (:wat::rete::where (:wat::rete::i64::>= ?wi (:wat::rete::i64::- ?s 1 :undefined 0)))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?wi (:wat::rete::i64::- ?s 1 :undefined 0) :undefined 0) 2 :undefined 1) 0))
         (:wat::rete::where (:wat::rete::string::not= ?wk "keyword"))]
  :then [(:wat::fmt::Claim :form ?p)
         (:wat::fmt::AlignPairs :form ?p)])

(:wat::rete::defrule :fmt::kwargs-keys-from-1
  :when [(:wat::fmt::Claim (?p <- :form))
         (:wat::fmt::AlignPairs (?p <- :form))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi 1))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index) (?ck <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?ck "keyword"))
         (:wat::rete::where (:wat::rete::i64::>= ?ci 1))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?ci 1 :undefined 0) 2 :undefined 1) 0))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::+ ?ci 1 :undefined 0)))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])

(:wat::rete::defrule :fmt::kwargs-keys-later
  :when [(:wat::fmt::Claim (?p <- :form))
         (:wat::fmt::AlignPairs (?p <- :form))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::> ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?prev <- :id) (?p <- :parent) (?pi <- :index))
             (:wat::grep::Named (?prev <- :id) (?pn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?pn "->"))
             (:wat::rete::where (:wat::rete::i64::= ?pi (:wat::rete::i64::- ?s 1 :undefined 0)))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dash <- :id) (?p <- :parent) (?di <- :index))
             (:wat::grep::Named (?dash <- :id) (?dn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?di (:wat::rete::i64::- ?s 1 :undefined 0)))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))
         (:wat::grep::Node  (?wit <- :id) (?p <- :parent) (?wi <- :index) (?wk <- :kind))
         (:wat::rete::where (:wat::rete::i64::>= ?wi (:wat::rete::i64::- ?s 1 :undefined 0)))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?wi (:wat::rete::i64::- ?s 1 :undefined 0) :undefined 0) 2 :undefined 1) 0))
         (:wat::rete::where (:wat::rete::string::not= ?wk "keyword"))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index) (?ck <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?ck "keyword"))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?s))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?ci ?s :undefined 0) 2 :undefined 1) 0))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::+ ?ci 1 :undefined 0)))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])
