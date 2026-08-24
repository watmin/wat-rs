;; wat-scripts/fixes/wrap-connect-prime-in-connectoutcome.wat — arc 278 peer-lifecycle Strike 4
;; (the connect'-outcome wall, the LAST peer wall) — the corpus sweep.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;; The exact sibling of wrap-client-method-match-in-recvoutcome.wat (the recv'-wall codemod).
;;
;; THE CHANGE: `:wat::kernel::connect'` now returns `(:wat::kernel::ConnectOutcome :- [S R])` instead
;; of a bare `(Peer' :- [S R])` — the dial failure is a matchable VALUE the caller faces (ADT; wat has
;; no try/catch). Every CALL SITE that used the bare Peer' now type-errors. This codemod wraps
;; each `(connect' ARG)` call in the ConnectOutcome match, extracting the peer on the happy path
;; and dying (fatal, preserving the pre-wall raise-unwind) on the three failure arms — the RULED
;; disposition for the whole corpus (a fixture/probe that cannot dial should fail loudly; the
;; sibling pattern is spawn.wat's recv'/send' assertion-failed! arms):
;;
;;   (connect' ARG)
;;     ->  (:wat::core::match (connect' ARG)
;;           ((ConnectOutcome::Connected p) p)
;;           ((ConnectOutcome::Refused  c) (assertion-failed! (Failure/message c) :None :None))
;;           ((ConnectOutcome::Rejected c) (assertion-failed! (Failure/message c) :None :None))
;;           ((ConnectOutcome::Failed   c) (assertion-failed! (Failure/message c) :None :None)))
;;
;; The stdlib sites (journal/span/query/bracket — wat/*.wat) are NOT here: they were hand-faced
;; (per-site semantic — a service :init dial, a with-span/defservice/bracket MACRO body — not
;; uniform codemod material). The connect'-wall probe (probe_arc278_connect_outcome_wall.wat) is
;; NOT here either: it intentionally RETURNS the raw ConnectOutcome for the Rust probe to assert.
;;
;; MATCHER (structural): a list whose head keyword is `:wat::kernel::connect'`.
;; IDEMPOTENCY: the walk suppresses the top-wrap of a connect' that is already the scrutinee
;; (child[1]) of a `:wat::core::match` carrying a `ConnectOutcome::` arm — so re-run = 0 changes
;; (it still recurses INTO that scrutinee's ARG, catching a nested connect').
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-connect-prime-in-connectoutcome.wat

;; ── helpers (mirror wrap-client-method-match-in-recvoutcome.wat) ─────────────────
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

;; connect-call? — a list whose head keyword is exactly `:wat::kernel::connect'`.
(:wat::core::defn :user::connect-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:user::head-kw-name node) ":wat::kernel::connect'"))

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

;; already-facing-connect-match? — a `:wat::core::match` whose scrutinee (child[1]) is a connect'
;; call AND which already carries a `ConnectOutcome::` arm (the shape THIS codemod emits). Its
;; scrutinee's top-wrap must be suppressed on re-run (idempotency).
(:wat::core::defn :user::already-facing-connect-match? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:user::head-kw-name node) ":wat::core::match")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::let
          [scrut (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
           arms  (:wat::core::into [] (:wat::core::drop ch 2))]
          (:wat::core::if (:user::connect-call? scrut)
            (:user::any-arm-head-contains? arms "ConnectOutcome::")
            false))))
    false))

;; ── EDIT: two span inserts wrapping the connect' call node ────────────────────────
(:wat::core::defn :user::wrap-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
    (:wat::core::Tuple (:user::start-off node lines) 0
      "(:wat::core::match ")
    (:wat::core::Tuple (:user::end-off node lines) 0
      " ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))")))

;; recurse a node's children WITHOUT wrapping the node's own top (idempotency suppression).
(:wat::core::defn :user::node-edits-no-top
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) lines)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

;; walk one node → its edit (if a connect' call) + descendants'. For an already-facing
;; ConnectOutcome match, the scrutinee (child[1]) is recursed WITHOUT re-wrapping its top.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::connect-call? node)
            (:user::wrap-edits node lines)
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))]
    (:wat::core::if (:user::already-facing-connect-match? node)
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
        (:wat::kernel::println (:wat::core::string::concat "[wrap-connectoutcome] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
