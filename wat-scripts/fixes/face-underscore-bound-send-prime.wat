;; wat-scripts/fixes/face-underscore-bound-send-prime.wat — arc 278 send'-wall Phase 3b, Move 1
;; (the SWEEP). Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files.
;;
;; THE CHANGE: a `send'` outcome bound to `_` inside a `:wat::core::let` binding vector —
;; `(:wat::core::let [_ (:wat::kernel::send' p m)] …)` — is a swallow through the SAME discard
;; door the do-non-final gate (Phase 3a) already walls. Phase 3b's gate (`src/check.rs`,
;; `process_let_binding`) makes it a compile error too. Landing the gate first would flip all
;; 41 pre-existing `_`-bound `send'` sites RED at once — so this codemod FACES every one of them
;; first: it wraps the `send'` call in the SendOutcome match, so its type becomes `nil` (legal to
;; bind to `_`).
;;
;;   _ (:wat::kernel::send' X Y)
;;     ->
;;   _ (:wat::core::match (:wat::kernel::send' X Y)
;;       (:wat::kernel::SendOutcome::Sent      nil)
;;       (:wat::kernel::SendOutcome::Closed    nil)
;;       ((:wat::kernel::SendOutcome::Lost _c) nil))
;;
;; (Fire-and-continue probe/test sends whose outcome was discarded → all three arms → `nil`. The
;; arm shape is the exemplar `wat/service.wat:995-997`.)
;;
;; MATCHER: walk `:wat::core::let` nodes; take the binding vector (child[1], `ast-kind` "vector");
;; iterate its children as `[name0 rhs0 name1 rhs1 …]` pairs (even index = name, +1 = rhs); for
;; each pair where `name` is the symbol `_` AND `rhs` is a list whose head keyword is
;; `:wat::kernel::send'`, emit the wrap edit on `rhs`. The generic recursion (`seq-edits` over
;; `ast->children`, gated by `:wat::fix::structural?`) still descends everywhere else (nested
;; lets, fn bodies, etc.) — mirrors `wrap-client-method-match-in-recvoutcome.wat`'s shape.
;;
;; IDEMPOTENT BY CONSTRUCTION: after a wrap the rhs head is `:wat::core::match` (not
;; `:wat::kernel::send'`), so a re-run's head-keyword check skips it — 0 changes.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/face-underscore-bound-send-prime.wat

;; ── helpers (mirror wrap-client-method-match-in-recvoutcome.wat) ────────────────
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; underscore-bound-send'? — a binding pair (name, rhs) to wrap: name is the bare symbol `_`,
;; rhs is a list whose head keyword is exactly `:wat::kernel::send'`.
(:wat::core::defn :user::underscore-bound-send'? [name <- :wat::WatAST  rhs <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind name) "symbol")
                    (:wat::core::= (:wat::core::ast-name name) "_") false)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind rhs) "list")
      (:wat::core::let [ch (:wat::core::ast->children rhs)]
        (:wat::core::if (:wat::core::empty? ch)
          false
          (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::kernel::send'")))
      false)
    false))

;; ── EDIT: two span inserts around the send' call (start; end) ───────────────────
(:wat::core::defn :user::wrap-edits
  [rhs <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
    (:wat::core::Tuple (:user::start-off rhs lines) 0 "(:wat::core::match ")
    (:wat::core::Tuple (:user::end-off rhs lines) 0
      " (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))")))

;; pair-edits — walk a let binding vector's children [name0 rhs0 name1 rhs1 …] two at a time
;; (index i = name, i+1 = rhs); emit a wrap for every underscore-bound-send' pair.
(:wat::core::defn :user::pair-edits
  [vch <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let [n (:wat::core::length vch)
                    npairs (:wat::core::i64::/ n 2)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])]) i <- :wat::core::i64]
        -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
        (:wat::core::let [idx  (:wat::core::i64::* i 2)
                          name (:wat::core::Option/expect (:wat::core::get vch idx) "pair name")
                          rhs  (:wat::core::Option/expect (:wat::core::get vch (:wat::core::+ idx 1)) "pair rhs")]
          (:wat::core::if (:user::underscore-bound-send'? name rhs)
            (:wat::core::concat acc (:user::wrap-edits rhs lines))
            acc)))
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
      (:wat::core::range 0 npairs))))

;; let-node? — is this node a `:wat::core::let` list (head keyword exactly `:wat::core::let`)?
(:wat::core::defn :user::let-node? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::let")))
    false))

;; let-edits — for a `:wat::core::let` node, the binding-vector pair-walk edits (empty if the
;; second child isn't a vector — malformed let, leave for the checker to report).
(:wat::core::defn :user::let-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let [ch (:wat::core::ast->children node)]
    (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
      (:wat::core::let [bindings (:wat::core::Option/expect (:wat::core::get ch 1) "bindings")]
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind bindings) "vector")
          (:user::pair-edits (:wat::core::ast->children bindings) lines)
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))))))

;; walk one node → its edits + descendants'. (Recurse into ALL structural children incl. a
;; let's own binding-vector/body — a wrapped send' rhs's new head is `:wat::core::match`, not
;; `:wat::kernel::send'`, so re-descending it is inert; nested lets inside the body are still
;; reached normally.)
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let [this (:wat::core::if (:user::let-node? node)
                            (:user::let-edits node lines)
                            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) lines))
      this)))

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
        (:wat::kernel::println (:wat::core::string::concat "[face-underscore-send] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
