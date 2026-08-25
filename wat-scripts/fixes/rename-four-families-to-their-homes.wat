;; wat-scripts/fixes/rename-four-families-to-their-homes.wat — arc 255, "the four that got homes".
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-four-that-got-homes-they-had-not-earned.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-the-four-that-got-homes.md
;;
;; Moves ten Rust-implemented verbs off the `:wat::core::` junk-drawer to the namespace each
;; earns:
;;   :wat::core::Uuid/v4               -> :wat::uuid::v4
;;   :wat::core::Uuid/v5               -> :wat::uuid::v5
;;   :wat::core::Uuid/from-string      -> :wat::uuid::from-string
;;   :wat::core::Uuid/to-string        -> :wat::uuid::to-string
;;   :wat::core::Uuid/nil              -> :wat::uuid::nil
;;   :wat::core::Uuid/version          -> :wat::uuid::version
;;   :wat::core::Uuid/rfc4122-variant? -> :wat::uuid::rfc4122-variant?
;;   :wat::core::regex::matches?       -> :wat::regex::matches?
;;   :wat::core::List/of               -> :wat::core::List      (finishing, not starting)
;;   :wat::core::char/of               -> :wat::core::char      (finishing, not starting)
;;
;; Same shape as `rename-core-string-to-string.wat` (stone E, AS RULES) — four rules over
;; `wat/grep.wat`'s stdlib fact base instead of two, one entry point pair, one applier.
;;
;; ★ THIS IS A RULES CODEMOD, NOT A CHAR-WALK. The reader already tokenized every file;
;; `wat/grep.wat`'s `Named` fact hands back a keyword leaf as ONE WHOLE TOKEN, so there is no
;; boundary question left to ask — a rule that matches nothing produces no Match facts,
;; countable before anything is written (`--grep` mode below).
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>     -> :user::grep  (the finder: prints every Match, unapplied)
;;   `wat` <this file>            -> :user::main  (the applier: rewrites files in place)
;;
;; ⚠ PATH LIST IS `*.wat` ONLY. Every one of these ten names also lives inside a Rust string
;; literal (the `#[wat_intrinsic("…")]` registrations and friends) — the kind guard below
;; excludes string literals by construction, so run against a `.rs` file this finder returns
;; zero matches, silently. The `.rs` side moves by hand; see the BRIEF's Act 1.
;;
;; Usage — finder (count before writing anything):
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/rename-four-families-to-their-homes.wat | wc -l
;;
;; Usage — apply (one EDN vector of paths on stdin):
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat ./wat-scripts/fixes/rename-four-families-to-their-homes.wat
;;
;; The rewrite is comment-faithful (rete's fact base has no notion of prose — a comment is not
;; a node, so a rule cannot touch it, by construction) and idempotent as a QUERY: after
;; applying, re-running the finder returns zero Match facts, because the old names are gone.

;; ── the finder — four rules over wat/grep.wat's stdlib fact base ────────────────────────────
;; Lifted verbatim from wat-scripts/scratch-pad/probe-four-homes-census.wat (committed at
;; 61dd04a3b), the FINDER HALF already proven against the live corpus (239/239).

