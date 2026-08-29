;; wat-scripts/fixes/rename-rete-numerics-to-their-homes.wat — arc 255 Stone B-ii.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-B-ii-the-rete-clone.md
;;
;; Moves the rete DSL's per-type numeric surface off `rete::core::` to its honest home, mirroring
;; `rename-core-numerics-to-their-homes.wat` (Stone B-i) one namespace segment over — copied as
;; the shape, only the prefixes and the `subs` offsets change (23 chars, not 17):
;;   :wat::rete::core::i64::*   -> :wat::rete::i64::*
;;   :wat::rete::core::f64::*   -> :wat::rete::f64::*
;;
;; This corpus-only rename is paired with a `src/rete/vocabulary.rs` edit that is NOT made by
;; this file: each numeric row's `core_name` moves from `:wat::core::i64::*` to `:wat::i64::*`
;; (B-i's corpus target), and the naming rule (`rete_name = core_name` with `rete::` inserted
;; after `wat::`) DERIVES the paired `rete_name` — `:wat::rete::i64::*` — from that. This file
;; only makes the `.wat` corpus's own call sites match the derived spelling.
;;
;; ⛔ NO SHAPE-REWRITING RULE. `max-of`/`min-of` have no rete row at all (measured: the census
;; below found zero `max-of`/`min-of` matches under either rete prefix) — a pure prefix rewrite
;; is correct for every site this file touches. If `--grep` ever surfaces one, STOP — the census
;; was wrong and that site needs a shape change, not this rule.
;;
;; ⚠ NO FILES EXCLUDED. Unlike B-i (which had to omit its own two dual-spelling probes from the
;; applier's path list), no `.wat` file in this corpus asserts BOTH rete spellings at once
;; (measured: zero scratch-pad files carrying `:wat::rete::i64::`/`:wat::rete::f64::` alongside
;; the old prefix) — every matched site is a pure one-way rewrite.
;;
;; The trailing `::` is the entire discrimination that keeps this off the rete surface's own
;; TYPE keywords and off anything shorter than the `::`-terminated prefix this rule matches —
;; same argument as B-i's header; do not weaken it.
;;
;; ★ THIS IS A RULES CODEMOD, NOT A CHAR-WALK. `wat/fix.wat`'s `rename-keyword-prefix` is a
;; silent no-op for an open (`::`-terminated) namespace prefix; `wat/grep.wat`'s `Named` fact
;; hands back `":wat::rete::core::i64::+"` as ONE WHOLE TOKEN, so there is no boundary question
;; left to ask — a rule that matches nothing produces no Match facts, countable before anything
;; is written (`--grep` mode below).
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>     -> :user::grep  (the finder: prints every Match, unapplied)
;;   `wat` <this file>            -> :user::main  (the applier: rewrites files in place)
;;
;; Usage — finder (count before writing anything):
;;   git ls-files '*.wat' \
;;     | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/rename-rete-numerics-to-their-homes.wat | wc -l
;;
;; Usage — apply (one EDN vector of paths on stdin):
;;   git ls-files '*.wat' \
;;     | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat ./wat-scripts/fixes/rename-rete-numerics-to-their-homes.wat
;;
;; The rewrite is comment-faithful and idempotent as a QUERY: after applying, re-running the
;; finder returns zero Match facts, because the old prefix is gone. Safe to run over the whole
;; corpus including itself: its own verb CALLS migrate along with everything else; its STRING
;; LITERAL prefixes (in the usage comments and the KEYWORD-ONLY guard) do not, because the
;; finder matches keyword leaves, and a string literal is not a keyword leaf.

;; ── the finder — two rules over wat/grep.wat's stdlib fact base ─────────────────────────

(:wat::rete::defrule :rn::rete-i64
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         ;; ⚠ KEYWORD ONLY. `Named` also fires for a "string" kind (wat/grep.wat's
         ;; `nameable?`) — a string literal's span covers its surrounding quotes while its
         ;; `name` does not, so splicing the unquoted replacement into that span would corrupt
         ;; the literal into unquoted keyword syntax. See `rename-core-numerics-to-their-
         ;; homes.wat`'s header for the fuller argument; the same trap applies here verbatim.
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::rete::core::i64::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "rete-core-i64-to-rete-i64"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::string::concat ":wat::rete::i64::"
                                  (:wat::rete::string::subs ?n 23
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

(:wat::rete::defrule :rn::rete-f64
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         ;; ⚠ KEYWORD ONLY — see :rn::rete-i64's comment.
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::rete::core::f64::"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "rete-core-f64-to-rete-f64"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::string::concat ":wat::rete::f64::"
                                  (:wat::rete::string::subs ?n 23
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :rn))

;; ── the applier's own query — field-destructured, NOT wat-grep's whole-record q-match,
;; because the applier needs the Span fields alongside the captures to compute an edit. ────

(:wat::rete::defquery :rn::q-match
  :params []
  :when [(:wat::grep::Match (?line <- :line) (?col <- :col)
           (?end-line <- :end-line) (?end-col <- :end-col) (?captures <- :captures))])

;; second-capture — a typed wrapper around `second`. `PersistentMap/get`'s value type is a
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
      (:wat::kernel::println (:wat::string::concat "[rete-numerics-to-their-homes] " path)))))

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
