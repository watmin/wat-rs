;; wat-scripts/fixes/rename-core-vectors-to-their-homes.wat — arc 255 Stone E-ii.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; BRIEF: docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-ii-the-vectors-get-their-homes.md
;;
;; Moves the two vector families off the `:wat::core::` junk-drawer to their honest homes —
;; copied from `rename-core-maps-to-their-homes.wat` (Stone E-i)'s slash-form rule shape,
;; because BOTH families here are slash-form ONLY — neither has a `::`-terminated per-type verb
;; the way i64/f64/bigint/rational did:
;;
;;   :wat::core::PersistentVector/*  -> :wat::vector::*   (the UNMARKED home — PersistentVector
;;                                                          never moves again once the
;;                                                          persistent-backend swap lands)
;;   :wat::core::Vector/*            -> :wat::vec::*      (the flavor-marked home)
;;
;; verbs — measured against the corpus + `collection/eval.rs`, NOT the brief's list (which
;; omitted `empty?` for both families):
;;   PersistentVector: concat, conj, contains?, empty?, get, length            (6)
;;   Vector:           concat, conj, contains?, empty?, extend, get, length    (7 — extend only here)
;;
;; ⛔ Both flavors survive this stone — this is a SPELLING migration, not a backend decision.
;; ⛔ Do NOT touch the bare TYPE keywords `:wat::core::PersistentVector` / `:wat::core::Vector`
;; (no trailing `/` or `::`) — those are a SEPARATE future stone (the numerics precedent's
;; type/ops split), and the two PREFIX rules' `/`-terminated `starts-with?` guard cannot match
;; them anyway (a bare type keyword is strictly shorter than the `/`-terminated prefix).
;;
;; ★★ THE RETE-NAMESPACED SPELLING NEEDS ITS OWN RULES — THE THING THE MAPS STONE DID NOT HIT.
;; `src/rete/vocabulary.rs`'s naming-rule invariant means each moved verb's `rete_name` ALSO
;; moved (`:wat::rete::core::PersistentVector/length` -> `:wat::rete::vector::length`, etc.), and
;; the corpus has files that spell the rete-prefixed form DIRECTLY (`wat-scripts/perf/grid/
;; where-collection.wat` and siblings) rather than relying on `where`-clause auto-resolution. A
;; blanket `:wat::rete::core::PersistentVector/` PREFIX rule would be WRONG here: exactly THREE
;; of PersistentVector's rete rows moved (`length`, `contains?`, `get`) and ONE of Vector's
;; (`get`) — `first` did NOT move (it is `naming_rule_tests`'s frozen NAMING_RULE_EXCEPTIONS
;; entry, sharing the ONE polymorphic `:wat::core::first` core_name across three containers, and
;; its rete_name keeps the OLD `:wat::rete::core::{PersistentVector,Vector,List}/first` spelling
;; forever — a prefix rule would sweep it up as collateral damage). So the rete forms are FOUR
;; EXACT-MATCH rules (`string::=`, not `starts-with?`), one per moved row, with a fixed
;; replacement string — never a `subs`-derived one, because there is no shared prefix to strip.
;;
;; ★ THIS IS A RULES CODEMOD, NOT A CHAR-WALK. `wat/fix.wat`'s `rename-keyword-prefix` is a
;; silent no-op for an open (`::`-terminated, or here `/`-terminated) namespace prefix.
;; `wat/grep.wat`'s `Named` fact hands back ":wat::core::Vector/get" as ONE WHOLE TOKEN, so
;; there is no boundary question left to ask — a rule that matches nothing produces no Match
;; facts, countable before anything is written (`--grep` mode below).
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>     -> :user::grep  (the finder: prints every Match, unapplied)
;;   `wat` <this file>            -> :user::main  (the applier: rewrites files in place)
;;
;; Usage — finder (count before writing anything; list EVERY path across BOTH extensions this
;; stone's census found — .wat/.rs; `.rs` files need HAND edits, this tool only rewrites the
;; wat-shaped corpus, so feed it .wat/.edn/.bad paths only):
;;   git grep -lE ':wat::(rete::)?core::(PersistentVector|Vector)[:/]' -- ':!docs' ':!*.rs' \
;;     | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/rename-core-vectors-to-their-homes.wat | wc -l
;;
;; Usage — apply (one EDN vector of paths on stdin, same path list):
;;   git grep -lE ':wat::(rete::)?core::(PersistentVector|Vector)[:/]' -- ':!docs' ':!*.rs' \
;;     | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat ./wat-scripts/fixes/rename-core-vectors-to-their-homes.wat
;;
;; The rewrite is comment-faithful (rete's fact base has no notion of prose — a comment is not
;; a node, so a rule cannot touch it, by construction) and idempotent as a QUERY: after
;; applying, re-running the finder returns zero Match facts, because the old prefix is gone.
;; Safe to run over the whole corpus including itself: its own verb CALLS migrate along with
;; everything else; its STRING LITERAL prefixes (above and below, in the usage comments) do
;; not, because the finder matches keyword leaves, and a string literal is not a keyword leaf.

;; ── the finder — six rules over wat/grep.wat's stdlib fact base ─────────────────────────

(:wat::rete::defrule :rn::core-persistentvector-slash
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         ;; ⚠ KEYWORD ONLY. `Named` also fires for a "string" kind (wat/grep.wat's
         ;; `nameable?`) — a string literal's span covers its surrounding quotes while its
         ;; `name` does not, so splicing the unquoted replacement into that span would corrupt
         ;; the literal into unquoted keyword syntax. See rename-core-string-to-string.wat's
         ;; header for the fuller argument; the same trap applies here verbatim.
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::core::PersistentVector/"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "core-persistentvector-slash-to-vector-colon"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::string::concat ":wat::vector::"
                                  (:wat::rete::string::subs ?n 29
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

(:wat::rete::defrule :rn::core-vector-slash
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         ;; ⚠ KEYWORD ONLY — see :rn::core-persistentvector-slash's comment.
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::core::Vector/"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "core-vector-slash-to-vec-colon"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::string::concat ":wat::vec::"
                                  (:wat::rete::string::subs ?n 19
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

;; ── the four rete-namespaced EXACT rules — NOT prefix rules, see header ─────────────────

(:wat::rete::defrule :rn::rete-persistentvector-length
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::rete::core::PersistentVector/length"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "rete-persistentvector-length-to-rete-vector-length"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::rete::vector::length")))])

(:wat::rete::defrule :rn::rete-persistentvector-contains
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::rete::core::PersistentVector/contains?"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "rete-persistentvector-contains-to-rete-vector-contains"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::rete::vector::contains?")))])

(:wat::rete::defrule :rn::rete-persistentvector-get
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::rete::core::PersistentVector/get"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "rete-persistentvector-get-to-rete-vector-get"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::rete::vector::get")))])

(:wat::rete::defrule :rn::rete-vector-get
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::rete::core::Vector/get"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "rete-vector-get-to-rete-vec-get"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::rete::vec::get")))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :rn))

;; ── the applier's own query — field-destructured, NOT wat-grep's whole-record q-match,
;; because the applier needs the Span fields alongside the captures to compute an edit. ────

(:wat::rete::defquery :rn::q-match
  :params []
  :when [(:wat::grep::Match (?line <- :line) (?col <- :col)
           (?end-line <- :end-line) (?end-col <- :end-col) (?captures <- :captures))])

;; second-capture — a typed wrapper around `second`. `PersistentVector/get`'s value type is a
;; fresh metavariable until something FORCES it concrete; a Tuple-constructor slot does that
;; by checking-mode unification (the template's own `?offset`/`?len` extraction), but `second`
;; inspects its argument's container shape at INFER time and cannot defer — so the argument
;; must already be concretely typed before it gets there. An explicit-signature wrapper does
;; that: the call site's argument is checked against this fn's declared param type first.
(:wat::core::defn :rn::second-capture
  [captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::second captures))

;; first-capture — the "old" capture: the rule's CLAIM about what text sits at this match's
;; span. Typed the same way second-capture is, for the same INFER-vs-checking-mode reason.
(:wat::core::defn :rn::first-capture
  [captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::first captures))

;; edits-of — query rows -> Vector of Tuple(offset, old-text, new-text), UNSORTED.
;; `?captures` is (old, new) in that fixed order (this file's own :then above). old-text is
;; the FIRST capture's value — the rule's belief, not a slice of src at `offset`.
(:wat::core::defn :rn::edits-of
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
         old-text (:wat::grep::Capture/value (:rn::first-capture captures))
         new-text (:wat::grep::Capture/value (:rn::second-capture captures))
         start    {:line line     :col col}
         offset   (:wat::fix::fix-text-offset-of start lines)]
        (:wat::core::concat a
          (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
            (:wat::core::Tuple offset old-text new-text)))))
    acc rows))

;; convert-one — one file, through the already-compiled network via `overlay` (mirrors
;; wat/grep.wat's run-one, but applies edits instead of printing matches).
(:wat::core::defn :rn::convert-one
  [overlay <- :wat::rete::Overlay
   path    <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let
    [src     (:wat::io::read-file path)
     lines   (:wat::string::split src "\n")
     facts   (:wat::grep::facts-of path src)
     records (:wat::grep::facts-as-records facts)
     fired   (overlay records)
     rows    (:wat::rete::query fired (:rn::q-match))
     empty-e (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
     edits   (:rn::edits-of rows lines empty-e)
     ;; ★ SORT DESCENDING BY OFFSET — fix-text-apply splices right-to-left; rete returns query
     ;; results in NETWORK order, not source order (to-faithful-clojure-net.wat:275's comparator).
     sorted  (:wat::core::sort
               (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                                b <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                 -> :wat::core::bool
                 (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
               edits)
     out     (:wat::fix::fix-text-apply src sorted)]
    (:wat::core::do
      (:wat::io::write-file path out)
      (:wat::kernel::println (:wat::string::concat "[core-vectors-to-their-homes] " path)))))

(:wat::core::defn :rn::convert-each
  [overlay <- :wat::rete::Overlay
   paths   <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:rn::convert-one overlay (:wat::core::first paths))
      (:rn::convert-each overlay (:wat::core::rest paths)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
             (:wat::kernel::ReadlnOutcome::Eof
               (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
             (:wat::kernel::ReadlnOutcome::Stopped
               (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    (:wat::rete::with-overlay (:wat::rete::collect-rules :rn)
      (:wat::core::PersistentVector :- [:wat::rete::Query] (:rn::q-match))
      (:wat::core::fn [overlay <- :wat::rete::Overlay] -> :wat::core::nil
        (:rn::convert-each overlay paths)))))
