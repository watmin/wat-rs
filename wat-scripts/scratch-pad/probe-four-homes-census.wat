;; wat-scripts/scratch-pad/probe-four-homes-census.wat — arc 255, the four-that-got-homes stone.
;;
;; ORCHESTRATOR'S DISCONFIRMING PROBE (FM 2-bis), not a codemod. It is the FINDER HALF ONLY of
;; the migration that follows: four rules over `wat/grep.wat`'s stdlib fact base, each matching
;; a KEYWORD LEAF by name and computing its replacement. No applier — this file never writes.
;;
;; Its job is to make the stone's acceptance bar DERIVED rather than expected
;; (`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`): the unit is
;; "keyword-leaf occurrences the rules-based finder can see", which is the only unit the
;; migration's own idempotence claim can be stated in. A `grep -c` counts lines, prose and
;; string literals; this counts what will actually be rewritten.
;;
;; Usage:
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/scratch-pad/probe-four-homes-census.wat
;;
;; ⚠ KEYWORD ONLY, per stone E's rider-found defect: `Named` fires for STRING LITERALS too, and
;; a literal's span covers its quotes while its `name` does not — splicing an unquoted name into
;; that span corrupts the literal. The kind guard is what makes the count honest as well as the
;; rewrite safe.

(:wat::rete::defrule :fhc::uuid
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::core::String/starts-with? ?n ":wat::core::Uuid/"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "uuid"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::core::String/concat ":wat::uuid::"
                                  (:wat::rete::string::subs ?n 17
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

(:wat::rete::defrule :fhc::regex
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::core::String/starts-with? ?n ":wat::core::regex::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "regex"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::core::String/concat ":wat::regex::"
                                  (:wat::rete::string::subs ?n 19
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

(:wat::rete::defrule :fhc::list-of
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::List/of"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "list-of"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::core::List")))])

(:wat::rete::defrule :fhc::char-of
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::char/of"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "char-of"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::core::char")))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :fhc))