(:wat::rete::defrule :fhc::uuid
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         ;; ⚠ KEYWORD ONLY, per stone E's rider-found defect: `Named` fires for STRING
         ;; LITERALS too, and a literal's span covers its quotes while its `name` does not.
         ;; Measured for these ten names: zero genuine string-literal occurrences in the .wat
         ;; corpus, so the guard changes no outcome here — kept anyway, it is what makes the
         ;; count honest as well as the rewrite safe.
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

;; ── the applier's own query — field-destructured, NOT wat-grep's whole-record q-match,
;; because the applier needs the Span fields alongside the captures to compute an edit. ────

(:wat::rete::defquery :fhc::q-match
  :params []
  :when [(:wat::grep::Match (?line <- :line) (?col <- :col)
           (?end-line <- :end-line) (?end-col <- :end-col) (?captures <- :captures))])

;; second-capture — a typed wrapper around `second`. `PersistentMap/get`'s value type is a
;; fresh metavariable until something FORCES it concrete; a Tuple-constructor slot does that
;; by checking-mode unification (the template's own `?offset`/`?len` extraction), but `second`
;; inspects its argument's container shape at INFER time and cannot defer — so the argument
;; must already be concretely typed before it gets there. An explicit-signature wrapper does
;; that: the call site's argument is checked against this fn's declared param type first.
(:wat::core::defn :fhc::second-capture
  [captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::second captures))

;; first-capture — the "old" capture: the rule's CLAIM about what text sits at this match's
;; span (arc 282). Captured in :then above and, until that stone, never read — the belief was
;; thrown away and old-len was derived from the span instead, which would have compared a slice
;; against itself. Typed the same way second-capture is, for the same INFER-vs-checking-mode
;; reason (PersistentMap/get's value type needs a concrete argument type to force it).
(:wat::core::defn :fhc::first-capture
  [captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::first captures))

;; edits-of — query rows -> Vector of Tuple(offset, old-text, new-text), UNSORTED.
;; `?captures` is (old, new) in that fixed order (this file's own :then above). old-text is the
;; FIRST capture's value — the rule's belief — never a slice of src at `offset`.
(:wat::core::defn :fhc::edits-of
  [rows  <- :wat::core::PersistentVector
   lines <- (:wat::core::Vector :- [:wat::core::String])
   acc   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [a   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     row <- :wat::core::PersistentMap]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::let
        [line     (:wat::core::Option/expect (:wat::core::PersistentMap/get row "?line")     "q-match: ?line")
         col      (:wat::core::Option/expect (:wat::core::PersistentMap/get row "?col")      "q-match: ?col")
         captures (:wat::core::Option/expect (:wat::core::PersistentMap/get row "?captures") "q-match: ?captures")
         old-text (:wat::grep::Capture/value (:fhc::first-capture captures))
         new-text (:wat::grep::Capture/value (:fhc::second-capture captures))
         start    {:line line     :col col}
         offset   (:wat::fix::fix-text-offset-of start lines)]
        (:wat::core::concat a
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
            (:wat::core::Tuple offset old-text new-text)))))
    acc rows))

;; convert-one — one file, through the already-compiled network via `overlay` (mirrors
;; wat/grep.wat's run-one, but applies edits instead of printing matches).
(:wat::core::defn :fhc::convert-one
  [overlay <- :wat::rete::Overlay
   path    <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let
    [src     (:wat::io::read-file path)
     lines   (:wat::string::split src "\n")
     facts   (:wat::grep::facts-of path src)
     records (:wat::grep::facts-as-records facts)
     fired   (overlay records)
     rows    (:wat::rete::query fired (:fhc::q-match))
     empty-e (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
     edits   (:fhc::edits-of rows lines empty-e)
     ;; ★ SORT DESCENDING BY OFFSET — fix-text-apply splices right-to-left; rete returns query
     ;; results in NETWORK order, not source order.
     sorted  (:wat::core::sort
               (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                                b <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                 -> :wat::core::bool
                 (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
               edits)
     out     (:wat::fix::fix-text-apply src sorted)]
    (:wat::core::do
      (:wat::io::write-file path out)
      (:wat::kernel::println (:wat::string::concat "[rename-four-families-to-their-homes] " path)))))

(:wat::core::defn :fhc::convert-each
  [overlay <- :wat::rete::Overlay
   paths   <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:fhc::convert-one overlay (:wat::core::first paths))
      (:fhc::convert-each overlay (:wat::core::rest paths)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
             (:wat::kernel::ReadlnOutcome::Eof
               (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
             (:wat::kernel::ReadlnOutcome::Stopped
               (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    (:wat::rete::with-overlay (:wat::rete::collect-rules :fhc)
      (:wat::core::PersistentVector :- [:wat::rete::Query] (:fhc::q-match))
      (:wat::core::fn [overlay <- :wat::rete::Overlay] -> :wat::core::nil
        (:fhc::convert-each overlay paths)))))
