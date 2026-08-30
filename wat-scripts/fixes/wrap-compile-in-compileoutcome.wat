;; wat-scripts/fixes/wrap-compile-in-compileoutcome.wat — arc 278 the outcome wall, S2e
;; (`compile` / `compile-all` / `arm-session` — the TERMINATION VERDICT, ~368 sites.)
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;; A copy of wrap-fire-rules-in-fireoutcome.wat with FOUR head keywords and the arm strings
;; changed — which is exactly the reuse that justified writing 177 lines for 13 sites. Kept as a
;; SEPARATE recorded migration rather than parameterising the first: a codemod is a record of what
;; was actually run, and one file that rewrites itself per invocation records nothing.
;;
;; THE CHANGE: `:wat::rete::compile` / `compile-all` / `arm-session` now return
;; `(:wat::rete::CompileOutcome)` instead of a bare Session — a
;; fire's two ceilings (`max-fire-rounds`, `max-session-bytes`) cannot be proven at load, so the
;; breach is a matchable VALUE the caller faces, never a raise that unwinds past them (ADT; wat has
;; no try/catch). Every CALL SITE that used the bare Session now type-errors. This codemod wraps
;; each call in the CompileOutcome match, extracting the session on the happy path and dying loudly on
;; the two ceiling arms — the RULED disposition for the corpus, and the honest one: these fixtures
;; derive a handful of facts against a 1 GiB default, so a ceiling arm here is a SUBSTRATE BUG, not
;; a workload that outgrew its bound. The sibling pattern is spawn.wat's recv'/send' arms.
;;
;;   (compile-all ARGS)
;;     ->  (:wat::core::match (compile-all ARGS)
;;           ((CompileOutcome::Compiled __session) __session)
;;           ((CompileOutcome::MayNotTerminate __rule __ft) (assertion-failed! "…" :None :None)))
;;
;; TWO arms. `MayNotTerminate` is the ONLY refusal that converts: `arm-session`'s ArityMismatch and
;; TypeMismatch are BUGS IN THE PROGRAM, not judgements about the caller's data, and they stay
;; raises (see `wat/rete.wat`'s `CompileOutcome` note and `kernel::outcome`).
;;
;; ⚠ THE BINDERS ARE `__`-PREFIXED, and that is not style. A bare `s`/`p` binder (the connect'
;; codemod's choice) SHADOWS any same-named binding in the enclosing scope, silently, at every one
;; of the sites it rewrites. It got away with it; a sweep of 1_182 sites will not.
;;
;; MATCHER (structural): a list whose head keyword is EXACTLY `:wat::rete::compile`,
;; `:wat::rete::compile-all`, or `:wat::rete::arm-session`. ⚠ `compile` is a strict PREFIX of
;; `compile-all` AND of the INTERNAL helpers `compile-rule` / `compile-query` / `compile-condition`,
;; which return a CompileState rather than a Session — a prefix matcher would wrap those and emit
;; nonsense that still parses. Exactness is more load-bearing here than anywhere prior in this wall.
;; IDEMPOTENCY: the walk suppresses the top-wrap of a call that is already the scrutinee (child[1])
;; of a `:wat::core::match` carrying a `CompileOutcome::` arm — so re-run = 0 changes (it still
;; recurses INTO that scrutinee's ARG, catching a nested call).
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-compile-in-compileoutcome.wat

;; ── helpers (mirror wrap-connect-prime-in-connectoutcome.wat) ─────────────────
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; head-kw-name — a list's head keyword name (child[0]); "" if not a list / empty / non-keyword head.
(:wat::core::defn :user::head-kw-name [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    ""))

;; compile-call? — a list whose head keyword is EXACTLY one of the three session-minting verbs.
;;
;; ⛔ EXACTNESS IS CRITICAL AND MORE SO THAN ANYWHERE PRIOR. `:wat::rete::compile` is a strict
;; prefix of `compile-all`, `compile-rule`, `compile-query` and `compile-condition` — and the last
;; three are INTERNAL helpers that return a CompileState, not a Session. A prefix matcher would
;; wrap them and produce nonsense that still parses. Three exact names, nothing else.
(:wat::core::defn :user::compile-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [h (:user::head-kw-name node)]
    (:wat::core::if (:wat::core::= h ":wat::rete::compile")
      true
      (:wat::core::if (:wat::core::= h ":wat::rete::compile-all")
        true
        (:wat::core::= h ":wat::rete::arm-session")))))

;; arm-head-name — an arm is `(pattern body…)`; if pattern (child[0]) is a list, return its head
;; keyword name; else "" (a bare/`_` pattern has no head keyword).
(:wat::core::defn :user::arm-head-name [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::empty? ch)
        ""
        (:wat::core::let [pat (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
            (:wat::core::let [pch (:wat::core::ast->children pat)]
              (:wat::core::if (:wat::core::empty? pch) "" (:user::kw-name (:wat::core::first pch))))
            ""))))
    ""))

;; any-arm-head-contains? — does any arm (children[2..]) have a pattern head keyword containing `needle`?
(:wat::core::defn :user::any-arm-head-contains?
  [arms <- (:wat::core::Vector :- [:wat::WatAST])  needle <- :wat::core::String] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  arm <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::core::string::contains? (:user::arm-head-name arm) needle)))
    false arms))

;; already-facing-compile-match? — a `:wat::core::match` whose scrutinee (child[1]) is a compile
;; call AND which already carries a `CompileOutcome::` arm (the shape THIS codemod emits). Its
;; scrutinee's top-wrap must be suppressed on re-run (idempotency).
(:wat::core::defn :user::already-facing-compile-match? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:user::head-kw-name node) ":wat::core::match")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::let
          [scrut (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
           arms  (:wat::core::into [] (:wat::core::drop ch 2))]
          (:wat::core::if (:user::compile-call? scrut)
            (:user::any-arm-head-contains? arms "CompileOutcome::")
            false))))
    false))

;; ── EDIT: two span inserts wrapping the compile call node ────────────────────
(:wat::core::defn :user::wrap-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
    (:wat::core::Tuple (:user::start-off node lines) 0
      "(:wat::core::match ")
    (:wat::core::Tuple (:user::end-off node lines) 0
      " ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))")))

;; recurse a node's children WITHOUT wrapping the node's own top (idempotency suppression).
(:wat::core::defn :user::node-edits-no-top
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) lines)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

;; walk one node → its edit (if a compile call) + descendants'. For an already-facing
;; CompileOutcome match, the scrutinee (child[1]) is recursed WITHOUT re-wrapping its top.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::compile-call? node)
            (:user::wrap-edits node lines)
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))]
    (:wat::core::if (:user::already-facing-compile-match? node)
      ;; suppress the scrutinee's top-wrap; recurse everything else normally.
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
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    items))

;; ── per-file migrate ─────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::core::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
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
        (:wat::kernel::println (:wat::core::string::concat "[wrap-compileoutcome] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
