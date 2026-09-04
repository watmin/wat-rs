;; wat-scripts/fixes/vocabulary-stops-mumbling.wat — Stone D2: the vocabulary stops mumbling.
;;
;; Self-hosted, comment-faithful fix-wat codemod. The test-helper vocabulary
;; lied about what it did. Mechanical whole-token keyword renames:
;;
;;   :fanout::accept!              -> :fanout::publish-stamped-until-accepted!
;;   :fanout::accept-stamped       -> :fanout::publish-until-accepted!
;;   :demo::accept!                -> :demo::publish-until-accepted!
;;   :fanout::face-start           -> :fanout::start-worker!
;;   :demo::face-start-tw          -> :demo::start-topic-worker!
;;   :fanout::nap-ms               -> :fanout::await-timer-ms
;;   :demo::nap-ms                 -> :demo::await-timer-ms
;;   :user::nap-ms                 -> :user::await-timer-ms
;;   :vr::nap-ms                   -> :vr::await-timer-ms
;;   :vw::nap-ms                   -> :vw::await-timer-ms
;;   :vb::nap-ms                   -> :vb::await-timer-ms
;;   :user::do-stats               -> :user::read-call-counters
;;   :user::do-depth               -> :user::read-queue-counts
;;   :user::do-send                -> :user::send
;;   :user::do-receive-wait        -> :user::receive-wait
;;   :user::do-receive             -> :user::receive
;;   :user::do-ack                 -> :user::ack
;;
;; Exact whole-token (rename-keyword-exact), not prefix: `:user::do-receive`
;; must not eat `:user::do-receive-wait`, and `:fanout::face-start` must not
;; eat `:demo::face-start-tw`.
;;
;; The accept! liveness bound and the face-start WHY are HAND work after this
;; rewrite — this file only moves names. Comments are not rewritten.
;;
;; Finder (`wat --grep`): every KEYWORD leaf whose full name is one of the
;; old tokens. Count occurrences, not lines.
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>  -> :user::grep  (prints every Match, unapplied)
;;   `wat` <this file>         -> :user::main  (rewrites files in place)
;;
;; Usage — finder:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/vocabulary-stops-mumbling.wat
;;
;; Usage — dry-run (copy, then apply, then diff):
;;   cp <file> /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/vocabulary-stops-mumbling.wat
;;   diff <file> /tmp/pilot.wat
;;
;; Usage — apply (list EVERY path the finder named):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/vocabulary-stops-mumbling.wat
;;
;; Idempotent: after a site is rewritten its keyword is no longer an old
;; name, so a re-run emits 0 edits.

;; ── finder ───────────────────────────────────────────────────────────────────

(:wat::rete::defrule :d2::fanout-accept
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":fanout::accept!"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "fanout-accept"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::fanout-accept-stamped
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":fanout::accept-stamped"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "fanout-accept-stamped"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::demo-accept
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":demo::accept!"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "demo-accept"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::fanout-face-start
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":fanout::face-start"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "fanout-face-start"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::demo-face-start-tw
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":demo::face-start-tw"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "demo-face-start-tw"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::fanout-nap-ms
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":fanout::nap-ms"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "fanout-nap-ms"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::demo-nap-ms
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":demo::nap-ms"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "demo-nap-ms"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::user-nap-ms
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":user::nap-ms"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "user-nap-ms"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::vr-nap-ms
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":vr::nap-ms"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "vr-nap-ms"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::vw-nap-ms
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":vw::nap-ms"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "vw-nap-ms"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::vb-nap-ms
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":vb::nap-ms"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "vb-nap-ms"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::do-stats
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":user::do-stats"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "do-stats"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::do-depth
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":user::do-depth"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "do-depth"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::do-send
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":user::do-send"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "do-send"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::do-receive-wait
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":user::do-receive-wait"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "do-receive-wait"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::do-receive
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":user::do-receive"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "do-receive"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::rete::defrule :d2::do-ack
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":user::do-ack"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "do-ack"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :d2))

;; ── applier ──────────────────────────────────────────────────────────────────

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":user::do-ack" ":user::ack"
    (:wat::fix::rename-keyword-exact ":user::do-receive" ":user::receive"
      (:wat::fix::rename-keyword-exact ":user::do-receive-wait" ":user::receive-wait"
        (:wat::fix::rename-keyword-exact ":user::do-send" ":user::send"
          (:wat::fix::rename-keyword-exact ":user::do-depth" ":user::read-queue-counts"
            (:wat::fix::rename-keyword-exact ":user::do-stats" ":user::read-call-counters"
              (:wat::fix::rename-keyword-exact ":vb::nap-ms" ":vb::await-timer-ms"
                (:wat::fix::rename-keyword-exact ":vw::nap-ms" ":vw::await-timer-ms"
                  (:wat::fix::rename-keyword-exact ":vr::nap-ms" ":vr::await-timer-ms"
                    (:wat::fix::rename-keyword-exact ":user::nap-ms" ":user::await-timer-ms"
                      (:wat::fix::rename-keyword-exact ":demo::nap-ms" ":demo::await-timer-ms"
                        (:wat::fix::rename-keyword-exact ":fanout::nap-ms" ":fanout::await-timer-ms"
                          (:wat::fix::rename-keyword-exact ":demo::face-start-tw" ":demo::start-topic-worker!"
                            (:wat::fix::rename-keyword-exact ":fanout::face-start" ":fanout::start-worker!"
                              (:wat::fix::rename-keyword-exact ":demo::accept!" ":demo::publish-until-accepted!"
                                (:wat::fix::rename-keyword-exact ":fanout::accept-stamped" ":fanout::publish-until-accepted!"
                                  (:wat::fix::rename-keyword-exact ":fanout::accept!" ":fanout::publish-stamped-until-accepted!"
                                    src))))))))))))))))))

(:wat::core::defn :d2::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[vocabulary-stops-mumbling] " path))
        (:d2::apply-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:d2::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
