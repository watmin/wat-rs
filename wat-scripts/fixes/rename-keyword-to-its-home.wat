;; wat-scripts/fixes/rename-keyword-to-its-home.wat — arc 255 Stone E-iv.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; BRIEF: docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-iv-keyword-gets-its-home.md
;; PRIOR ART: wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat (Stone E-iii) — this
;; file copies its rule shape.
;;
;; Moves the LAST scalar without a home off `:wat::core::` to its honest home:
;;
;;   :wat::core::keyword/*  -> :wat::keyword::*   (the PLAIN, unmarked home — `keyword` has only
;;                                                  ONE flavor, unlike E-iii's hashset/linkedlist,
;;                                                  so there is no marked/unmarked question and
;;                                                  nothing is reserved against the plain name)
;;
;; verbs — measured against the DISPATCH TABLE (`src/runtime.rs`'s `dispatch_keyword_head` +
;; `dispatch_keyword_head_value`, `src/edn/render.rs`, `src/check.rs`'s `register_builtins`), NOT
;; corpus usage (E-ii's lesson: a migration census cannot see a verb nobody calls, and an unseen
;; verb strands at retirement):
;;   keyword: from-string, to-string, to-symbol, to-type-form, to-type-form-colon   (5 — ALL of
;;            them share the one `:wat::core::keyword/` prefix, so ONE rule covers the family;
;;            unlike HashSet/List there is no second family sharing this file)
;;
;; ⛔ Do NOT touch the bare TYPE keyword `:wat::core::keyword` (no trailing `/`) — arc 251's
;; `wat.type/keyword` territory, a separate concern (the numerics precedent's type/ops split).
;; The prefix rule's `/`-terminated `starts-with?` guard cannot match a bare type keyword anyway
;; (strictly shorter than the `/`-terminated prefix). `:wat::core::keyword-node` (the AST
;; constructor) and `:wat::core::keyword::=`/`not=` / `:wat::rete::core::keyword::=`/`not=` (the
;; generic-equality alias rows, core_name `:wat::core::=`/`not=`, an UNRELATED family) are also
;; untouched for the same structural reason — none of them has this rule's exact `/`-terminated
;; prefix.
;;
;; ★ NO RETE-NAMESPACED RULE NEEDED — measured, and it is the one place this brief's own ground
;; table is WRONG. `src/rete/vocabulary.rs`'s RETE_OPS has ZERO rows whose `core_name` is any of
;; the 5 verbs above (verified: no row's `core_name` starts with `:wat::core::keyword/` at all).
;; The `:wat::rete::core::keyword::=`/`not=` rows that make the naive substring "keyword" appear
;; near "core::" in that file are the UNRELATED generic-equality alias (`core_name:
;; :wat::core::=`/`not=`) — an accident of the disambiguating middle segment, not evidence of a
;; forced row. `naming_rule_tests` stays 5/5 with ZERO new `NAMING_RULE_EXCEPTIONS` because
;; nothing here renames a `core_name` any RETE_OPS row depends on. The two scratch-pad files that
;; spell a rete form directly (`probe-cond-rete-scorecard.wat`, `probe-cond-rete-where.wat`) use
;; ONLY `:wat::rete::core::keyword::=` (that same unrelated equality alias) — grepped for
;; `:wat::rete::core::keyword/` across the whole corpus: zero hits. Neither file needs an edit,
;; and both are re-run after the codemod anyway as a control.
;;
;; ★ THIS IS A RULES CODEMOD, NOT A CHAR-WALK — see rename-core-vectors-to-their-homes.wat's
;; header for the fuller argument (`rename-keyword-prefix` is a silent no-op for an open,
;; `/`-terminated prefix; `wat/grep.wat`'s `Named` fact hands back the whole token).
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>     -> :user::grep  (the finder: prints every Match, unapplied)
;;   `wat` <this file>            -> :user::main  (the applier: rewrites files in place)
;;
;; Usage — finder (count before writing anything; the population is wat-grep's, not a text
;; grep's — a text `git grep -lE ':wat::core::keyword[:/]'` also catches files where the string
;; appears only in a STRING LITERAL, a comment, or the UNRELATED bare-type/`keyword-node`/
;; equality-alias spellings a keyword-leaf structural match will correctly skip):
;;   git grep -lE ':wat::core::keyword/' -- ':!docs' ':!*.rs' \
;;     | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/rename-keyword-to-its-home.wat | wc -l
;;
;; Usage — apply (one EDN vector of paths on stdin — the wat-grep population; `.rs` files need
;; HAND edits, this tool only rewrites the wat-shaped corpus):
;;   git grep -lE ':wat::core::keyword/' -- ':!docs' ':!*.rs' \
;;     | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat ./wat-scripts/fixes/rename-keyword-to-its-home.wat
;;
;; The rewrite is comment-faithful and idempotent as a QUERY: after applying, re-running the
;; finder returns zero Match facts, because the old prefix is gone. Safe to run over the whole
;; corpus including itself: its own verb CALLS migrate along with everything else; its STRING
;; LITERAL prefixes (in the usage comments) do not, because the finder matches keyword leaves,
;; and a string literal is not a keyword leaf.

;; ── the finder — one rule over wat/grep.wat's stdlib fact base ──────────────────────────

(:wat::rete::defrule :rn::core-keyword-slash
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         ;; ⚠ KEYWORD ONLY. `Named` also fires for a "string" kind (wat/grep.wat's
         ;; `nameable?`) — a string literal's span covers its surrounding quotes while its
         ;; `name` does not, so splicing the unquoted replacement into that span would corrupt
         ;; the literal into unquoted keyword syntax.
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::core::keyword/"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "core-keyword-slash-to-keyword-colon"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::string::concat ":wat::keyword::"
                                  (:wat::rete::string::subs ?n 20
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

;; second-capture — a typed wrapper around `second`. See rename-core-vectors-to-their-homes.wat's
;; header comment for why this needs an explicit signature (INFER-vs-checking-mode).
(:wat::core::defn :rn::second-capture
  [captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::second captures))

;; first-capture — the "old" capture: the rule's CLAIM about what text sits at this match's span.
(:wat::core::defn :rn::first-capture
  [captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::first captures))

;; edits-of — query rows -> Vector of Tuple(offset, old-text, new-text), UNSORTED.
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

;; convert-one — one file, through the already-compiled network via `overlay`.
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
      (:wat::kernel::println (:wat::string::concat "[rename-keyword-to-its-home] " path)))))

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
