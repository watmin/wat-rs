;; wat-scripts/fixes/rename-math-stat-seq-to-their-homes.wat — arc 255 Stone HOME-9,
;; ":wat::std::" finally dies.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; BRIEF: docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-HOME-9-the-std-namespace-finally-dies.md
;; PRIOR ART: wat-scripts/fixes/rename-four-families-to-their-homes.wat (Stone "the four that
;; got homes") — this file copies its rule shape (a prefix rule per namespace-wide move, an
;; exact-match rule per single-name move).
;;
;; Moves ELEVEN of the fourteen `:wat::std::` verbs off the junk-drawer namespace arc 109 was
;; supposed to annihilate four months ago:
;;
;;   :wat::std::math::{ln exp sqrt sin cos}  -> :wat::math::*     (prefix rule)
;;   :wat::std::math::pi                     -> :wat::math::pi    (caught by the same prefix rule)
;;   :wat::std::stat::{mean variance stddev} -> :wat::stat::*     (prefix rule)
;;   :wat::std::list::zip                    -> :wat::seq::zip    (exact-match rule)
;;   :wat::std::list::window                 -> :wat::seq::window (exact-match rule)
;;   :wat::std::list::remove-at              -> :wat::seq::remove-at (exact-match rule)
;;
;; THREE VERBS ARE DELIBERATELY NOT HERE:
;;   :wat::std::math::log        — a level-1 lie (wired to f64::ln, zero call sites) — DELETED,
;;                                  not moved. No codemod rule: nothing to rewrite TO.
;;   :wat::std::list::map-with-index — DELETED. Its replacement, `:wat::core::map-indexed`, is
;;                                  NOT a drop-in (arg order flips (Vector,fn)->(fn,coll); the
;;                                  result is a lazy Stream, not an eager Vector) — its one real
;;                                  caller (wat/holon/Sequential.wat) and its direct unit-test
;;                                  fixtures are migrated BY HAND, deliberately, per-caller. A
;;                                  mechanical/codemod rewrite here would compile and be wrong
;;                                  (BRIEF STOP-2). Do NOT add an exact-match rule for it.
;;
;; ⚠ `:wat::std::list::` is a SHARED prefix with `map-with-index` — a blanket prefix rule
;; `:wat::std::list -> :wat::seq` would also rewrite `map-with-index` into
;; `:wat::seq::map-with-index`, which is WRONG (that verb dies, it does not move). Hence three
;; separate EXACT-match rules for zip/window/remove-at instead of one prefix rule, mirroring
;; `rename-four-families-to-their-homes.wat`'s `list-of`/`char-of` exact-match shape.
;;
;; ★ THIS IS A RULES CODEMOD, NOT A CHAR-WALK. The reader already tokenized every file;
;; `wat/grep.wat`'s `Named` fact hands back a keyword leaf as ONE WHOLE TOKEN, so there is no
;; boundary question left to ask.
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>     -> :user::grep  (the finder: prints every Match, unapplied)
;;   `wat` <this file>            -> :user::main  (the applier: rewrites files in place)
;;
;; ⚠ PATH LIST IS `*.wat` ONLY (including `tests/**/*.wat` fixtures, which ARE `.wat` files).
;; `.rs` files (dispatch arms, embedded test literals, check.rs registrations) move BY HAND —
;; the kind guard below excludes string literals and comments by construction, so running this
;; against a `.rs` file returns zero matches, silently.
;;
;; Usage — finder (count before writing anything; population from wat-grep, not text grep —
;; the brief's own per-verb text counts were contaminated by docs/):
;;   git ls-files '*.wat' '*.wat.bad' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/rename-math-stat-seq-to-their-homes.wat | wc -l
;;
;; Usage — apply (one EDN vector of paths on stdin):
;;   git ls-files '*.wat' '*.wat.bad' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat ./wat-scripts/fixes/rename-math-stat-seq-to-their-homes.wat
;;
;; The rewrite is comment-faithful and idempotent as a QUERY: after applying, re-running the
;; finder returns zero Match facts, because the old names are gone.

;; ── the finder — five rules over wat/grep.wat's stdlib fact base ───────────────────────────

(:wat::rete::defrule :hms::math
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         ;; ⚠ KEYWORD ONLY. `Named` also fires for a "string" kind — a string literal's span
         ;; covers its surrounding quotes while its `name` does not, so splicing the unquoted
         ;; replacement into that span would corrupt the literal into unquoted keyword syntax.
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::std::math::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "math"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::string::concat ":wat::math::"
                                  (:wat::rete::string::subs ?n 17
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

(:wat::rete::defrule :hms::stat
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::std::stat::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "stat"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::string::concat ":wat::stat::"
                                  (:wat::rete::string::subs ?n 17
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

(:wat::rete::defrule :hms::zip
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::std::list::zip"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "zip"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::seq::zip")))])

(:wat::rete::defrule :hms::window
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::std::list::window"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "window"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::seq::window")))])

