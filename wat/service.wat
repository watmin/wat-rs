;; Arc 209 Stone C.1 / C.2 — :wat::service::defservice (PURE-WAT defmacro)
;;
;; C.1 deliverable: the macro skeleton + the OP ENUM only.
;; C.2 deliverable: ALSO emits the REPLY ENUM + the SERVE dispatch loop.
;; Final output: `(:wat::core::do (defenum Op …) (defenum Reply …) (defn serve …))`
;;
;; PROGRAM-BODY path (per feasibility pt 3 + cond template): top-level is a regular
;; form (`let`), params are node-values that `ast->children` accepts, output built with
;; a NESTED quasiquote. A top-level quasiquote would EVALUATE the arg and break
;; `ast->children` (STOP-2). Model: `cond` (wat/core.wat:254) + foundation probe.
;;
;; SINGLE FORMAT: every op clause is the RPC shape
;;   (:Op [s <- :State …in] -> [..out fields] body)
;; ch[0]=opkw ch[1]=in-argvec ch[2]=-> ch[3]=out-fieldvec ch[4]=body
;; No dual-format / detection branch (hard-cut; no old (:Tuple :State :T) shim).

;; ── Outcome<S,R> — the handler result (the gen_server callback-return model) ──────────
;;
;; A handler is a PURE transform `(s <- :State, …in) -> :Outcome<:State, <fqdn>::Reply>`.
;; It returns what to DO: reply-and-continue (C.2), and later (C.4) no-reply / stop. This
;; is OTP gen_server's `{reply,R,S} | {noreply,S} | {stop,…}` re-derived as a wat tagged sum
;; (named — NOT a bare `(:Tuple state reply)`; a structured result with distinct roles is a
;; record/sum, per the ADT identity, not an order-convention pair). Generic + stdlib: every
;; service reuses it (not minted per-service). C.4 GROWS it by ADDING variants — no reshape.
(:wat::core::defenum :wat::service::Outcome<S,R>
  :Reply [state <- :S  reply <- :R])

