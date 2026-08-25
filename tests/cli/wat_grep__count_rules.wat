;; wat_grep__count_rules.wat — fixture: one Match-emitting rule per fact type, for
;; BRIEF-STONE-wat-grep-never-lies.md's G1/G2/G5/G6. `?id` rides in as `:line`/`:end-line` so
;; each firing is a DISTINCT Match — a rete Session is set-semantics over facts, so an all-zero
;; Match would collapse every firing of a rule into one fact and undercount.

(:wat::rete::defrule :cnt::node
  :when [(:wat::grep::Node (?id <- :id))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?id :col 0 :end-line ?id :end-col 0
           :rule "cnt::node" :captures (:wat::rete::core::PersistentVector))])

(:wat::rete::defrule :cnt::named
  :when [(:wat::grep::Named (?id <- :id))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?id :col 0 :end-line ?id :end-col 0
           :rule "cnt::named" :captures (:wat::rete::core::PersistentVector))])

(:wat::rete::defrule :cnt::span
  :when [(:wat::grep::Span (?id <- :id))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?id :col 0 :end-line ?id :end-col 0
           :rule "cnt::span" :captures (:wat::rete::core::PersistentVector))])

(:wat::rete::defrule :cnt::written
  :when [(:wat::grep::Written (?id <- :id))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?id :col 0 :end-line ?id :end-col 0
           :rule "cnt::written" :captures (:wat::rete::core::PersistentVector))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :cnt))
