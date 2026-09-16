;; wat-scripts/fixes/wrap-overlay-in-fireoutcome.wat — arc 294, the grok-rete replay (2a4d).
;; SCOPE: wat-scripts/fixes/*.wat
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; THE RECORD THIS MAKES. Step #155 (`406a7c340`, grok source `701cf473a`) made `fire-rules`
;; total: `:wat::rete::fire-rules` — and with it the closure `:wat::rete::with-overlay` hands its
;; callback — began returning `(:wat::rete::FireOutcome :- [T])` instead of a bare Session. Eleven
;; CHAIN members (the `rename-…-to-their-homes` grep-codemods) call that closure as
;; `(overlay records)`, so after the flip the chain could not LOAD; the wrap went in BY HAND,
;; seven identical lines a file, eleven files. An eleven-file structural rewrite by hand is
;; exactly what R21 routes to a codemod ("we use wat-fix to unfuck the farm"). #155's content is
;; not in question — it is green and its text is unchanged. THIS file is the METHOD, recorded
;; after the fact, so the migration can be replayed, audited and re-run rather than retyped.
;;
;;   (overlay ARG)
;;     ->  (:wat::core::match (overlay ARG)
;;           [:wat::rete::FireOutcome.Fired {:value __fired} __fired]
;;           [:wat::rete::FireOutcome.MemoryCeilingExceeded {…} (assertion-failed! "codemod: …")]
;;           [:wat::rete::FireOutcome.RoundCapExceeded      {…} (assertion-failed! "codemod: …")])
;;
;; MATCHER (structural): a list whose head is the bare SYMBOL `overlay` — the binder
;; `with-overlay` hands its callback. ⚠ EXACT symbol equality, and a SYMBOL head, never a keyword:
;; `(:wat::rete::with-overlay rules …)` is a keyword head and is not a site; neither is a name
;; that merely contains "overlay".
;;
;; EMISSION IS INDENTED, not one line: the arms sit at the wrapped call's own column + 2 and each
;; ceiling body at + 4. That is byte-for-byte what #155 wrote, at every column it wrote it at
;; (13 in the eleven chain members, 17 in `wat/grep.wat`, 36 in `wat/fmt.wat`).
;;
;; ⚠ SCOPE IS `wat-scripts/fixes/`, and that is a MEASURED boundary. `wat/grep.wat` and
;; `wat/fmt.wat` carry overlay sites too, but their ceiling arms carry file-specific messages
;; (`(:wat::string::concat "wat::grep: session memory ceiling exceeded on " path)`, `"fmt: …"`) —
;; a DIFFERENT edit, which this migration must not overwrite with the codemods' `"codemod: …"`.
;;
;; ⚠ THE BINDERS ARE `__`-PREFIXED, and that is not style — the ruling recorded in
;; `wrap-fire-rules-in-fireoutcome.wat`: a bare `s`/`p` binder SHADOWS any same-named binding in
;; the enclosing scope, silently, at every site it rewrites.
;;
;; IDEMPOTENCY: the walk suppresses the top-wrap of a call that is already the scrutinee (child[1])
;; of a `:wat::core::match` carrying a `FireOutcome.` arm — so re-run = 0 changes — while still
;; recursing INTO that scrutinee's ARG, catching a nested call.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-overlay-in-fireoutcome.wat

;; ── helpers (mirror wrap-fire-rules-in-fireoutcome.wat) ──────────────────────
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::sym-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "symbol")
    (:wat::core::ast-name n) ""))

;; head-kw-name — a list's head KEYWORD name (child[0]); "" if not a list / empty / non-keyword.
(:wat::core::defn :user::head-kw-name [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    ""))

;; head-sym-name — a list's head SYMBOL name (child[0]); "" if not a list / empty / non-symbol.
(:wat::core::defn :user::head-sym-name [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::sym-name (:wat::core::first ch))))
    ""))

;; overlay-call? — a call of the `with-overlay` callback binder. EXACT name, SYMBOL head.
(:wat::core::defn :user::overlay-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:user::head-sym-name node) "overlay"))

;; arm-head-name — an arm is `(pattern body…)` / `[pattern map body]`; if the pattern is a list
;; or vector, its head keyword name; else "" (a bare/`_` pattern has no head keyword).
(:wat::core::defn :user::arm-head-name [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "vector")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    (:wat::fix::arm-head-name arm)))

(:wat::core::defn :user::any-arm-head-contains?
  [arms <- (:wat::core::Vector :- [:wat::WatAST])  needle <- :wat::core::String] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  arm <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::string::contains? (:user::arm-head-name arm) needle)))
    false arms))

;; already-facing-overlay-match? — a `:wat::core::match` whose scrutinee (child[1]) is an overlay
;; call AND which already carries a `FireOutcome.` arm (the shape THIS codemod emits).
(:wat::core::defn :user::already-facing-overlay-match? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:user::head-kw-name node) ":wat::core::match")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::let
          [scrut (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
           arms  (:wat::core::into [] (:wat::core::drop ch 2))]
          (:wat::core::if (:user::overlay-call? scrut)
            (:user::any-arm-head-contains? arms "FireOutcome.")
            false))))
    false))

;; ── EDIT: two span inserts wrapping the overlay call node ────────────────────
(:wat::core::defn :user::spaces [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::core::< n 1)
    ""
    (:wat::string::concat " " (:user::spaces (:wat::i64::- n 1)))))

(:wat::core::defn :user::cat [xs <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::core::String  b <- :wat::core::String] -> :wat::core::String
      (:wat::string::concat a b))
    ""
    xs))

;; The arms, indented against the WRAPPED CALL's own column (`ast-span`'s `:col` is 1-indexed):
;; arms at column + 2, each ceiling body at column + 4 — #155's layout, at any column.
(:wat::core::defn :user::wrap-suffix [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let
    [col (:wat::i64::- (:wat::fix::span-col node) 1)
     arm (:wat::string::concat "\n" (:user::spaces (:wat::i64::+ col 2)))
     bod (:wat::string::concat "\n" (:user::spaces (:wat::i64::+ col 4)))]
    (:user::cat
      (:wat::core::Vector :- [:wat::core::String]
        arm "[:wat::rete::FireOutcome.Fired {:value __fired} __fired]"
        arm "[:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds}"
        bod "(:wat::kernel::assertion-failed! :message \"codemod: session memory ceiling exceeded\")]"
        arm "[:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still}"
        bod "(:wat::kernel::assertion-failed! :message \"codemod: fixpoint round cap exceeded\")])"))))

(:wat::core::defn :user::wrap-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple (:user::start-off node lines) ""
      "(:wat::core::match ")
    (:wat::core::Tuple (:user::end-off node lines) ""
      (:user::wrap-suffix node))))

;; recurse a node's children WITHOUT wrapping the node's own top (idempotency suppression).
(:wat::core::defn :user::node-edits-no-top
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) lines)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])))

;; walk one node → its edit (if an overlay call) + descendants'. For an already-facing
;; FireOutcome match, the scrutinee (child[1]) is recursed WITHOUT re-wrapping its top.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::overlay-call? node)
            (:user::wrap-edits node lines)
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
    (:wat::core::if (:user::already-facing-overlay-match? node)
      (:wat::core::let
        [ch    (:wat::core::ast->children node)
         scrut (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
         head  (:wat::core::first ch)
         arms  (:wat::core::into [] (:wat::core::drop ch 2))]
        (:wat::core::concat
          (:wat::core::concat (:user::node-edits head lines) (:user::node-edits-no-top scrut lines))
          (:user::seq-edits arms lines)))
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
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))]))
     eds   (:user::seq-edits forms lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[wrap-overlay-in-fireoutcome] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
