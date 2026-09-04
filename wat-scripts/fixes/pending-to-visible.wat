;; wat-scripts/fixes/pending-to-visible.wat — Stone D: the surface names the two counts.
;;
;; Mechanical rename of the queue's StatsResponse / State field keywords:
;;
;;   :queue::queue::State/pending    -> :queue::queue::State/visible
;;   :queue::queue::State/in-flight  -> :queue::queue::State/unacked
;;   :in-flight                      -> :unacked
;;   :pending                        -> :visible     (sqs.wat ONLY — circuit.wat's
;;                                                    :fanout::Hist :pending is a
;;                                                    different field)
;;   symbol in-flight                -> unacked      (field binders)
;;   symbol pending                  -> visible      (sqs.wat ONLY)
;;
;; Comments are not rewritten. The semantic helper rewrites are hand-typed.
;;
;; Finder (`wat --grep`): KEYWORD leaves named `:in-flight` or
;; `:queue::queue::State/pending` or `:pending`.
;;
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/pending-to-visible.wat
;;
;; Apply (list EVERY path; sqs.wat gets the :pending / pending-symbol pass):
;;   printf '["wat-scripts/queue/sqs.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/pending-to-visible.wat
;;
;; Idempotent: after the rename the old keywords are gone.

(:wat::rete::defrule :pv::in-flight-kw
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":in-flight"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "in-flight"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :pv::state-pending-kw
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":queue::queue::State/pending"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "State/pending"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :pv::pending-kw
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":pending"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "pending-kw"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :pv))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-symbol-exact "in-flight" "unacked"
    (:wat::fix::rename-symbol-exact "pending" "visible"
      (:wat::fix::rename-keyword-exact ":pending" ":visible"
        (:wat::fix::rename-keyword-exact ":in-flight" ":unacked"
          (:wat::fix::rename-keyword-exact ":queue::queue::State/in-flight" ":queue::queue::State/unacked"
            (:wat::fix::rename-keyword-exact ":queue::queue::State/pending" ":queue::queue::State/visible"
              src)))))))

(:wat::core::defn :pv::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[pending-to-visible] " path))
        (:pv::apply-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:pv::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
