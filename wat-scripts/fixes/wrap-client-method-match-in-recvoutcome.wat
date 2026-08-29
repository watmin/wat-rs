;; wat-scripts/fixes/wrap-client-method-match-in-recvoutcome.wat — arc 278 the recv'-wall client-method cascade.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; THE CHANGE (RULING 2026-07-22g, four-questions, builder-ratified): a `:nature :Peer` surface
;; client-method (`Store/scan`, `Op1/do-op`, `Journal/write-metrics`, …) now returns
;; `(:wat::kernel::RecvOutcome :- [Response])` instead of a bare `Response` — the transport failure is a
;; matchable VALUE the caller faces (ADT; wat has no try/catch). Every CALL SITE that matched the bare
;; Response now type-errors (`match scrutinee expects X; got (RecvOutcome :- [X])`). This codemod wraps each
;; such match in the RecvOutcome match:
;;
;;   (match SCRUT <Response arms>)
;;     ->  (match SCRUT
;;           ((RecvOutcome::Message __recv) (match __recv <the SAME Response arms, verbatim>))
;;           ((RecvOutcome::Lost __cause)   (assertion-failed! (Failure/message __cause) :None :None))
;;           (RecvOutcome::Closed           (assertion-failed! "recv': peer closed" :None :None)))
;;
;; The `Lost`/`Closed` default is a VISIBLE matched-then-die (an author call site choosing death —
;; R53-sanctioned, NOT a hidden raise). A caller that wants resilience refines its own arms after (a
;; few tests that specifically exercise Lost/Closed — dead_child_speaks, rst — are hand-checked). The
;; SERVICE HANDLERS (journal/span) are NOT here — their Lost/Closed maps to their own Fatal response
;; (keep serving); those 7 were hand-done (per-service semantic, not codemod material).
;;
;; MATCHER (structural, dry-run+diff verified against the checker's TypeMismatch worklist — the checker
;; is the ground-truth site oracle, R52): a `:wat::core::match` list where
;;   - the scrutinee (child[1]) is NOT the symbol `__recv` (idempotency: skip our own inner match), AND
;;   - NO arm's pattern-head keyword contains `RecvOutcome::` (idempotency: skip already-wrapped), AND
;;   - >=1 arm's pattern-head keyword contains `Response::` (a client-method-result match).
;; Idempotent (re-run = 0: the inner match's scrutinee is `__recv`; the outer's first arm is Message).
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/wrap-client-method-match-in-recvoutcome.wat

;; ── helpers (mirror response-record-to-enum.wat) ─────────────────────────────────
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; arm-pattern-head-name — an arm is `(pattern body…)`; if pattern (child[0]) is a list, return its
;; head keyword name; else "" (a `_` wildcard or bare binder pattern has no head keyword).
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
        (:wat::string::contains? (:user::arm-head-name arm) needle)))
    false arms))

;; client-method-match? — a match to wrap: head :wat::core::match, >=3 children (kw scrut arm+),
;; scrutinee not `__recv`, no arm already RecvOutcome::, >=1 arm Response::.
(:wat::core::defn :user::client-method-match? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::match")
          (:wat::core::let
            [scrut (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
             arms  (:wat::core::into [] (:wat::core::drop ch 2))
             scrut-is-recv (:wat::core::if (:wat::core::= (:wat::core::ast-kind scrut) "symbol")
                             (:wat::core::= (:wat::core::ast-name scrut) "__recv") false)]
            (:wat::core::if scrut-is-recv
              false
              (:wat::core::if (:user::any-arm-head-contains? arms "RecvOutcome::")
                false
                (:user::any-arm-head-contains? arms "Resp"))))   ;; "Resp" catches both `…Response::` and `…Resp::`/`…GetResp::`
          false)))
    false))

;; ── EDIT: two span inserts (after scrutinee; after last arm) ──────────────────────
(:wat::core::defn :user::wrap-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [scrut    (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
     last-arm (:wat::core::Option/expect (:wat::core::get ch (:wat::core::- (:wat::core::length ch) 1)) "last")]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple (:user::end-off scrut lines) ""
        " ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv")
      (:wat::core::Tuple (:user::end-off last-arm lines) ""
        ")) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! \"recv': peer closed\" :wat::core::None :wat::core::None))"))))

;; walk one node → its edits + descendants'. (Recurse ALL kids incl. a wrapped match's inner arms —
;; but the inner match's scrutinee is `__recv`, so client-method-match? rejects it: no double-wrap.)
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::client-method-match? node)
            (:user::wrap-edits (:wat::core::ast->children node) lines)
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) lines))
      this)))

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
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[wrap-recvoutcome] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