(:wat::core::defmacro :wat::service::defservice
  [fqdn      <- :wat::WatAST     ;; :my::counter
   _state-kw <- :wat::WatAST     ;; the literal :state marker (ignored)
   state-ty  <- :wat::WatAST     ;; :wat::core::i64  (used in serve params)
   _ops-kw   <- :wat::WatAST     ;; the literal :ops marker (ignored)
   ops       <- :wat::WatAST]    ;; the [ (:Get …) (:Increment …) ] vector NODE
  -> :wat::WatAST
  ;; PROGRAM-BODY path: top-level `let`, params are node-values, nested quasiquote at the end.
  (:wat::core::let
    [fqdn-str      (:wat::core::keyword/to-string fqdn)
     enum-name     (:wat::core::keyword/from-string
                     (:wat::core::string::concat fqdn-str "::Op"))
     reply-name    (:wat::core::keyword/from-string
                     (:wat::core::string::concat fqdn-str "::Reply"))
     serve-name    (:wat::core::keyword/from-string
                     (:wat::core::string::concat fqdn-str "::serve"))
     ;; Parametric type keywords for serve's typed params.
     ;; Peer'<fqdn::Reply,fqdn::Op>
     peer-ty       (:wat::core::keyword/from-string
                     (:wat::core::string::concat "wat::kernel::Peer'<"
                       (:wat::core::string::concat fqdn-str
                         (:wat::core::string::concat "::Reply,"
                           (:wat::core::string::concat fqdn-str "::Op>")))))
     ;; Listener'<fqdn::Op,fqdn::Reply>
     listener-ty   (:wat::core::keyword/from-string
                     (:wat::core::string::concat "wat::kernel::Listener'<"
                       (:wat::core::string::concat fqdn-str
                         (:wat::core::string::concat "::Op,"
                           (:wat::core::string::concat fqdn-str "::Reply>")))))
     ;; Vector<Peer'<fqdn::Reply,fqdn::Op>>
     vector-ty     (:wat::core::keyword/from-string
                     (:wat::core::string::concat "wat::core::Vector<wat::kernel::Peer'<"
                       (:wat::core::string::concat fqdn-str
                         (:wat::core::string::concat "::Reply,"
                           (:wat::core::string::concat fqdn-str "::Op>>")))))
     clauses       (:wat::core::ast->children ops)            ;; list of op-List nodes

     ;; ── C.1: Op variants (KEEP) ───────────────────────────────────────────────────
     ;; variant per op = the in-argvec MINUS the leading `s <- :State` triple
     variants      (:wat::core::foldl
                     (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                      clause <- :wat::WatAST]
                       -> :wat::core::Vector<wat::WatAST>
                       (:wat::core::let
                         [ch      (:wat::core::ast->children clause)
                          opkw    (:wat::core::Option/expect -> :wat::WatAST
                                    (:wat::core::first ch)
                                    "defservice: op-clause has no head")
                          argvec  (:wat::core::Option/expect -> :wat::WatAST
                                    (:wat::core::first (:wat::core::drop ch 1))
                                    "defservice: op-clause has no arg-vec")
                          fieldch (:wat::core::drop (:wat::core::ast->children argvec) 3)]
                         (:wat::core::if (:wat::core::empty? fieldch)
                           (:wat::core::conj acc opkw)
                           (:wat::core::conj (:wat::core::conj acc opkw)
                                             (:wat::core::with-children argvec fieldch)))))
                     (:wat::core::Vector :wat::WatAST)
                     clauses)

     ;; ── C.2: Reply variants (NEW) ─────────────────────────────────────────────────
     ;; variant per op = the out-fieldvec (ch[3]) verbatim
     reply-variants (:wat::core::foldl
                      (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                       clause <- :wat::WatAST]
                        -> :wat::core::Vector<wat::WatAST>
                        (:wat::core::let
                          [ch           (:wat::core::ast->children clause)
                           opkw         (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first ch)
                                          "defservice reply: op-clause has no head")
                           out-fieldvec (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first (:wat::core::drop ch 3))
                                          "defservice reply: op-clause has no out-fieldvec")
                           reply-fieldch (:wat::core::ast->children out-fieldvec)]
                          (:wat::core::if (:wat::core::empty? reply-fieldch)
                            (:wat::core::conj acc opkw)
                            (:wat::core::conj (:wat::core::conj acc opkw)
                                              (:wat::core::with-children out-fieldvec reply-fieldch)))))
                      (:wat::core::Vector :wat::WatAST)
                      clauses)

     ;; ── C.2: serve op-arms (NEW) ──────────────────────────────────────────────────
     ;; One match arm per op inside the :Message arm's (match op …).
     ;; Bare pattern if no input fields; destructuring pattern if has fields.
     ;; Body: (match (let [~s state] <inline-body>) -> nil
     ;;              ((Outcome::Reply ns r) (do (send' ...) (serve …))))
     ;;
     ;; Hygiene: literal Symbol binders in quasiquote templates are refused
     ;; (ProgramBodyIntroducesName gate). Fixes:
     ;;  (a) The state binder `s` is extracted from the user's argvec first-child
     ;;      and unquoted (~state-binder), so it comes from the caller, not the template.
     ;;  (b) The `_` discard binder for send' is replaced with (:wat::core::do ...)
     ;;      to sequence the effect without introducing a literal binder.
     ;; Arg-names are extracted inline using map+range+get (no helper defn needed;
     ;; user-defined fns are not on the pure-combinator allow-list for macro bodies).
     serve-op-arms (:wat::core::foldl
                     (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                      clause <- :wat::WatAST]
                       -> :wat::core::Vector<wat::WatAST>
                       (:wat::core::let
                         [ch            (:wat::core::ast->children clause)
                          opkw          (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first ch)
                                          "defservice serve-arm: op-clause has no head")
                          argvec        (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first (:wat::core::drop ch 1))
                                          "defservice serve-arm: op-clause has no arg-vec")
                          body          (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first (:wat::core::drop ch 4))
                                          "defservice serve-arm: op-clause has no body")
                          fieldch       (:wat::core::drop (:wat::core::ast->children argvec) 3)
                          op-variant-kw (:wat::core::keyword/from-string
                                          (:wat::core::string::concat fqdn-str
                                            (:wat::core::string::concat "::Op::"
                                              (:wat::core::keyword/to-string opkw))))
                          ;; Extract the state binder symbol (e.g. `s`) from the first
                          ;; triple of the in-argvec. Must be unquoted in the generated
                          ;; `let` form to pass the ProgramBodyIntroducesName hygiene check.
                          state-binder  (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first (:wat::core::ast->children argvec))
                                          "defservice serve-arm: argvec has no state binder")
                          ;; Arg-name symbols at positions 0, 3, 6, … in fieldch.
                          ;; Inline extraction via map+range+get (no helper defn; user-defined
                          ;; fns are not on the pure-combinator allow-list).
                          fieldch-len   (:wat::core::length fieldch)
                          n-args        (:wat::core::i64::/ fieldch-len 3)
                          arg-indices   (:wat::core::map
                                          (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
                                            (:wat::core::i64::* i 3))
                                          (:wat::core::range 0 n-args))
                          arg-names     (:wat::core::map
                                          (:wat::core::fn [i <- :wat::core::i64] -> :wat::WatAST
                                            (:wat::core::Option/expect -> :wat::WatAST
                                              (:wat::core::get fieldch i)
                                              "defservice serve-arm: fieldch index out of bounds"))
                                          arg-indices)
                          ;; Outcome match body: shared by all arms.
                          outcome-match `(:wat::core::match
                                              (:wat::core::let [~state-binder state] ~body)
                                              -> :wat::core::nil
                                            ((:wat::service::Outcome::Reply new-state reply)
                                              (:wat::core::do
                                                (:wat::kernel::send'
                                                  (:wat::core::nth clients idx)
                                                  reply)
                                                (~serve-name self l clients new-state))))
                          arm           (:wat::core::if (:wat::core::empty? fieldch)
                                          `(~op-variant-kw ~outcome-match)
                                          `((~op-variant-kw ~@arg-names) ~outcome-match))]
                         (:wat::core::conj acc arm)))
                     (:wat::core::Vector :wat::WatAST)
                     clauses)

     ;; ── serve params argvec ───────────────────────────────────────────────────────
     ;; Template is a Vector node; check_quasiquote_for_literal_binders only
     ;; processes List nodes, so self/l/clients/state as Vector elements are fine.
     serve-params `[self    <- ~peer-ty
                    l       <- ~listener-ty
                    clients <- ~vector-ty
                    state   <- ~state-ty]

     ;; ── serve body: the poll'/ServiceEvent dispatch loop ─────────────────────────
     serve-body   `(:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
                     (:wat::kernel::ServiceEvent::Shutdown nil)
                     ((:wat::kernel::ServiceEvent::Connection peer)
                       (~serve-name self l (:wat::core::conj clients peer) state))
                     ((:wat::kernel::ServiceEvent::Message idx op)
                       (:wat::core::match op -> :wat::core::nil
                         ~@serve-op-arms))
                     ((:wat::kernel::ServiceEvent::Closed idx)
                       (~serve-name self l (:wat::std::list::remove-at clients idx) state))
                     ((:wat::kernel::ServiceEvent::Lost idx _cause)
                       (~serve-name self l (:wat::std::list::remove-at clients idx) state)))]
    ;; Wrap in `do`: defenum forms get splice_type_decl'd to top-level; `defn serve`
    ;; keeps the `do` non-empty (a `do` with only type decls fails check — defn is not
    ;; a type decl so this `do` is always non-empty after stripping).
    `(:wat::core::do
       (:wat::core::defenum ~enum-name ~@variants)
       (:wat::core::defenum ~reply-name ~@reply-variants)
       (:wat::core::defn ~serve-name ~serve-params -> :wat::core::nil ~serve-body))))
