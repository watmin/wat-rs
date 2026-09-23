;; wat-scripts/fixes/wrap-cache-new-in-result-expect.wat — excursus 003 stone A, the corpus half.
;; SCOPE: corpus
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;; Structurally a thinner `wrap-compile-in-compileoutcome.wat` (its span-insert walk, its
;; `__`-free single-wrap edit); the difference is that the wrapper is a CALL, not a `match`, so
;; there are no arms and the idempotency test is "is this node already an expect's scrutinee".
;;
;; THE CHANGE: `:wat::cache::Lru/new` and `:wat::cache::HolographicLru/new` now return
;; `(:wat::core::Result :- [… :wat::cache::Fault])` instead of the bare handle — a non-positive
;; capacity is a wat VALUE, not a Rust `panic!` with a backtrace note and no span
;; (the-little-wat F-084; wat/cache.wat's "failure surface" header). Every call site that used
;; the bare handle now type-errors. This wraps each in `Result/expect`:
;;
;;   (:wat::cache::Lru/new 16)
;;     ->  (:wat::core::Result/expect (:wat::cache::Lru/new 16)
;;           ":wat::cache::Lru/new refused the capacity: it must be positive")
;;
;; ⭐ WHY `Result/expect` AND NOT A MATCH. The sites this runs over are FIXTURES and a scratch
;; probe, every one of which passes a positive literal (2, 3, 8, 10, 16). An `Err` arm there is
;; not a workload outcome — it is a substrate bug — so the honest disposition is to die loudly
;; naming the verb, exactly as `wrap-compile-in-compileoutcome.wat` ruled for its ceiling arms.
;; A caller whose capacity is DATA must match instead; `wat/cache.wat`'s `HolographicLru/new`
;; is the worked example, and it is hand-written because it propagates rather than expects.
;; ⛔ The message names `:wat::cache::Lru/new` for BOTH heads on purpose: `HolographicLru/new`
;; has no guard of its own and re-wraps Lru/new's fault verbatim, so naming itself would lie
;; about where the refusal came from.
;;
;; SPEC: a call to `:wat::cache::Lru/new` or `:wat::cache::HolographicLru/new` becomes that call wrapped as `(:wat::core::Result/expect <call> ":wat::cache::Lru/new refused the capacity: it must be positive")`.
;;
;; MATCHER (structural): a list whose head keyword is EXACTLY `:wat::cache::Lru/new` or
;; `:wat::cache::HolographicLru/new`. Exactness matters in the usual way — `:wat::cache::Lru/`
;; is a prefix of `Lru/put`, `Lru/get`, `Lru/len`, none of which return a Result.
;; IDEMPOTENCY: the walk suppresses the top-wrap of a call that is already the scrutinee
;; (child[1]) of a `:wat::core::Result/expect`, so re-run = 0 changes; it still recurses INTO
;; that scrutinee's args, catching a nested call.
;;
;; ⛔ NOT RUN OVER `wat/cache.wat`. That file is the DEFINITION, not a migration target: its
;; three sites each take a different shape (a match that lifts the raw tuple, a match that
;; propagates, and two service `:init`s that expect with their own messages). A blanket wrap
;; there would have turned `HolographicLru/new`'s propagating body into an expect — i.e. exactly
;; the swallow the stone exists to prevent.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-cache-new-in-result-expect.wat

;; ── helpers (mirror wrap-compile-in-compileoutcome.wat) ─────────────────────
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

;; cache-new-call? — EXACT head match on the two constructors that now return a Result.
(:wat::core::defn :user::cache-new-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [h (:user::head-kw-name node)]
    (:wat::core::if (:wat::core::= h ":wat::cache::Lru/new")
      true
      (:wat::core::= h ":wat::cache::HolographicLru/new"))))

;; already-expected? — a `:wat::core::Result/expect` whose scrutinee (child[1]) is one of those
;; calls. Its scrutinee's top-wrap must be suppressed on re-run (idempotency).
(:wat::core::defn :user::already-expected? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:user::head-kw-name node) ":wat::core::Result/expect")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:user::cache-new-call? (:wat::core::Option/expect (:wat::core::get ch 1) "scrut"))))
    false))

;; ── EDIT: two span inserts wrapping the constructor call node ────────────────
(:wat::core::defn :user::wrap-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple (:user::start-off node lines) ""
      "(:wat::core::Result/expect ")
    (:wat::core::Tuple (:user::end-off node lines) ""
      " \":wat::cache::Lru/new refused the capacity: it must be positive\")")))

;; recurse a node's children WITHOUT wrapping the node's own top (idempotency suppression).
(:wat::core::defn :user::node-edits-no-top
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) lines)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])))

;; walk one node → its edit (if a constructor call) + descendants'. For a node already facing an
;; expect, the scrutinee (child[1]) is recursed WITHOUT re-wrapping its top.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::cache-new-call? node)
            (:user::wrap-edits node lines)
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
    (:wat::core::if (:user::already-expected? node)
      ;; suppress the scrutinee's top-wrap; recurse everything else normally.
      (:wat::core::let
        [ch    (:wat::core::ast->children node)
         scrut (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
         head  (:wat::core::first ch)
         rest  (:wat::core::into [] (:wat::core::drop ch 2))]
        (:wat::core::concat
          (:wat::core::concat (:user::node-edits head lines) (:user::node-edits-no-top scrut lines))
          (:user::seq-edits rest lines)))
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
        (:wat::kernel::println (:wat::string::concat "[wrap-cache-new] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
