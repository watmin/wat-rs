;; head-position.wat — WHERE IS THIS VERB ACTUALLY CALLED?
;;
;; The question grep cannot answer. A text search for `:wat::core::first` returns every
;; occurrence: calls, mentions inside a comment, a longer name that contains it, a string
;; literal, and the argument position of some OTHER call. Those are different facts about the
;; program and text renders them identically.
;;
;; A form's HEAD is its child at index 0. That is the whole definition of "called here", and it
;; is one integer in the fact base:
;;
;;     (:wat::grep::Node (?i <- :index))  +  (:wat::rete::where (i64::= ?i 0))
;;
;; `:wat::core::first` is the subject because it is PARTIAL — it raises on an empty sequence
;; ("sequence has fewer than 1 element(s)"), which is exactly the red this session hit in
;; corpus-03. Every call site is a place that can raise; every mention is not.

(:wat::core::defrecord :hp::IsHead [id <- :wat::core::i64])

;; a node in head position — index 0 of its parent form
(:wat::rete::defrule :hp::head
  :when [(:wat::grep::Node (?id <- :id) (?k <- :kind) (?i <- :index))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::core::i64::= ?i 0))]
  :then [(:hp::IsHead :id ?id)])

;; ...whose name is the partial verb
(:wat::rete::defrule :hp::calls-first
  :when [(:hp::IsHead (?id <- :id))
         (:wat::grep::Named (?id <- :id) (?n <- :name))
         (:wat::grep::Span  (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::first"))]
  :then [(:wat::grep::Match
           :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "calls-a-partial-verb"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "verb" :value ?n)))])

;; `collect-rules` reflects the symbol table for every zero-arg fn in `hp::` whose return type is
;; `:wat::rete::Rule` — the marker `defrule` plants. A hand-written vector would be a second list
;; of the same rules, and a rule added later would silently not run.
(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :hp))
