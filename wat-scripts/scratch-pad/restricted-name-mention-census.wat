;; restricted-name-mention-census.wat — CENSUS ONLY, form-aware (`wat --grep`).
;;
;; Phase 1 of docs/excursus/2026/08/001-sns-sqs/a-mention-in-a-quoted-form-is-not-a-call/DESIGN.md:
;; "does ANY site rely on a restricted name being caught via a mention that is not a call?"
;;
;; A text grep cannot answer that: it cannot tell a head-position call
;;   (:wat::kernel::write-fd-raw 1 "x")
;; from a value-position mention
;;   (:wat::core::let [f :wat::kernel::write-fd-raw] ...)
;; from a keyword sitting inside a quoted child-program template. Only the FORM TREE does.
;;
;; The nine restricted bindings in the frozen world (4 declared in wat/, 5 via
;; `#[restricted_to]` in src/): write-fd-raw, flood-stdout-raw, str-double, spawn-program,
;; IOWriter/from-fd, IOReader/from-fd, close, spawn-thread, spawn-process.
;;
;; Reports every KEYWORD node naming one of them, split by syntactic position:
;;   rc::head    — index 0 of its parent list: a head-position call.
;;   rc::nonhead — anywhere else: the surface arc 198 widened the walker to cover.
;;
;; Census:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/scratch-pad/restricted-name-mention-census.wat

(:wat::core::defrecord :rc::Restricted [id <- :wat::core::i64])

(:wat::rete::defrule :rc::name-write-fd-raw
  :when [(:wat::grep::Node  (?k <- :id) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::kernel::write-fd-raw"))]
  :then [(:rc::Restricted ?k)])

(:wat::rete::defrule :rc::name-flood-stdout-raw
  :when [(:wat::grep::Node  (?k <- :id) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::kernel::flood-stdout-raw"))]
  :then [(:rc::Restricted ?k)])

(:wat::rete::defrule :rc::name-str-double
  :when [(:wat::grep::Node  (?k <- :id) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::kernel::str-double"))]
  :then [(:rc::Restricted ?k)])

(:wat::rete::defrule :rc::name-spawn-program
  :when [(:wat::grep::Node  (?k <- :id) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::kernel::spawn-program"))]
  :then [(:rc::Restricted ?k)])

(:wat::rete::defrule :rc::name-iowriter-from-fd
  :when [(:wat::grep::Node  (?k <- :id) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::io::IOWriter/from-fd"))]
  :then [(:rc::Restricted ?k)])

(:wat::rete::defrule :rc::name-ioreader-from-fd
  :when [(:wat::grep::Node  (?k <- :id) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::io::IOReader/from-fd"))]
  :then [(:rc::Restricted ?k)])

(:wat::rete::defrule :rc::name-close
  :when [(:wat::grep::Node  (?k <- :id) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::kernel::close"))]
  :then [(:rc::Restricted ?k)])

(:wat::rete::defrule :rc::name-spawn-thread
  :when [(:wat::grep::Node  (?k <- :id) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::kernel::spawn-thread"))]
  :then [(:rc::Restricted ?k)])

(:wat::rete::defrule :rc::name-spawn-process
  :when [(:wat::grep::Node  (?k <- :id) (?kind <- :kind))
         (:wat::grep::Named (?k <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::kernel::spawn-process"))]
  :then [(:rc::Restricted ?k)])

;; ── the two reports ─────────────────────────────────────────────────────────

(:wat::rete::defrule :rc::head
  :when [(:rc::Restricted (?k <- :id))
         (:wat::grep::Node (?k <- :id) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::grep::Span (?k <- :id) (?ln <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?ln :col ?c :end-line ?el :end-col ?ec
           :rule "rc::head"
           :captures (:wat::rete::core::PersistentVector))])

(:wat::rete::defrule :rc::nonhead
  :when [(:rc::Restricted (?k <- :id))
         (:wat::grep::Node (?k <- :id) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::> ?i 0))
         (:wat::grep::Span (?k <- :id) (?ln <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?ln :col ?c :end-line ?el :end-col ?ec
           :rule "rc::nonhead"
           :captures (:wat::rete::core::PersistentVector))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :rc))
