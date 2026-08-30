;; wat-scripts/fixes/read-string-to-outcome.wat — arc 170, the read-string totality flip.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; THE CHANGE. `:wat::core::read-string` RAISED on malformed source. Proven live: an arrow key at
;; the REPL sends ESC (0x1B), the lexer rejects the control byte, and the raise unwinds THROUGH the
;; loop and kills the session. wat has no try/catch by design — a failure must be a matchable VALUE
;; — so there is nothing a caller can do about it. read-string now returns
;; `:wat::core::ReadOutcome::{Forms [forms], Malformed [cause <- :wat::core::Error]}` and every call
;; site faces the outcome.
;;
;; Converted IN PLACE, with no total-sibling verb, because that is what this substrate has done every
;; previous time: RecvOutcome / SendOutcome / CloseOutcome each replaced the raiser rather than
;; standing beside it (`src/types.rs`). Two ways to parse would be the synonym anti-pattern
;; (`docs/ITERATION-PATTERNS.md` — "Synonyms are LLM-hostile").
;;
;; THE REWRITE — each `(read-string X)` becomes the match that unwraps it:
;;
;;   (:wat::core::read-string X)
;;     ->  (:wat::core::match (:wat::core::read-string X)
;;           ((:wat::core::ReadOutcome::Forms __forms) __forms)
;;           ((:wat::core::ReadOutcome::Malformed __cause)
;;             (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :None :None)))
;;
;; The Malformed arm is a VISIBLE chosen death, not a reinstated hidden raise — the distinction
;; `DESIGN-no-hidden-failures.md` draws in its own words: "An AUTHOR-written recv' arm MAY
;; assertion-failed! (a visible chosen death)… a GENERATED method may NOT." Every site here is an
;; author call site, and for most of them (wat/fix.wat, deporder.wat, lint.wat — tools parsing files
;; they own) dying on unparseable input is the correct behaviour; it is now WRITTEN DOWN at the site
;; instead of being an invisible property of the verb. The cause is CARRIED, never dropped.
;;
;; A caller that wants resilience refines its own arms afterwards — that is the whole point of the
;; flip, and the REPL is the first one that does.
;;
;; MATCHER (structural): a list whose head keyword is `:wat::core::read-string`.
;; IDEMPOTENT: a `:wat::core::match` whose arms already mention `ReadOutcome::` does NOT get its
;; SCRUTINEE rewritten (that scrutinee is our own prior output) — but its arms are still walked, so a
;; read-string nested inside an arm body is still converted. Re-run = 0 edits.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/read-string-to-outcome.wat
;;
;; ⚠ BOOTSTRAP ORDER (the STASH-DANCE, `wat/fix.wat` header): this codemod's own tool — wat/fix.wat —
;; is itself a read-string caller, and the stdlib is BAKED into the binary. So the corpus is rewritten
;; FIRST, with the OLD binary (which still has the raising read-string and can therefore run), and the
;; `src/` flip lands SECOND. Rewriting in the other order leaves no working binary to rewrite with.

;; ── helpers (mirrored from wrap-client-method-match-in-recvoutcome.wat) ──────────
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; arm-head-name — an arm is `(pattern body…)`; if pattern is a list, its head keyword's name.
(:wat::core::defn :user::arm-head-name [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::empty? ch)
        ""
        (:wat::core::let [pat (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
            (:wat::core::let [pch (:wat::core::ast->children pat)]
              (:wat::core::if (:wat::core::empty? pch) "" (:user::kw-name (:wat::core::first pch))))
            (:user::kw-name pat)))))     ;; a UNIT-variant arm's pattern is a BARE keyword
    ""))

(:wat::core::defn :user::any-arm-head-contains?
  [arms <- (:wat::core::Vector :- [:wat::WatAST])  needle <- :wat::core::String] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  arm <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::string::contains? (:user::arm-head-name arm) needle)))
    false arms))

;; read-string-call? — a list whose head keyword is exactly `:wat::core::read-string`.
(:wat::core::defn :user::read-string-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::read-string")))
    false))

;; already-wrapped? — a `match` whose arms already name ReadOutcome:: (our own prior output).
(:wat::core::defn :user::already-wrapped? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::match")
          (:user::any-arm-head-contains? (:wat::core::into [] (:wat::core::drop ch 2)) "ReadOutcome::")
          false)))
    false))

;; ── EDIT: two inserts — `(match ` before the call, the two arms + `)` after it ────
(:wat::core::defn :user::wrap-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple (:user::start-off node lines) ""
      "(:wat::core::match ")
    (:wat::core::Tuple (:user::end-off node lines) ""
      " ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))")))

;; walk one node → its edits + its descendants'.
;;
;; The idempotency cut lives HERE, not in the matcher: for an already-wrapped match we skip child[1]
;; (the scrutinee we produced last run) and walk the rest, so a read-string nested inside an ARM body
;; is still reachable. Skipping the whole node would strand those.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::already-wrapped? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:user::seq-edits
        (:wat::core::concat
          (:wat::core::into [] (:wat::core::take ch 1))
          (:wat::core::into [] (:wat::core::drop ch 2)))
        lines))
    (:wat::core::let
      [this (:wat::core::if (:user::read-string-call? node)
              (:user::wrap-edits node lines)
              (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
      (:wat::core::if (:wat::fix::structural? node)
        (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) lines))
        this))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    items))

;; ── per-file migrate ─────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
     eds   (:user::seq-edits forms lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let
      [path (:wat::core::first paths)
       src  (:wat::io::read-file path)
       out  (:user::migrate src)]
      (:wat::core::do
        (:wat::core::if (:wat::core::= src out)
          nil
          (:wat::io::write-file path out))
        (:user::apply-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
