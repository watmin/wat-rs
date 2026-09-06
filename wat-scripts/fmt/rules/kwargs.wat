;; A maximal TRAILING run of (keyword, value) pairs. Shape copied from
;; src/check.rs is_kwargs: length >= 2, length even, every even-offset
;; element a keyword — applied to a suffix, not the whole arg list.
;; `:-` and `->` are never keys of a run (type-args / ret-spec).
;;
;; Break each key; withhold each value; AlignPairs pads first tokens so
;; second tokens line up. Children BEFORE the run follow the ordinary
;; leading-atom rule. Parent must be a list. Claim is its own rule so
;; R11's `not Claim` is stratified after it. The start is the unique
;; smallest valid index.

(:wat::rete::defrule :fmt::kwargs-claim-from-1
  :when [(:wat::grep::Node  (?p <- :id) (?pk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?pk "list"))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::grep::Named (?start <- :id) (?sn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?sn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?sn "->"))
         (:wat::grep::Node  (?last <- :id) (?p <- :parent) (?L <- :index))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?more <- :id) (?p <- :parent) (?mi <- :index))
             (:wat::rete::where (:wat::rete::i64::= ?mi (:wat::rete::i64::+ ?L 1 :undefined 0)))))
         (:wat::rete::where (:wat::rete::i64::>= ?L 2))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem ?L 2 :undefined 1) 0))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi 1))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bd <- :id) (?p <- :parent) (?bdi <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bdi 1))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bdi 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?bd <- :id) (?bdn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?bdn ":-"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?ba <- :id) (?p <- :parent) (?bai <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bai 1))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bai 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?ba <- :id) (?ban <- :name))
             (:wat::rete::where (:wat::rete::string::= ?ban "->"))))]
  :then [(:wat::fmt::Claim :form ?p)
         (:wat::fmt::AlignPairs :form ?p)])

(:wat::rete::defrule :fmt::kwargs-claim-later
  :when [(:wat::grep::Node  (?p <- :id) (?pk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?pk "list"))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::> ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::grep::Named (?start <- :id) (?sn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?sn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?sn "->"))
         (:wat::grep::Node  (?last <- :id) (?p <- :parent) (?L <- :index))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?more <- :id) (?p <- :parent) (?mi <- :index))
             (:wat::rete::where (:wat::rete::i64::= ?mi (:wat::rete::i64::+ ?L 1 :undefined 0)))))
         (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s 1 :undefined 0)))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
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
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bd <- :id) (?p <- :parent) (?bdi <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bdi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bdi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?bd <- :id) (?bdn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?bdn ":-"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?ba <- :id) (?p <- :parent) (?bai <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bai ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bai ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?ba <- :id) (?ban <- :name))
             (:wat::rete::where (:wat::rete::string::= ?ban "->"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?alt <- :id) (?p <- :parent) (?s2 <- :index) (?a2k <- :kind))
             (:wat::rete::where (:wat::rete::i64::> ?s2 0))
             (:wat::rete::where (:wat::rete::i64::< ?s2 ?s))
             (:wat::rete::where (:wat::rete::string::= ?a2k "keyword"))
             (:wat::grep::Named (?alt <- :id) (?a2n <- :name))
             (:wat::rete::where (:wat::rete::string::not= ?a2n ":-"))
             (:wat::rete::where (:wat::rete::string::not= ?a2n "->"))
             (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s2 1 :undefined 0)))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s2 :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::not
               (:wat::rete::and
                 (:wat::grep::Node  (?ab <- :id) (?p <- :parent) (?abi <- :index) (?abk <- :kind))
                 (:wat::rete::where (:wat::rete::i64::>= ?abi ?s2))
                 (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?abi ?s2 :undefined 0) 2 :undefined 1) 0))
                 (:wat::rete::where (:wat::rete::string::not= ?abk "keyword"))))))]
  :then [(:wat::fmt::Claim :form ?p)
         (:wat::fmt::AlignPairs :form ?p)])

