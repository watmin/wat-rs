;; wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat — arc 255 Stone E-iii.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; BRIEF: docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-iii-set-and-list-get-their-homes.md
;;
;; Moves the two remaining junk-drawer families off `:wat::core::` to their honest homes —
;; copied from `rename-core-vectors-to-their-homes.wat` (Stone E-ii)'s rule shape, because BOTH
;; families here are slash-form ONLY, same as Vector/PersistentVector were:
;;
;;   :wat::core::HashSet/*  -> :wat::hashset::*     (the flavor-marked home — `:wat::set::`
;;                                                    stays FREE for the persistent-backed
;;                                                    sibling the builder has ruled is coming)
;;   :wat::core::List/*     -> :wat::linkedlist::*  (the flavor-marked home — `:wat::list::`
;;                                                    stays FREE the same way)
;;
;; ⛔ UNLIKE E-ii, BOTH families here take a MARKED name (neither is the unmarked
;; `PersistentVector`-style "never moves again" home) — `Arc<HashSet<Value>>` and
;; `Arc<std::collections::LinkedList<Value>>` (value.rs:340) are both the COPY-ON-WRITE flavor,
;; the same axis-side as `HashMap`/`Vector`, not the structurally-shared `rpds` side
;; `PersistentMap`/`PersistentVector` sit on.
;;
;; verbs — measured against the DISPATCH TABLE (`src/runtime.rs`'s two match tables +
;; `src/check.rs`'s `register_builtins`), NOT corpus usage (`empty?` has near-zero per-type call
;; sites the same way E-ii found for Vector/PersistentVector):
;;   HashSet: conj, contains?, empty?, length          (4 — no `get`: HashSet's "get-by-equality"
;;                                                            is `contains?`, reached only via the
;;                                                            generic `:wat::core::get` surface)
;;   List:    conj, contains?, empty?, get, length      (5 — no `concat`/`extend`)
;;
;; ⛔ Do NOT touch the bare TYPE keywords `:wat::core::HashSet` / `:wat::core::List` (no trailing
;; `/` or `::`) — `List` is arc 251's territory (`intrinsic/list.rs`), a SEPARATE future stone for
;; both (the numerics precedent's type/ops split). The two PREFIX rules' `/`-terminated
;; `starts-with?` guard cannot match a bare type keyword anyway (strictly shorter than the
;; `/`-terminated prefix). `:wat::core::List?` (the AST form-shape predicate, unrelated to the
;; `List` container despite the name) is untouched for the same structural reason — it has no
;; trailing `/`.
;;
;; ★ THE RETE-NAMESPACED SPELLING NEEDS ITS OWN RULE — ONLY FOR List, NOT HashSet. Measured
;; against `src/rete/vocabulary.rs`'s `RETE_OPS`: HashSet has NO row there at all (nothing to
;; move), but `List/get` does, so its `rete_name` ALSO moved
;; (`:wat::rete::core::List/get` -> `:wat::rete::linkedlist::get`) — and the corpus has a file
;; that spells the rete-prefixed form DIRECTLY
;; (`wat-scripts/scratch-pad/probe-brief-get-is-total-by-fallback.wat`), calling it as an ordinary
;; function rather than relying on `where`-clause auto-resolution. A blanket
;; `:wat::rete::core::List/` PREFIX rule would be WRONG: List's OTHER rete row, `first`
;; (`:wat::rete::core::List/first`), did NOT move — it is `naming_rule_tests`'s frozen
;; NAMING_RULE_EXCEPTIONS entry, sharing the ONE polymorphic `:wat::core::first` core_name across
;; three containers, and its rete_name keeps the OLD spelling forever (the corpus also spells
;; THIS one directly, in `probe-brief-first-nth-to-string.wat` — a prefix rule would sweep it up
;; as collateral damage). So the rete form is ONE EXACT-MATCH rule (`string::=`, not
;; `starts-with?`), not a prefix.
;;
;; ★ THIS IS A RULES CODEMOD, NOT A CHAR-WALK — see rename-core-vectors-to-their-homes.wat's
;; header for the fuller argument (`rename-keyword-prefix` is a silent no-op for an open,
;; `/`-terminated prefix; `wat/grep.wat`'s `Named` fact hands back the whole token).
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>     -> :user::grep  (the finder: prints every Match, unapplied)
;;   `wat` <this file>            -> :user::main  (the applier: rewrites files in place)
;;
;; Usage — finder (count before writing anything; list EVERY path across BOTH extensions this
;; stone's census found — .wat/.rs; `.rs` files need HAND edits, this tool only rewrites the
;; wat-shaped corpus, so feed it .wat/.edn/.bad paths only). The population is wat-grep's, not a
;; text grep's — a text `git grep -lE ':wat::core::(HashSet|List)[:/]'` also catches files where
;; the string appears only in a STRING LITERAL or as an unrelated `/of`-family retirement's own
;; subject matter (`rename-four-families-to-their-homes.wat`,
;; `wat-scripts/scratch-pad/probe-four-homes-census.wat` — both about `List/of`, a DIFFERENT,
;; already-finished migration — and `wat-scripts/scratch-pad/census-growing-collection-in-a-lazy-walk.wat`,
;; whose growth-verb strings are DATA, never a keyword leaf a `Named` fact can see):
;;   git grep -lE ':wat::core::(HashSet|List)[:/]' -- ':!docs' ':!*.rs' \
;;     | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat | wc -l
;;
;; Usage — apply (one EDN vector of paths on stdin — the wat-grep population PLUS the one file
;; that spells the rete form directly, which the text census above cannot see because
;; `:wat::rete::core::List/get` does not contain the substring `:wat::core::List/`):
;;   git grep -lE ':wat::core::(HashSet|List)[:/]' -- ':!docs' ':!*.rs' \
;;     | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat ./wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat
;;
;; The rewrite is comment-faithful and idempotent as a QUERY: after applying, re-running the
;; finder returns zero Match facts, because the old prefix is gone. Safe to run over the whole
;; corpus including itself: its own verb CALLS migrate along with everything else; its STRING
;; LITERAL prefixes (in the usage comments) do not, because the finder matches keyword leaves,
;; and a string literal is not a keyword leaf.

;; ── the finder — three rules over wat/grep.wat's stdlib fact base ───────────────────────

(:wat::rete::defrule :rn::core-hashset-slash
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         ;; ⚠ KEYWORD ONLY. `Named` also fires for a "string" kind (wat/grep.wat's
         ;; `nameable?`) — a string literal's span covers its surrounding quotes while its
         ;; `name` does not, so splicing the unquoted replacement into that span would corrupt
         ;; the literal into unquoted keyword syntax.
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::core::HashSet/"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "core-hashset-slash-to-hashset-colon"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::string::concat ":wat::hashset::"
                                  (:wat::rete::string::subs ?n 20
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

(:wat::rete::defrule :rn::core-list-slash
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         ;; ⚠ KEYWORD ONLY — see :rn::core-hashset-slash's comment.
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::core::List/"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "core-list-slash-to-linkedlist-colon"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new"
                         :value (:wat::rete::string::concat ":wat::linkedlist::"
                                  (:wat::rete::string::subs ?n 17
                                    (:wat::rete::string::length ?n)
                                    :undefined "")))))])

;; ── the one rete-namespaced EXACT rule — NOT a prefix rule, see header ──────────────────

(:wat::rete::defrule :rn::rete-list-get
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::rete::core::List/get"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "rete-list-get-to-rete-linkedlist-get"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value ":wat::rete::linkedlist::get")))])

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
      (:wat::kernel::println (:wat::string::concat "[core-set-and-list-to-their-homes] " path)))))

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
