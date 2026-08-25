;; wat_grep__g7_rule.wat — the `:user::grep` program for G7. Fires on the `<-` binder symbol —
;; the exact shape `wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat`'s
;; `:fx::match-arrow` proved (and `probe-grep-driver.wat`/`probe-grep-cli.wat` reuse).
(:wat::core::defrecord :g7::IsArrow [id <- :wat::core::i64])

(:wat::rete::defrule :g7::arrow
  :when [(:wat::grep::Node  (?id <- :id) (?k <- :kind))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?k "symbol"))
         (:wat::rete::where (:wat::rete::string::= ?n "<-"))]
  :then [(:g7::IsArrow :id ?id)])

(:wat::rete::defrule :g7::match-arrow
  :when [(:wat::grep::Node (?id <- :id) (?k <- :kind))
         (:wat::grep::Span (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:g7::IsArrow (?id <- :id))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match
           :file     ?f
           :line     ?l
           :col      ?c
           :end-line ?el
           :end-col  ?ec
           :rule     "g7::match-arrow"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "kind" :value ?k)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector :- [:wat::rete::Rule] (:g7::arrow) (:g7::match-arrow)))
