;; wat-scripts/fixes/mandate-invocation-ctx-param.wat — arc 278 ctx-is-mandatory: STEP 4 codemod.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; Ships ALONGSIDE the `wat/service.wat` macro change (STEP 1-3, this same strike) that makes the
;; old arities illegal: a public `defservice` op arm's param vector `[s req]` becomes `[s ctx req]`
;; (166 sites), and an internal (`-`) op arm's `[s]` becomes `[s ctx]` (2 sites). See
;; docs/arc/2026/06/278-rules-engine/BRIEF-ctx-is-mandatory.md STEP 4 and
;; DESIGN-STONE-mandatory-ctx-and-lifecycle-ops.md "THE SHAPE, RULED".
;;
;; STRUCTURAL, not textual: this walks the parse tree exactly like
;; wat-scripts/census-defservice-arm-arity.wat (the census that measured the 166/2/1 worklist —
;; do not re-derive the worklist with grep, an arm is a structure, not a line) — for each top-level
;; `defservice` form, find its `:impls` child, take its children as arms, and for each arm whose
;; param-vector arity matches the OLD shape for its kind, insert " ctx" as a new second element,
;; right after the `s` binder (span-edit via ast-end-span, comment/whitespace-faithful — mirrors
;; wat-scripts/fixes/declare-max-request-bytes.wat's insertion idiom):
;;
;;   internal (arity 1, `[s]`)      -> `[s ctx]`
;;   public   (arity 2, `[s req]`)  -> `[s ctx req]`
;;
;; Idempotent: an arm already at its NEW arity (2 for internal, 3 for public) is left byte-untouched
;; — re-running emits zero edits. An arm at any OTHER arity is also left untouched (nothing in the
;; corpus has one; the census proved the worklist is exactly 166+2+1).
;;
;; Usage (one EDN vector of paths on stdin — list EVERY path):
;;   printf '["wat-tests/service-stop-resp.wat" ...]\n' \
;;     | cargo wat ./wat-scripts/fixes/mandate-invocation-ctx-param.wat

;; ── small helpers ────────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::ast->source n))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

;; ── is this top-level form a defservice? (mirrors the census exactly) ─────────────────────
(:wat::core::defn :user::defservice-form?
  [form <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [ch (:wat::core::ast->children form)]
    (:wat::core::if (:wat::core::empty? ch)
      false
      (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::service::defservice"))))

;; ── the children FOLLOWING the child whose source is `kw` (the census's own helper) ───────
(:wat::core::defn :user::index-after-keyword
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  kw <- :wat::core::String  i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::i64::>= i (:wat::core::length ch))
    -1
    (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::nth ch i)) kw)
      (:wat::core::i64::+ i 1)
      (:user::index-after-keyword ch kw (:wat::core::i64::+ i 1)))))

;; ── one defservice form → its :impls arms (empty Vector if no :impls) ─────────────────────
(:wat::core::defn :user::arms-of
  [form <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::let [ch  (:wat::core::ast->children form)
                    idx (:user::index-after-keyword ch ":impls" 0)]
    (:wat::core::if (:wat::core::i64::< idx 0)
      (:wat::core::Vector :wat::WatAST)
      (:wat::core::if (:wat::core::i64::>= idx (:wat::core::length ch))
        (:wat::core::Vector :wat::WatAST)
        (:wat::core::ast->children (:wat::core::nth ch idx))))))

;; ── one arm → zero-or-one insertion edit ───────────────────────────────────────────────────
;; arm children: [op-node, param-vec, body]. Insert " ctx" right after the s-binder (param-ch[0])
;; iff the arm is at the OLD arity for its kind (internal: 1; public: 2). Anything else (already
;; migrated, or unrecognized) is left untouched — idempotent, and safe against surprises the
;; census didn't predict.
(:wat::core::defn :user::arm-edit
  [arm <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let [ch (:wat::core::ast->children arm)]
    (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
      (:wat::core::let
        [op-node     (:wat::core::first ch)
         op-str      (:user::kw-name op-node)
         is-internal (:wat::string::starts-with? op-str "-")
         param-vec   (:wat::core::nth ch 1)
         param-ch    (:wat::core::ast->children param-vec)
         arity       (:wat::core::length param-ch)
         needs-edit  (:wat::core::if is-internal
                       (:wat::core::= arity 1)
                       (:wat::core::= arity 2))]
        (:wat::core::if (:wat::core::not needs-edit)
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
          (:wat::core::let
            [s-binder (:wat::core::first param-ch)
             end      (:user::end-off s-binder lines)]
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
              (:wat::core::Tuple end 0 " ctx"))))))))

;; ── all edits for one defservice form's arms ───────────────────────────────────────────────
(:wat::core::defn :user::arms-edits
  [arms <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
                     arm <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::arm-edit arm lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    arms))

;; ── generic tree walk — reaches EVERY defservice, top-level OR nested (a defservice embedded
;; inside a defmacro's quasiquoted template reads as an ordinary
;; `(:wat::core::quasiquote ...)` List, so the generic walk descends into it exactly like any
;; other nested List — mirrors wat-scripts/fixes/declare-max-request-bytes.wat's own walk, which
;; documents the same case for `defsurface`). Discovered the hard way: the top-level-only walk
;; missed tests/macros/probe_arc278_macro_generates_service.wat's `defservice` (it lives inside
;; `:probe::echo-defsvc`'s backtick body), silently leaving one arm unmigrated.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
        (:wat::core::let
          [this (:wat::core::if (:user::defservice-form? node)
                  (:user::arms-edits (:user::arms-of node) lines)
                  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))]
          (:wat::core::concat this (:user::seq-edits ch lines)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) lines)
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
                     it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    items))

;; ── per-file migrate ────────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     eds   (:user::seq-edits forms lines)
     ;; sort by offset ascending, then reverse for right-to-left application (the recursive walk
     ;; is no longer guaranteed left-to-right-by-offset once nested forms are involved — top level
     ;; is visited before descending into it, so a nested edit could sort earlier or later than a
     ;; sibling top-level edit; SORT before reversing to make the ordering safe regardless).
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver ──────────────────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[ctx-param] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