(:wat::rete::defrule :hms::remove-at
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::std::list::remove-at"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "remove-at"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::seq::remove-at")))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :hms))

;; ── the applier's own query — field-destructured, NOT wat-grep's whole-record q-match,
;; because the applier needs the Span fields alongside the captures to compute an edit. ────

(:wat::rete::defquery :hms::q-match
  :params []
  :when [(:wat::grep::Match (?line <- :line) (?col <- :col)
           (?end-line <- :end-line) (?end-col <- :end-col) (?captures <- :captures))])

(:wat::core::defn :hms::second-capture
  [captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::second captures))

(:wat::core::defn :hms::first-capture
  [captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::first captures))

;; edits-of — query rows -> Vector of Tuple(offset, old-text, new-text), UNSORTED.
(:wat::core::defn :hms::edits-of
  [rows  <- :wat::core::PersistentVector
   lines <- (:wat::core::Vector :- [:wat::core::String])
   acc   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [a   <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     row <- :wat::core::PersistentMap]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::let
        [line     (:wat::core::Option/expect (:wat::map::get row "?line")     "q-match: ?line")
         col      (:wat::core::Option/expect (:wat::map::get row "?col")      "q-match: ?col")
         captures (:wat::core::Option/expect (:wat::map::get row "?captures") "q-match: ?captures")
         old-text (:wat::grep::Capture/value (:hms::first-capture captures))
         new-text (:wat::grep::Capture/value (:hms::second-capture captures))
         start    {:line line     :col col}
         offset   (:wat::fix::fix-text-offset-of start lines)]
        (:wat::core::concat a
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
            (:wat::core::Tuple offset old-text new-text)))))
    acc rows))

(:wat::core::defn :hms::convert-one
  [overlay <- :wat::rete::Overlay
   path    <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let
    [src     (:wat::io::read-file path)
     lines   (:wat::string::split src "\n")
     facts   (:wat::grep::facts-of path src)
     records (:wat::grep::facts-as-records facts)
     fired   (overlay records)
     rows    (:wat::rete::query fired (:hms::q-match))
     empty-e (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
     edits   (:hms::edits-of rows lines empty-e)
     ;; ★ SORT DESCENDING BY OFFSET — fix-text-apply splices right-to-left.
     sorted  (:wat::core::sort
               (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                                b <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                 -> :wat::core::bool
                 (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
               edits)
     out     (:wat::fix::fix-text-apply src sorted)]
    (:wat::core::do
      (:wat::io::write-file path out)
      (:wat::kernel::println (:wat::string::concat "[rename-math-stat-seq-to-their-homes] " path)))))

(:wat::core::defn :hms::convert-each
  [overlay <- :wat::rete::Overlay
   paths   <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:hms::convert-one overlay (:wat::core::first paths))
      (:hms::convert-each overlay (:wat::core::rest paths)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
             (:wat::kernel::ReadlnOutcome::Eof
               (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
             (:wat::kernel::ReadlnOutcome::Stopped
               (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    (:wat::rete::with-overlay (:wat::rete::collect-rules :hms)
      (:wat::core::PersistentVector :- [:wat::rete::Query] (:hms::q-match))
      (:wat::core::fn [overlay <- :wat::rete::Overlay] -> :wat::core::nil
        (:hms::convert-each overlay paths)))))
