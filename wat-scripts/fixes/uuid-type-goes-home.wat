;; wat-scripts/fixes/uuid-type-goes-home.wat — arc 255, STONE 255.77 (cutover stone 3).
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; BRIEF: docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-255.77-the-uuid-goes-home.md
;; RULING: U1 (builder, 2026-10-01) — the Uuid type's key becomes `:wat::uuid::UUID`,
;;         spelled in `.wat` as the faithful-Clojure symbol `wat.uuid/UUID` (NOT a keyword —
;;         `wat.type/` cutover's 24-name closed set does not include Uuid, so it does not take
;;         the `wat.type/<tail>` spelling `types-to-wat-type.wat` used; it takes a namespace-
;;         preserving symbol, the SAME shape `wat.time/Instant` / `wat.holon/HolonAST` already
;;         use for a non-`wat.type/`-member home type).
;;
;; Converts every BARE type-position keyword `:wat::core::Uuid` to the symbol `wat.uuid/UUID`.
;; One name, one rule — unlike `rename-four-families-to-their-homes.wat`'s four verb families,
;; there is exactly one spelling to move (the TYPE key; the six verbs — `v4`, `v5`, `nil`,
;; `from-string`, `to-string`, `version`, `rfc4122-variant?` — already live in `:wat::uuid::*`,
;; stone 255's "the four that got homes"; not touched again here).
;;
;; ⚠ EXACT-STRING match on the WHOLE keyword token (`:wat::core::Uuid`, no trailing `/` and no
;; trailing `::`), same discipline as `rename-four-families-to-their-homes.wat`'s rules — this is
;; what keeps the rewrite from ever touching `:wat::core::Uuid/v4` (already-retired verb spelling,
;; untouched here) or any other `:wat::core::Uuid*` prefix. Measured against the corpus
;; (BRIEF's "the measurement" section): every one of the 11 `.wat` type-position occurrences is
;; this exact bare keyword — no symbol-spelled `wat.core/Uuid` occurrence exists to find.
;;
;; ★ THIS IS A RULES CODEMOD, NOT A CHAR-WALK. `wat/grep.wat`'s `Written` fact hands back a
;; keyword leaf as ONE WHOLE TOKEN (the reader already tokenized every file), so a rule that
;; matches nothing produces no Match facts, countable before anything is written (`--grep` mode).
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>     -> :user::grep  (the finder: prints every Match, unapplied)
;;   `wat` <this file>            -> :user::main  (the applier: rewrites files in place)
;;
;; ⚠ `.wat` ONLY. The same name also lives inside Rust string literals (`src/types.rs`,
;; `src/check.rs`, `src/value/value.rs`, …) — the kind guard below excludes string literals by
;; construction, so running this against a `.rs` file returns zero matches, silently. The `.rs`
;; side moves by hand (the BRIEF's "the key" step).
;;
;; Usage — finder (count before writing anything):
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/uuid-type-goes-home.wat | wc -l
;;
;; Usage — apply (one EDN vector of paths on stdin):
;;   git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
;;     | ./target/release/wat ./wat-scripts/fixes/uuid-type-goes-home.wat
;;
;; The rewrite is comment-faithful (rete's fact base has no notion of prose — a comment is not a
;; node, so a rule cannot touch it, by construction) and idempotent as a QUERY: after applying,
;; re-running the finder returns zero Match facts, because the old spelling is gone.

;; ── the finder — one rule over wat/grep.wat's stdlib fact base ──────────────────────────────

(:wat::rete::defrule :utgh::uuid
  :when [(:wat::grep::Node   (?id :- :id) (?k :- :kind))
         (:wat::grep::Written (?id :- :id) (?n :- :text) (?l :- :line) (?c :- :col) (?el :- :end-line) (?ec :- :end-col))
         (:wat::grep::Source (?f :- :file))
         ;; KEYWORD ONLY (`Written` already excludes string literals; `Named` still fires for
         ;; string literals too — the stone 255 "four families" precedent's own guard).
         (:wat::rete::where (:wat::rete::core::enum::= ?k (:wat::grep::NodeKind.Keyword {})))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::Uuid"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "uuid-type-goes-home"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)
                       (:wat::grep::Capture :name "new" :value "wat.uuid/UUID")))])

(:wat::core::defn :user::grep [] -> (wat.type/PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :utgh))

;; ── the applier's own query — field-destructured, same shape as the four-families codemod ──

(:wat::rete::defquery :utgh::q-match
  :params []
  :when [(:wat::grep::Match (?line :- :line) (?col :- :col)
           (?end-line :- :end-line) (?end-col :- :end-col) (?captures :- :captures))])

;; second-capture / first-capture — typed wrappers, same reasoning as
;; `rename-four-families-to-their-homes.wat`'s pair (PersistentMap/get's value type needs a
;; concrete argument type to force it; an explicit-signature wrapper does that).
(:wat::core::defn :utgh::second-capture
  [captures <- (wat.type/PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::second captures))

(:wat::core::defn :utgh::first-capture
  [captures <- (wat.type/PersistentVector :- [:wat::grep::Capture])]
  -> :wat::grep::Capture
  (:wat::core::first captures))

;; edits-of — query rows -> Vector of Tuple(offset, old-text, new-text), UNSORTED.
(:wat::core::defn :utgh::edits-of
  [rows  <- wat.type/PersistentVector
   lines <- (wat.type/Vector :- [wat.type/String])
   acc   <- (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])]
  -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
  (:wat::core::foldl
    (:wat::core::fn [a   <- (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
                     row <- wat.type/PersistentMap]
      -> (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
      (:wat::core::let
        [line     (:wat::core::Option/expect (:wat::core::get row "?line")     "q-match: ?line")
         col      (:wat::core::Option/expect (:wat::core::get row "?col")      "q-match: ?col")
         captures (:wat::core::Option/expect (:wat::core::get row "?captures") "q-match: ?captures")
         old-text (:wat::grep::Capture/value (:utgh::first-capture captures))
         new-text (:wat::grep::Capture/value (:utgh::second-capture captures))
         start    {:line line     :col col}
         offset   (:wat::fix::fix-text-offset-of start lines)]
        (:wat::core::concat a
          (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
            (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String] offset old-text new-text)))))
    acc rows))

;; convert-one — one file, through the already-compiled network via `overlay`.
(:wat::core::defn :utgh::convert-one
  [overlay <- :wat::rete::Overlay
   path    <- wat.type/String]
  -> wat.type/nil
  (:wat::core::let
    [src     (:wat::io::read-file path)
     lines   (:wat::string::split src "\n")
     facts   (:wat::grep::facts-of path src)
     records (:wat::grep::facts-as-records facts)
     fired   (:wat::core::match (overlay records)
               [:wat::rete::FireOutcome.Fired {:value __fired} __fired]
               [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds}
                 (:wat::kernel::assertion-failed! :message "codemod: session memory ceiling exceeded")]
               [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still}
                 (:wat::kernel::assertion-failed! :message "codemod: fixpoint round cap exceeded")])
     rows    (:wat::rete::query fired (:utgh::q-match))
     empty-e (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])])
     edits   (:utgh::edits-of rows lines empty-e)
     ;; SORT DESCENDING BY OFFSET — fix-text-apply splices right-to-left; rete returns query
     ;; results in NETWORK order, not source order.
     sorted  (:wat::core::sort
               (:wat::core::fn [a <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])
                                b <- (wat.type/Tuple :- [wat.type/i64 wat.type/String wat.type/String])]
                 -> wat.type/bool
                 (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
               edits)
     out     (:wat::fix::fix-text-apply src sorted)]
    (:wat::core::do
      (:wat::io::write-file path out)
      (:wat::kernel::println (:wat::string::concat "[uuid-type-goes-home] " path)))))

(:wat::core::defn :utgh::convert-each
  [overlay <- :wat::rete::Overlay
   paths   <- (wat.type/Vector :- [wat.type/String])]
  -> wat.type/nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:utgh::convert-one overlay (:wat::core::first paths))
      (:utgh::convert-each overlay (:wat::core::rest paths)))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
             [:wat::kernel::ReadlnOutcome.Eof {}
               (:wat::kernel::assertion-failed! :message "readln: end of input")]
             [:wat::kernel::ReadlnOutcome.Stopped {}
               (:wat::kernel::assertion-failed! :message "readln: stop requested")])]
    (:wat::rete::with-overlay (:wat::rete::collect-rules :utgh)
      (wat.type/PersistentVector :- [:wat::rete::Query] (:utgh::q-match))
      (:wat::core::fn [overlay <- :wat::rete::Overlay] -> wat.type/nil
        (:utgh::convert-each overlay paths)))))