(:wat::rete::defrule :fmt::kwargs-keys-from-1
  :when [(:wat::fmt::Claim (?p <- :form))
         (:wat::fmt::AlignPairs (?p <- :form))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::grep::Named (?start <- :id) (?sn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?sn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?sn "->"))
         (:wat::grep::Node  (?last <- :id) (?p <- :parent) (?L <- :index))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?more <- :id) (?p <- :parent) (?mi <- :index))
             (:wat::rete::where (:wat::rete::i64::= ?mi (:wat::rete::i64::+ ?L 1 :undefined 0)))))
         (:wat::rete::where (:wat::rete::i64::>= ?L 2))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem ?L 2 :undefined 1) 0))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi 1))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bd <- :id) (?p <- :parent) (?bdi <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bdi 1))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bdi 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?bd <- :id) (?bdn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?bdn ":-"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?ba <- :id) (?p <- :parent) (?bai <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bai 1))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bai 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?ba <- :id) (?ban <- :name))
             (:wat::rete::where (:wat::rete::string::= ?ban "->"))))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index) (?ck <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?ck "keyword"))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?s))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?ci ?s :undefined 0) 2 :undefined 1) 0))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::+ ?ci 1 :undefined 0)))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])

(:wat::rete::defrule :fmt::kwargs-keys-later
  :when [(:wat::fmt::Claim (?p <- :form))
         (:wat::fmt::AlignPairs (?p <- :form))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::> ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::grep::Named (?start <- :id) (?sn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?sn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?sn "->"))
         (:wat::grep::Node  (?last <- :id) (?p <- :parent) (?L <- :index))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?more <- :id) (?p <- :parent) (?mi <- :index))
             (:wat::rete::where (:wat::rete::i64::= ?mi (:wat::rete::i64::+ ?L 1 :undefined 0)))))
         (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s 1 :undefined 0)))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
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
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bd <- :id) (?p <- :parent) (?bdi <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bdi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bdi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?bd <- :id) (?bdn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?bdn ":-"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?ba <- :id) (?p <- :parent) (?bai <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bai ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bai ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?ba <- :id) (?ban <- :name))
             (:wat::rete::where (:wat::rete::string::= ?ban "->"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?alt <- :id) (?p <- :parent) (?s2 <- :index) (?a2k <- :kind))
             (:wat::rete::where (:wat::rete::i64::> ?s2 0))
             (:wat::rete::where (:wat::rete::i64::< ?s2 ?s))
             (:wat::rete::where (:wat::rete::string::= ?a2k "keyword"))
             (:wat::grep::Named (?alt <- :id) (?a2n <- :name))
             (:wat::rete::where (:wat::rete::string::not= ?a2n ":-"))
             (:wat::rete::where (:wat::rete::string::not= ?a2n "->"))
             (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s2 1 :undefined 0)))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s2 :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::not
               (:wat::rete::and
                 (:wat::grep::Node  (?ab <- :id) (?p <- :parent) (?abi <- :index) (?abk <- :kind))
                 (:wat::rete::where (:wat::rete::i64::>= ?abi ?s2))
                 (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?abi ?s2 :undefined 0) 2 :undefined 1) 0))
                 (:wat::rete::where (:wat::rete::string::not= ?abk "keyword"))))))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index) (?ck <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?ck "keyword"))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?s))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?ci ?s :undefined 0) 2 :undefined 1) 0))
         (:wat::grep::Node  (?v <- :id) (?p <- :parent) (?vi <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?vi (:wat::rete::i64::+ ?ci 1 :undefined 0)))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])

