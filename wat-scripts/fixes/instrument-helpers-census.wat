;; wat-scripts/fixes/instrument-helpers-census.wat — Stone D finder (census only).
;;
;; Counts KEYWORD leaves for the helper family. Comments and string literals are
;; not keywords. Count occurrences, not lines.
;;
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/instrument-helpers-census.wat
;;
;; No applier — semantic rewrites are hand-typed; this file is the instrument.

(:wat::rete::defrule :ih::take-one
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":demo::take-one"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "take-one"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::rete::defrule :ih::wait-pending
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":demo::wait-pending"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "wait-pending"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::rete::defrule :ih::wait-inflight
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":demo::wait-inflight"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "wait-inflight"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::rete::defrule :ih::q-depth
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":demo::q-depth"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "q-depth"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::rete::defrule :ih::depth-of-topic
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":demo::depth-of-topic"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "depth-of-topic"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::rete::defrule :ih::wait-inbox-zero
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":demo::wait-inbox-zero"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "wait-inbox-zero"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::rete::defrule :ih::wait-drained
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":fanout::wait-drained"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "wait-drained"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::rete::defrule :ih::wait-pending-zero
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":fanout::wait-pending-zero"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "wait-pending-zero"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::rete::defrule :ih::depth-of
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":fanout::depth-of"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "depth-of"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "name" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :ih))
