;; wat-scripts/fixes/wrap-cache-put-in-result-expect.wat — excursus 003 stone C, the corpus half.
;; SCOPE: corpus
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;; A copy of stone A's `wrap-cache-new-in-result-expect.wat` with the matcher and the message
;; changed; the walk, the span inserts, and the idempotency suppression are identical.
;;
;; THE CHANGE: `:wat::cache::Lru/put` now returns
;; `(:wat::core::Result :- [(:wat::core::Option :- [(:wat::cache::Entry :- [K V])]) :wat::cache::Fault])`
;; and `:wat::cache::HolographicLru/put` returns `(:wat::core::Result :- [:wat::core::nil :wat::cache::Fault])`
;; — an unhashable (opaque-handle) key is a wat VALUE, not a Rust `panic!` with a backtrace note
;; and no span (wat/cache.wat's "failure surface" header; src/rust_deps/cache.rs's module doc).
;; Every call site that matched the bare `Option`, or discarded the bare `nil`, now holds a
;; `Result`. This wraps each in `Result/expect`:
;;
;;   (:wat::cache::Lru/put cache :a 1)
;;     ->  (:wat::core::Result/expect (:wat::cache::Lru/put cache :a 1)
;;           ":wat::cache::Lru/put refused the key: it must be a hashable value")
;;
;; ⭐ WHY `Result/expect` AND NOT A MATCH. The sites this runs over are FIXTURES, a board
;; specimen and a scratch probe, every one of which puts a hashable key (a keyword, an i64, a
;; `HolonAST`). An `Err` there is not a workload outcome — it is a substrate bug — so the honest
;; disposition is to die loudly naming the verb, the ruling stone A's codemod made for its own
;; wrap. A caller whose key can be an opaque handle must match instead;
;; `wat/cache.wat`'s `HolographicLru/put` is the worked example, hand-written because it
;; PROPAGATES rather than expects. ⚠ `Result/expect` DISCARDS the `Err` payload (stone A's
;; finding), so the verb name survives only because it is written into the message.
;; ⛔ The message names `:wat::cache::Lru/put` for BOTH heads on purpose: `HolographicLru/put` has
;; no guard of its own and re-wraps Lru/put's fault verbatim, so naming itself would lie about
;; where the refusal came from.
;;
;; A discarded site stays a discard, one level in: `_ (… put …)` binds the `Ok` payload instead
;; of the `Result`. Binding the bare `Result` to `_` would also type-check — and would be exactly
;; the silent swallow this stone exists to remove.
;;
;; SPEC: a call to `:wat::cache::Lru/put` or `:wat::cache::HolographicLru/put` becomes that call wrapped as `(:wat::core::Result/expect <call> ":wat::cache::Lru/put refused the key: it must be a hashable value")`.
;;
;; MATCHER (structural): a list whose head keyword is EXACTLY `:wat::cache::Lru/put` or
;; `:wat::cache::HolographicLru/put`. Exactness matters in the usual way — `Lru/get`, `Lru/len`,
;; `Lru/new` share the `:wat::cache::Lru/` prefix and are not touched (`Lru/new` was stone A's).
;; A `defn` NAMING the verb is not a call — its head is `:wat::core::defn` — so it never matches.
;; IDEMPOTENCY: the walk suppresses the top-wrap of a call that is already the scrutinee
;; (child[1]) of a `:wat::core::Result/expect`, so re-run = 0 changes; it still recurses INTO
;; that scrutinee's args, catching a nested call.
;;
;; ⛔ NOT RUN OVER `wat/cache.wat`. That file is the DEFINITION, not a migration target: its four
;; call sites each take a different shape (`HolographicLru/put` PROPAGATES the Result; `HolographicLru/get`
;; and the two service `put` handlers expect with their own messages). A blanket wrap there would
;; have turned `HolographicLru/put`'s propagating body into an expect — the swallow this stone
;; exists to prevent. ⛔ Nor over the stone's own probe fixtures, which MATCH the Result.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-cache-put-in-result-expect.wat

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

;; cache-put-call? — EXACT head match on the two `put` verbs that now return a Result.
(:wat::core::defn :user::cache-put-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [h (:user::head-kw-name node)]
    (:wat::core::if (:wat::core::= h ":wat::cache::Lru/put")
      true
      (:wat::core::= h ":wat::cache::HolographicLru/put"))))

;; already-expected? — a `:wat::core::Result/expect` whose scrutinee (child[1]) is one of those
;; calls. Its scrutinee's top-wrap must be suppressed on re-run (idempotency).
(:wat::core::defn :user::already-expected? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:user::head-kw-name node) ":wat::core::Result/expect")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:user::cache-put-call? (:wat::core::Option/expect (:wat::core::get ch 1) "scrut"))))
    false))

;; ── EDIT: two span inserts wrapping the put call node ───────────────────────
(:wat::core::defn :user::wrap-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple (:user::start-off node lines) ""
      "(:wat::core::Result/expect ")
    (:wat::core::Tuple (:user::end-off node lines) ""
      " \":wat::cache::Lru/put refused the key: it must be a hashable value\")")))

;; recurse a node's children WITHOUT wrapping the node's own top (idempotency suppression).
(:wat::core::defn :user::node-edits-no-top
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) lines)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])))

;; walk one node → its edit (if a put call) + descendants'. For a node already facing an
;; expect, the scrutinee (child[1]) is recursed WITHOUT re-wrapping its top.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::cache-put-call? node)
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
        (:wat::kernel::println (:wat::string::concat "[wrap-cache-put] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
