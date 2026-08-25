;; wat_grep__any_symbol_rule.wat — drives `:wat::grep::run` for G3/G4 (the malformed-vs-balanced
;; control). The rule's content doesn't matter for those rows — G3/G4 assert on the `Unreadable`
;; fact + stderr + exit code, not on whether anything matched — so this fires on any symbol node,
;; which the balanced fixture (`wat_grep__balanced.wat`) has plenty of.
(:wat::rete::defrule :ctrl::any-symbol
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "symbol"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "ctrl::any-symbol"
           :captures (:wat::rete::core::PersistentVector))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :ctrl))