;; Prefix of a trailing pair-run: ordinary leading-atom. First compound in
;; the prefix, and every prefix child after it, starts a line. Withhold after
;; `->` / `:-` so a type-args vector stays on the head line.
(:wat::rete::defrule :fmt::kwargs-prefix-list
  :when [(:wat::fmt::Claim (?p <- :form))
         (:wat::fmt::AlignPairs (?p <- :form))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::> ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::grep::Named (?start <- :id) (?sn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?sn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?sn "->"))
         (:wat::grep::Node  (?last <- :id) (?p <- :parent) (?L <- :index))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?more <- :id) (?p <- :parent) (?mi <- :index))
             (:wat::rete::where (:wat::rete::i64::= ?mi (:wat::rete::i64::+ ?L 1 :undefined 0)))))
         (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s 1 :undefined 0)))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bd <- :id) (?p <- :parent) (?bdi <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bdi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bdi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?bd <- :id) (?bdn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?bdn ":-"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?ba <- :id) (?p <- :parent) (?bai <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bai ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bai ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?ba <- :id) (?ban <- :name))
             (:wat::rete::where (:wat::rete::string::= ?ban "->"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?alt <- :id) (?p <- :parent) (?s2 <- :index) (?a2k <- :kind))
             (:wat::rete::where (:wat::rete::i64::> ?s2 0))
             (:wat::rete::where (:wat::rete::i64::< ?s2 ?s))
             (:wat::rete::where (:wat::rete::string::= ?a2k "keyword"))
             (:wat::grep::Named (?alt <- :id) (?a2n <- :name))
             (:wat::rete::where (:wat::rete::string::not= ?a2n ":-"))
             (:wat::rete::where (:wat::rete::string::not= ?a2n "->"))
             (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s2 1 :undefined 0)))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s2 :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::not
               (:wat::rete::and
                 (:wat::grep::Node  (?ab <- :id) (?p <- :parent) (?abi <- :index) (?abk <- :kind))
                 (:wat::rete::where (:wat::rete::i64::>= ?abi ?s2))
                 (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?abi ?s2 :undefined 0) 2 :undefined 1) 0))
                 (:wat::rete::where (:wat::rete::string::not= ?abk "keyword"))))))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?fi <- :index) (?fk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?fk "list"))
         (:wat::rete::where (:wat::rete::i64::> ?fi 0))
         (:wat::rete::where (:wat::rete::i64::< ?fi ?s))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?fi))
         (:wat::rete::where (:wat::rete::i64::< ?ci ?s))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?arrow <- :id) (?p <- :parent) (?ai <- :index))
             (:wat::grep::Named (?arrow <- :id) (?an <- :name))
             (:wat::rete::where (:wat::rete::string::= ?an "->"))
             (:wat::rete::where (:wat::rete::i64::= ?ai (:wat::rete::i64::- ?ci 1 :undefined 0)))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dsh <- :id) (?p <- :parent) (?dsi <- :index))
             (:wat::grep::Named (?dsh <- :id) (?dsn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dsn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?dsi (:wat::rete::i64::- ?ci 1 :undefined 0)))))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])

(:wat::rete::defrule :fmt::kwargs-prefix-vector
  :when [(:wat::fmt::Claim (?p <- :form))
         (:wat::fmt::AlignPairs (?p <- :form))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::> ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::grep::Named (?start <- :id) (?sn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?sn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?sn "->"))
         (:wat::grep::Node  (?last <- :id) (?p <- :parent) (?L <- :index))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?more <- :id) (?p <- :parent) (?mi <- :index))
             (:wat::rete::where (:wat::rete::i64::= ?mi (:wat::rete::i64::+ ?L 1 :undefined 0)))))
         (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s 1 :undefined 0)))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bd <- :id) (?p <- :parent) (?bdi <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bdi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bdi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?bd <- :id) (?bdn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?bdn ":-"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?ba <- :id) (?p <- :parent) (?bai <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bai ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bai ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?ba <- :id) (?ban <- :name))
             (:wat::rete::where (:wat::rete::string::= ?ban "->"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?alt <- :id) (?p <- :parent) (?s2 <- :index) (?a2k <- :kind))
             (:wat::rete::where (:wat::rete::i64::> ?s2 0))
             (:wat::rete::where (:wat::rete::i64::< ?s2 ?s))
             (:wat::rete::where (:wat::rete::string::= ?a2k "keyword"))
             (:wat::grep::Named (?alt <- :id) (?a2n <- :name))
             (:wat::rete::where (:wat::rete::string::not= ?a2n ":-"))
             (:wat::rete::where (:wat::rete::string::not= ?a2n "->"))
             (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s2 1 :undefined 0)))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s2 :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::not
               (:wat::rete::and
                 (:wat::grep::Node  (?ab <- :id) (?p <- :parent) (?abi <- :index) (?abk <- :kind))
                 (:wat::rete::where (:wat::rete::i64::>= ?abi ?s2))
                 (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?abi ?s2 :undefined 0) 2 :undefined 1) 0))
                 (:wat::rete::where (:wat::rete::string::not= ?abk "keyword"))))))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?fi <- :index) (?fk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?fk "vector"))
         (:wat::rete::where (:wat::rete::i64::> ?fi 0))
         (:wat::rete::where (:wat::rete::i64::< ?fi ?s))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?fi))
         (:wat::rete::where (:wat::rete::i64::< ?ci ?s))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?arrow <- :id) (?p <- :parent) (?ai <- :index))
             (:wat::grep::Named (?arrow <- :id) (?an <- :name))
             (:wat::rete::where (:wat::rete::string::= ?an "->"))
             (:wat::rete::where (:wat::rete::i64::= ?ai (:wat::rete::i64::- ?ci 1 :undefined 0)))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dsh <- :id) (?p <- :parent) (?dsi <- :index))
             (:wat::grep::Named (?dsh <- :id) (?dsn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dsn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?dsi (:wat::rete::i64::- ?ci 1 :undefined 0)))))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])

