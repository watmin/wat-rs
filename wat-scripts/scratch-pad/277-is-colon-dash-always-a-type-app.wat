
;; Is a LIST whose child at index 1 is `:-` ALWAYS a type application?
;; The builder ruled a parametric arg-spec renders as a one-liner, so the formatter needs a way to
;; RECOGNISE a type application. "child 1 is :-" is the candidate. This asks whether anything else
;; in the corpus has that shape. Emits the HEAD of every such form; the answer is the distribution.

(:wat::rete::defrule :ta::colon-dash-at-1
  :when [(:wat::grep::Node   (?p <- :id) (?pk <- :kind))
         (:wat::rete::where  (:wat::rete::string::= ?pk "list"))
         (:wat::grep::Node   (?c <- :id) (?p <- :parent) (?ci <- :index))
         (:wat::rete::where  (:wat::rete::i64::= ?ci 1))
         (:wat::grep::Named  (?c <- :id) (?cn <- :name))
         (:wat::rete::where  (:wat::rete::string::= ?cn ":-"))
         (:wat::grep::Node   (?h <- :id) (?p <- :parent) (?hi <- :index))
         (:wat::rete::where  (:wat::rete::i64::= ?hi 0))
         (:wat::grep::Named  (?h <- :id) (?hn <- :name))
         (:wat::grep::Span   (?p <- :id) (?l <- :line) (?co <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?co :end-line ?el :end-col ?ec
           :rule "colon-dash-at-1"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "head" :value ?hn)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :ta))
