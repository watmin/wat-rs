
;; How many MAP literals are in the wat corpus, and how big are they?
;; The builder asked whether the formatter needs a rule for pretty-printing maps/records.
;; Records are already covered by the pair-run rule; a MAP literal is not. This asks the
;; fact base rather than grepping, because `{` also appears inside interpolation strings.

(:wat::rete::defrule :ml::map-node
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::rete::where  (:wat::rete::string::= ?k "map"))
         (:wat::grep::Node   (?c <- :id) (?id <- :parent))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?co <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?co :end-line ?el :end-col ?ec
           :rule "map-literal"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "id" :value (:wat::rete::i64::to-string ?id))))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :ml))
