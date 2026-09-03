;; wat-scripts/fixes/repoint-retired-heads-to-live-spellings.wat — arc 255 Stone 1c-0a-ii.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-0a-ii-the-capability-outlived-the-name.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-1c-0a-ii-three-repoints.md
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files, no python, no sed — wat rewrites
;; wat (repository doctrine R21). Three corpus artifacts each call a verb that was retired; each
;; has a live successor expressing the SAME capability (the DESIGN's finding, grounded against
;; wat/seq.wat, wat/bracket.wat, and src/check.rs's infer_positional_accessor before this file was
;; written). This codemod repoints all three call sites. It deletes nothing and edits no file
;; outside the three named below.
;;
;;   :wat::core::reduce-walk          -> :wat::core::foldl-spec-walk    (pure rename)
;;   (:wat::spawn::process/grants X)  -> (:wat::spawn::process)         (form rewrite: the
;;                                                                        retired combinator's
;;                                                                        sole argument — which
;;                                                                        also carries the dead
;;                                                                        `:wat::capability::
;;                                                                        Grantable` name — is
;;                                                                        dropped along with it)
;;   (:wat::core::tuple-get X 0)      -> (:wat::core::first X)          (form rewrite, NOT a pure
;;                                                                        rename — see note below)
;;
;; ⚠ `tuple-get` is NOT a pure rename, despite the BRIEF's framing. `tuple-get` was the general
;; 2-arg accessor `(tuple, index)`; `first` is the 1-arg member of the `first`/`second`/`third`
;; fixed-index family (src/check.rs `infer_positional_accessor`: `args.len() != 1` is an
;; ArityMismatch). The corpus's one call site is `(:wat::core::tuple-get t 0)` — a literal index
;; 0 — so `(:wat::core::first t)` is the exact same capability (same element, same type, per
;; `infer_positional_accessor`'s Tuple branch: `elements.get(index)` at `index=0`), but reaching
;; it requires ALSO dropping the trailing literal `0` argument, not just swapping the head token.
;; A bare head-rename here produces `(:wat::core::first t 0)`, a 2-arg call `first` rejects
;; outright — dry-run caught this (see the dry-run diff below); this codemod treats tuple-get
;; as a form rewrite for that reason, structurally identical in kind to the process/grants rule.
;;
;; The `reduce-walk` rename rides `:wat::fix::rename-keyword-prefix` (comment-faithful,
;; boundary-aware per `rename-valid-match?`, wat/fix.wat:632) exactly as
;; `deprime-telemetry-sqlite.wat` composes it — it is the one genuinely pure rename here. The two
;; form rewrites are NEW logic (there is no rename primitive for "drop an argument") — written
;; here, under :user::, the same way `address-transport-arity.wat` keeps its own predicate/
;; edit-walk pair local to the fixes file rather than adding a new verb to wat/fix.wat; both ride
;; wat/fix.wat's existing `structural?` / `ast-span` / `ast-end-span` / `fix-text-span-text` /
;; `fix-text-apply` primitives, mirroring `first-of-drop-edits`' shape (wat/fix.wat:1223) with
;; one fewer edit each: each collapses ONE argument out of an EXISTING call (2 edits: rename the
;; head, delete the gap between two of the call's own children) — neither merges two nested
;; lists into one, so neither needs a third "closing paren" edit.
;;
;; Comment-faithful and idempotent: each rule matches only the RETIRED spelling/shape (the
;; tuple-get rule additionally requires the literal index text to read "0"), so a second run
;; over already-migrated files reports 0 changes.
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["wat-scripts/scratch-pad/bench-reduce-foldl-vs-seqable-walk.wat" \
;;            "wat-scripts/probes/arc-170/probe-cap2-process-grantpath.wat" \
;;            "wat-scripts/scratch-pad/arc109-2iii-fn-bracket-destinations.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/repoint-retired-heads-to-live-spellings.wat

;; ── process/grants form rewrite ─────────────────────────────────────────────────────────────

;; process-grants-call? — a List headed EXACTLY :wat::spawn::process/grants with exactly one
;; argument (head + 1 arg = 2 children). Both the head identity and the arity are checked so a
;; differently-shaped call is left untouched rather than mis-edited.
(:wat::core::defn :user::process-grants-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 2)
        (:wat::fix::calls-to? node ":wat::spawn::process/grants")
        false))
    false))

