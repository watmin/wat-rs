;; wat-scripts/fixes/struct-new-failure-to-message-only-failure.wat — arc 278 item-c Strike B
;; reclamation.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; THE CHANGE: Strike B's src/check.rs struct-new NATURE WALL makes
;; `(:wat::core::struct-new :wat::kernel::Failure …)` a located compile error — struct-new mints
;; a Nature::Struct value, but Failure is Nature::Record (arc 293.W.2b: a crash cause crosses the
;; wire, pure EDN, only a Record round-trips it). Strike A already gave the corpus the one
;; canonical message-only constructor (`wat/spawn.wat`): `(:wat::kernel::message-only-failure
;; msg)`, whose body hardcodes the exact same defaulted tail every hand-rolled struct-new site
;; below repeats verbatim (location :None, frames empty (Vector :- [Frame]), actual :None, expected
;; :None). Every message-only struct-new site collapses to that one call:
;;
;;   (:wat::core::struct-new :wat::kernel::Failure MSG :wat::core::None
;;      (:wat::core::Vector :wat::kernel::Frame) :wat::core::None :wat::core::None)
;;     ->  (:wat::kernel::message-only-failure MSG)
;;
;; MATCHER (structural): a list node whose children[0] keyword is `:wat::core::struct-new` and
;; children[1] keyword is `:wat::kernel::Failure` (>=3 children — msg present). MSG (children[2])
;; is re-emitted VERBATIM by its own source span — whatever expression it is (a string literal or
;; a nested call like `(:wat::core::string::concat "startup: " …)`) — so no shape assumption is
;; made about MSG itself. The whole matched node's span (head through closing paren) is replaced
;; by one edit; the trailing default-shaped args are dropped since message-only-failure already
;; hardcodes that exact tail.
;;
;; Idempotent (re-run = 0 edits: the rewritten head is message-only-failure, not struct-new).
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' | cargo wat ./wat-scripts/fixes/struct-new-failure-to-message-only-failure.wat

;; ── small helpers (mirrors response-record-to-enum.wat / eprintln-recv-arm-to-assertion-failed.wat) ──
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; struct-new-failure? — a list `(:wat::core::struct-new :wat::kernel::Failure MSG …)`.
(:wat::core::defn :user::struct-new-failure? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::struct-new")
          (:wat::core::= (:user::kw-name (:wat::core::Option/expect (:wat::core::get ch 1) "type-kw")) ":wat::kernel::Failure")
          false)))
    false))

;; ── EDIT: replace the whole node's span with `(:wat::kernel::message-only-failure MSG)` ──
(:wat::core::defn :user::struct-new-failure-edit
  [node <- :wat::WatAST
   ch   <- (:wat::core::Vector :- [:wat::WatAST])
   src  <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  ;; old-text = fix-text-span-text over the WHOLE matched node's OWN span (arc 282) —
  ;; sanctioned: struct-new-failure? already verified this node's identity structurally
  ;; (ch[0]/ch[1] names), the entire call form (including any trailing default args) is
  ;; being replaced wholesale, and a List's own span can never diverge from its literal
  ;; text the way a reader-synthesized leaf's can — there is no separate name-based claim
  ;; narrower than "this whole call" to check against.
  (:wat::core::let
    [msg      (:wat::core::Option/expect (:wat::core::get ch 2) "msg")
     n0       (:user::start-off node lines)
     old-text (:wat::fix::fix-text-span-text (:wat::core::ast-span node) (:wat::core::ast-end-span node) lines src)
     msg-txt  (:wat::string::subs src (:user::start-off msg lines) (:user::end-off msg lines))]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple n0 old-text
        (:wat::string::concat "(:wat::kernel::message-only-failure " msg-txt ")")))))

;; walk one node → its edits + descendants'. A matched struct-new-failure node does NOT recurse
;; into its own children (MSG is re-emitted verbatim by span, not walked; the defaulted tail args
;; are dropped whole) — the single edit covers the entire node.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::struct-new-failure? node)
    (:user::struct-new-failure-edit node (:wat::core::ast->children node) src lines)
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) src lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it src lines)))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    items))

;; ── per-file migrate ─────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
     eds   (:user::seq-edits forms src lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[struct-new-Failure->message-only-failure] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