(:wat::rete::defrule :fmt::kwargs-prefix-map
  :when [(:wat::fmt::Claim (?p <- :form))
         (:wat::fmt::AlignPairs (?p <- :form))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::> ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::grep::Named (?start <- :id) (?sn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?sn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?sn "->"))
         (:wat::grep::Node  (?last <- :id) (?p <- :parent) (?L <- :index))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?more <- :id) (?p <- :parent) (?mi <- :index))
             (:wat::rete::where (:wat::rete::i64::= ?mi (:wat::rete::i64::+ ?L 1 :undefined 0)))))
         (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s 1 :undefined 0)))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bd <- :id) (?p <- :parent) (?bdi <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bdi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bdi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?bd <- :id) (?bdn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?bdn ":-"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?ba <- :id) (?p <- :parent) (?bai <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bai ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bai ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?ba <- :id) (?ban <- :name))
             (:wat::rete::where (:wat::rete::string::= ?ban "->"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?alt <- :id) (?p <- :parent) (?s2 <- :index) (?a2k <- :kind))
             (:wat::rete::where (:wat::rete::i64::> ?s2 0))
             (:wat::rete::where (:wat::rete::i64::< ?s2 ?s))
             (:wat::rete::where (:wat::rete::string::= ?a2k "keyword"))
             (:wat::grep::Named (?alt <- :id) (?a2n <- :name))
             (:wat::rete::where (:wat::rete::string::not= ?a2n ":-"))
             (:wat::rete::where (:wat::rete::string::not= ?a2n "->"))
             (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s2 1 :undefined 0)))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s2 :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::not
               (:wat::rete::and
                 (:wat::grep::Node  (?ab <- :id) (?p <- :parent) (?abi <- :index) (?abk <- :kind))
                 (:wat::rete::where (:wat::rete::i64::>= ?abi ?s2))
                 (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?abi ?s2 :undefined 0) 2 :undefined 1) 0))
                 (:wat::rete::where (:wat::rete::string::not= ?abk "keyword"))))))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?fi <- :index) (?fk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?fk "map"))
         (:wat::rete::where (:wat::rete::i64::> ?fi 0))
         (:wat::rete::where (:wat::rete::i64::< ?fi ?s))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?fi))
         (:wat::rete::where (:wat::rete::i64::< ?ci ?s))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?arrow <- :id) (?p <- :parent) (?ai <- :index))
             (:wat::grep::Named (?arrow <- :id) (?an <- :name))
             (:wat::rete::where (:wat::rete::string::= ?an "->"))
             (:wat::rete::where (:wat::rete::i64::= ?ai (:wat::rete::i64::- ?ci 1 :undefined 0)))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dsh <- :id) (?p <- :parent) (?dsi <- :index))
             (:wat::grep::Named (?dsh <- :id) (?dsn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dsn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?dsi (:wat::rete::i64::- ?ci 1 :undefined 0)))))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])