;; process-grants-edits — the 2 span edits for one matched node:
;;   1. head span -> rename text ":wat::spawn::process/grants" -> ":wat::spawn::process"
;;   2. the gap from the head's own end-span to the arg's own end-span (the leading space plus
;;      the whole retired-argument expression, including its buried
;;      `:wat::capability::Grantable` reference) -> deleted.
;; The outer list's own closing paren is untouched — unlike first-of-drop-edits (which collapses
;; two nested lists into one and so needs a third edit for the inner paren), this only drops one
;; argument out of an EXISTING call.
(:wat::core::defn :user::process-grants-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [ch        (:wat::core::ast->children node)
     head      (:wat::core::first ch)
     arg       (:wat::core::first (:wat::core::rest ch))
     head-off  (:wat::fix::fix-text-offset-of (:wat::core::ast-span head) lines)
     head-name (:wat::core::ast-name head)
     gap-off   (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span head) lines)
     gap-text  (:wat::fix::fix-text-span-text (:wat::core::ast-end-span head) (:wat::core::ast-end-span arg) lines src)]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple head-off head-name ":wat::spawn::process")
      (:wat::core::Tuple gap-off gap-text ""))))

;; process-grants-scan / process-grants-walk — recursive descent, same shape as
;; first-of-drop-scan/-walk (wat/fix.wat:1251/1268): a match emits its edits and does NOT also
;; recurse into its own children (the whole matched call is replaced as a unit); a non-match
;; recurses into every structural child.
(:wat::core::defn :user::process-grants-scan
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::process-grants-call? node)
    (:user::process-grants-edits node src lines)
    (:wat::core::if (:wat::fix::structural? node)
      (:user::process-grants-walk (:wat::core::ast->children node) src lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::process-grants-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::concat
      (:user::process-grants-scan (:wat::core::first items) src lines)
      (:user::process-grants-walk (:wat::core::rest items) src lines))))

;; process-grants-to-plain — the entry point for the form rewrite. src in, migrated src out;
;; comment- and layout-faithful (splices the ORIGINAL text at spans via fix-text-apply).
(:wat::core::defn :user::process-grants-to-plain [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     eds   (:user::process-grants-walk (:wat::core::ast->children tree) src lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── tuple-get(t, 0) -> first(t) form rewrite ────────────────────────────────────────────────

;; tuple-get-zero-call? — a List headed EXACTLY :wat::core::tuple-get with exactly two
;; arguments (head + tuple + index = 3 children) whose SECOND argument is an int-kind leaf
;; whose own source text reads "0". The literal-index requirement is deliberate: `first` is
;; only the correct successor for index 0 (`second`/`third` would be for 1/2) — a differently-
;; indexed call is left untouched rather than mis-edited.
(:wat::core::defn :user::tuple-get-zero-call?
  [node <- :wat::WatAST src <- :wat::core::String lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 3)
        (:wat::core::if (:wat::fix::calls-to? node ":wat::core::tuple-get")
          (:wat::core::let [idx (:wat::core::nth ch 2)]
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind idx) "int")
              (:wat::core::=
                (:wat::fix::fix-text-span-text (:wat::core::ast-span idx) (:wat::core::ast-end-span idx) lines src)
                "0")
              false))
          false)
        false))
    false))

;; tuple-get-zero-edits — the 2 span edits for one matched node:
;;   1. head span -> rename text ":wat::core::tuple-get" -> ":wat::core::first"
;;   2. the gap from the tuple-argument's own end-span to the index-argument's own end-span
;;      (the trailing " 0") -> deleted.
(:wat::core::defn :user::tuple-get-zero-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [ch         (:wat::core::ast->children node)
     head       (:wat::core::first ch)
     tuple-arg  (:wat::core::nth ch 1)
     idx-arg    (:wat::core::nth ch 2)
     head-off   (:wat::fix::fix-text-offset-of (:wat::core::ast-span head) lines)
     head-name  (:wat::core::ast-name head)
     gap-off    (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span tuple-arg) lines)
     gap-text   (:wat::fix::fix-text-span-text (:wat::core::ast-end-span tuple-arg) (:wat::core::ast-end-span idx-arg) lines src)]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple head-off head-name ":wat::core::first")
      (:wat::core::Tuple gap-off gap-text ""))))

;; tuple-get-zero-scan / tuple-get-zero-walk — recursive descent, same shape as
;; process-grants-scan/-walk above.
(:wat::core::defn :user::tuple-get-zero-scan
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::tuple-get-zero-call? node src lines)
    (:user::tuple-get-zero-edits node src lines)
    (:wat::core::if (:wat::fix::structural? node)
      (:user::tuple-get-zero-walk (:wat::core::ast->children node) src lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::tuple-get-zero-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::concat
      (:user::tuple-get-zero-scan (:wat::core::first items) src lines)
      (:user::tuple-get-zero-walk (:wat::core::rest items) src lines))))

;; tuple-get-zero-to-first — the entry point for this form rewrite.
(:wat::core::defn :user::tuple-get-zero-to-first [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     eds   (:user::tuple-get-zero-walk (:wat::core::ast->children tree) src lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── entry point: compose the pure rename + the two form rewrites ───────────────────────────

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:user::tuple-get-zero-to-first
    (:user::process-grants-to-plain
      (:wat::fix::rename-keyword-prefix ":wat::core::reduce-walk" ":wat::core::foldl-spec-walk"
        src))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[repointed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
