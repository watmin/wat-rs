
;; Builder ruled: a pair collection stays on ONE LINE when every value is an ATOM and it fits;
;; any value needing eval (a compound) explodes. This measures the blast radius — how many
;; pair-bearing forms have ALL-ATOM values (would collapse to one line) versus at least one
;; compound (already explode today).

(:wat::rete::defrule :aa::pair-with-compound-value
  :when [(:wat::grep::Node   (?p <- :id) (?pk <- :kind))
         (:wat::grep::Node   (?k <- :id) (?p <- :parent) (?ki <- :index) (?kk <- :kind))
         (:wat::rete::where  (:wat::rete::core::enum::= ?kk (:wat::grep::NodeKind.Keyword {})))
         (:wat::rete::where  (:wat::rete::i64::> ?ki 0))
         (:wat::grep::Node   (?v <- :id) (?p <- :parent) (?vi <- :index) (?vk <- :kind))
         (:wat::rete::where  (:wat::rete::i64::= ?vi (:wat::rete::i64::+ ?ki 1 :undefined 0)))
         (:wat::rete::where  (:wat::rete::core::enum::= ?vk (:wat::grep::NodeKind.List {})))
         (:wat::grep::Span   (?p <- :id) (?l <- :line) (?co <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?co :end-line ?el :end-col ?ec
           :rule "compound-value"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "form" :value (:wat::rete::i64::to-string ?p))))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :aa))