(:wat::rete::defrule :fmt::kwargs-prefix-set
  :when [(:wat::fmt::Claim (?p <- :form))
         (:wat::fmt::AlignPairs (?p <- :form))
         (:wat::grep::Node  (?start <- :id) (?p <- :parent) (?s <- :index) (?sk <- :kind))
         (:wat::rete::where (:wat::rete::i64::> ?s 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "keyword"))
         (:wat::grep::Named (?start <- :id) (?sn <- :name))
         (:wat::rete::where (:wat::rete::string::not= ?sn ":-"))
         (:wat::rete::where (:wat::rete::string::not= ?sn "->"))
         (:wat::grep::Node  (?last <- :id) (?p <- :parent) (?L <- :index))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?more <- :id) (?p <- :parent) (?mi <- :index))
             (:wat::rete::where (:wat::rete::i64::= ?mi (:wat::rete::i64::+ ?L 1 :undefined 0)))))
         (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s 1 :undefined 0)))
         (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bad <- :id) (?p <- :parent) (?bi <- :index) (?bk <- :kind))
             (:wat::rete::where (:wat::rete::i64::>= ?bi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::where (:wat::rete::string::not= ?bk "keyword"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?bd <- :id) (?p <- :parent) (?bdi <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bdi ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bdi ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?bd <- :id) (?bdn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?bdn ":-"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?ba <- :id) (?p <- :parent) (?bai <- :index))
             (:wat::rete::where (:wat::rete::i64::>= ?bai ?s))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?bai ?s :undefined 0) 2 :undefined 1) 0))
             (:wat::grep::Named (?ba <- :id) (?ban <- :name))
             (:wat::rete::where (:wat::rete::string::= ?ban "->"))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?alt <- :id) (?p <- :parent) (?s2 <- :index) (?a2k <- :kind))
             (:wat::rete::where (:wat::rete::i64::> ?s2 0))
             (:wat::rete::where (:wat::rete::i64::< ?s2 ?s))
             (:wat::rete::where (:wat::rete::string::= ?a2k "keyword"))
             (:wat::grep::Named (?alt <- :id) (?a2n <- :name))
             (:wat::rete::where (:wat::rete::string::not= ?a2n ":-"))
             (:wat::rete::where (:wat::rete::string::not= ?a2n "->"))
             (:wat::rete::where (:wat::rete::i64::>= ?L (:wat::rete::i64::+ ?s2 1 :undefined 0)))
             (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::+ (:wat::rete::i64::- ?L ?s2 :undefined 0) 1 :undefined 0) 2 :undefined 1) 0))
             (:wat::rete::not
               (:wat::rete::and
                 (:wat::grep::Node  (?ab <- :id) (?p <- :parent) (?abi <- :index) (?abk <- :kind))
                 (:wat::rete::where (:wat::rete::i64::>= ?abi ?s2))
                 (:wat::rete::where (:wat::rete::i64::= (:wat::rete::i64::rem (:wat::rete::i64::- ?abi ?s2 :undefined 0) 2 :undefined 1) 0))
                 (:wat::rete::where (:wat::rete::string::not= ?abk "keyword"))))))
         (:wat::grep::Node  (?comp <- :id) (?p <- :parent) (?fi <- :index) (?fk <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?fk "set"))
         (:wat::rete::where (:wat::rete::i64::> ?fi 0))
         (:wat::rete::where (:wat::rete::i64::< ?fi ?s))
         (:wat::grep::Node  (?c <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?ci 0))
         (:wat::rete::where (:wat::rete::i64::>= ?ci ?fi))
         (:wat::rete::where (:wat::rete::i64::< ?ci ?s))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?arrow <- :id) (?p <- :parent) (?ai <- :index))
             (:wat::grep::Named (?arrow <- :id) (?an <- :name))
             (:wat::rete::where (:wat::rete::string::= ?an "->"))
             (:wat::rete::where (:wat::rete::i64::= ?ai (:wat::rete::i64::- ?ci 1 :undefined 0)))))
         (:wat::rete::not
           (:wat::rete::and
             (:wat::grep::Node  (?dsh <- :id) (?p <- :parent) (?dsi <- :index))
             (:wat::grep::Named (?dsh <- :id) (?dsn <- :name))
             (:wat::rete::where (:wat::rete::string::= ?dsn ":-"))
             (:wat::rete::where (:wat::rete::i64::= ?dsi (:wat::rete::i64::- ?ci 1 :undefined 0)))))]
  :then [(:wat::fmt::Break :id ?c :kind "block")])
