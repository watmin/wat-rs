;; wat-scripts/fixes/wrap-fire-once-in-fireoutcome.wat — arc 278 the FIRE-OUTCOME wall, S2a
;; SCOPE: corpus
;; (`fire-once`, the 31-site verb sent first so the pattern is proven before `fire-rules`' 529).
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;; The exact sibling of wrap-connect-prime-in-connectoutcome.wat (the connect'-wall codemod), and
;; deliberately so: THIS FILE IS THE TOOL THAT WILL SWEEP `fire-rules` TOO. Writing it for 13 sites
;; is not the argument; writing it once for 1_182 is.
;;
;; THE CHANGE: `:wat::rete::fire-once` (and its `$oracle` twin, same TYPE by the dual-impl
;; contract) now returns `(:wat::rete::FireOutcome)` instead of a bare `:wat::rete::Session` — a
;; fire's two ceilings (`max-fire-rounds`, `max-session-bytes`) cannot be proven at load, so the
;; breach is a matchable VALUE the caller faces, never a raise that unwinds past them (ADT; wat has
;; no try/catch). Every CALL SITE that used the bare Session now type-errors. This codemod wraps
;; each call in the FireOutcome match, extracting the session on the happy path and dying loudly on
;; the two ceiling arms — the RULED disposition for the corpus, and the honest one: these fixtures
;; derive a handful of facts against a 1 GiB default, so a ceiling arm here is a SUBSTRATE BUG, not
;; a workload that outgrew its bound. The sibling pattern is spawn.wat's recv'/send' arms.
;;
;;   (fire-once ARG)
;;     ->  (:wat::core::match (fire-once ARG)
;;           [:wat::rete::FireOutcome.Fired {:session __fired} __fired]
;;           [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (assertion-failed! :message "…")]
;;           [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}          (assertion-failed! :message "…")])
;;
;; ⚠ THE BINDERS ARE `__`-PREFIXED, and that is not style. A bare `s`/`p` binder (the connect'
;; codemod's choice) SHADOWS any same-named binding in the enclosing scope, silently, at every one
;; of the sites it rewrites. It got away with it; a sweep of 1_182 sites will not.
;;
;; MATCHER (structural): a list whose head keyword is `:wat::rete::fire-once` or
;; `:wat::rete::fire-once$oracle`.
;; IDEMPOTENCY: the walk suppresses the top-wrap of a call that is already the scrutinee (child[1])
;; of a `:wat::core::match` carrying a `FireOutcome.` arm — so re-run = 0 changes (it still
;; recurses INTO that scrutinee's ARG, catching a nested call).
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-fire-once-in-fireoutcome.wat

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

;; fire-once-call? — a list whose head keyword is `:wat::rete::fire-once` or its `$oracle` twin.
;; BOTH, because the dual-impl contract makes them one type surface: a differential harness that
;; unwrapped only one side would be comparing two different things.
(:wat::core::defn :user::fire-once-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [h (:user::head-kw-name node)]
    (:wat::core::if (:wat::core::= h ":wat::rete::fire-once")
      true
      (:wat::core::= h ":wat::rete::fire-once$oracle"))))

;; arm-head-name — today's match arms are VECTORS (`[:Type.Variant {:k v} body]`); list arms
;; fall through to `wat::fix::arm-head-name` (the grok-era `(pattern body…)` shape).
(:wat::core::defn :user::arm-head-name [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "vector")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    (:wat::fix::arm-head-name arm)))

;; any-arm-head-contains? — does any arm (children[2..]) have a pattern head keyword containing `needle`?
(:wat::core::defn :user::any-arm-head-contains?
  [arms <- (:wat::core::Vector :- [:wat::WatAST])  needle <- :wat::core::String] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  arm <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::string::contains? (:user::arm-head-name arm) needle)))
    false arms))

;; already-facing-fire-match? — a `:wat::core::match` whose scrutinee (child[1]) is a fire-once
;; call AND which already carries a `FireOutcome.` arm (the shape THIS codemod emits). Its
;; scrutinee's top-wrap must be suppressed on re-run (idempotency).
(:wat::core::defn :user::already-facing-fire-match? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:user::head-kw-name node) ":wat::core::match")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::let
          [scrut (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
           arms  (:wat::core::into [] (:wat::core::drop ch 2))]
          (:wat::core::if (:user::fire-once-call? scrut)
            (:user::any-arm-head-contains? arms "FireOutcome.")
            false))))
    false))

;; ── EDIT: two span inserts wrapping the fire-once call node ───────────────────
(:wat::core::defn :user::wrap-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple (:user::start-off node lines) ""
      "(:wat::core::match ")
    (:wat::core::Tuple (:user::end-off node lines) ""
      " [:wat::rete::FireOutcome.Fired {:session __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message \"fire-once: session memory ceiling exceeded\")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message \"fire-once: fixpoint round cap exceeded\")])")))

;; recurse a node's children WITHOUT wrapping the node's own top (idempotency suppression).
(:wat::core::defn :user::node-edits-no-top
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) lines)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])))

;; walk one node → its edit (if a fire-once call) + descendants'. For an already-facing
;; FireOutcome match, the scrutinee (child[1]) is recursed WITHOUT re-wrapping its top.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::fire-once-call? node)
            (:user::wrap-edits node lines)
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
    (:wat::core::if (:user::already-facing-fire-match? node)
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
        (:wat::kernel::println (:wat::string::concat "[wrap-fireoutcome] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
