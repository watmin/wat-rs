;; Arc 209 Stone C.1 / C.2 / C.3 — :wat::service::defservice (PURE-WAT defmacro)
;;
;; C.1 deliverable: the macro skeleton + the OP ENUM only.
;; C.2 deliverable: ALSO emits the REPLY ENUM + the SERVE dispatch loop.
;; C.3 deliverable: REFINES to full-gRPC: per-op Request + Response records,
;;   Op/Reply WRAP them (one field: req/resp), serve unwraps+wraps,
;;   ADD client face (constructors, methods, start fn, Handle record).
;; Final output order:
;;   records (Request + Response, per op) → enums (Op, Reply) → serve defn →
;;   constructors (per op) → methods (per op) → start defn → Handle record
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
;;
;; HYGIENE (ProgramBodyIntroducesName gate — expand.rs:558):
;;   The check fires on literal Symbol at EVEN indices in a `let` binder Vector, or ANY
;;   Symbol in a `fn` param Vector, when those Vectors appear DIRECTLY inside a quasiquote
;;   template (the checker stops at nested quasiquotes; it does NOT recurse into Vectors
;;   — only into List forms). It skips match-arm pattern binders (not let/fn heads).
;;
;;   Fixes used throughout:
;;   (a) User-provided binders (state-binder `s`) extracted via ast->children and unquoted
;;       so they appear as Unquote nodes at definition time → checker skips them.
;;   (b) Synthetic let/fn binders needed inside nested quasiquotes (pair/l/addr/svc/self for
;;       start; _/r for methods) built via `(:wat::core::symbol-node "name")` and unquoted.
;;   (c) let-bindings for serve arms built as a WatAST::Vector via `with-children` and used
;;       as `~let-bindings` — the checker sees an Unquote at the binder-vector slot → passes.
;;   (d) Match-arm pattern binders (req/new-state/resp/peer/idx) in match sub-forms →
;;       checker skips them (not let/fn heads).
;;   (e) Vectors used as defenum field lists and serve/method/start params: the checker
;;       does NOT recurse into Vector nodes' children (only into List-headed forms).

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
   state-ty  <- :wat::WatAST     ;; :wat::core::i64  (used in serve/start params)
   _ops-kw   <- :wat::WatAST     ;; the literal :ops marker (ignored)
   ops       <- :wat::WatAST]    ;; the [ (:Get …) (:Increment …) ] vector NODE
  -> :wat::WatAST
  ;; PROGRAM-BODY path: top-level `let`, params are node-values, nested quasiquote at the end.
  (:wat::core::let
    [fqdn-str      (:wat::core::keyword/to-string fqdn)
     ;; Arc 265 — reconstruct fqdn as a keyword value so pascal->kebab-in
     ;; can use it as the namespace for acronym-registry lookup.
     fqdn-kw       (:wat::core::keyword/from-string fqdn-str)
     enum-name     (:wat::core::keyword/from-string
                     (:wat::core::string::concat fqdn-str "::Op"))
     reply-name    (:wat::core::keyword/from-string
                     (:wat::core::string::concat fqdn-str "::Reply"))
     serve-name    (:wat::core::keyword/from-string
                     (:wat::core::string::concat fqdn-str "::serve"))
     ;; Arc 209 host-parity-4a — the serve fqdn as a STRING, spliced into start's
     ;; `(keyword/from-string …)` so Host/launch receives serve by a RUNTIME keyword
     ;; (a spliced literal `:fqdn::serve` would Arc-009-resolve to a Fn, not a keyword).
     serve-name-str (:wat::core::string::concat fqdn-str "::serve")
     start-name    (:wat::core::keyword/from-string
                     (:wat::core::string::concat fqdn-str "/start"))
     handle-name   (:wat::core::keyword/from-string
                     (:wat::core::string::concat fqdn-str "::Handle"))
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
     ;; Address'<fqdn::Op,fqdn::Reply>
     addr-ty       (:wat::core::keyword/from-string
                     (:wat::core::string::concat "wat::kernel::Address'<"
                       (:wat::core::string::concat fqdn-str
                         (:wat::core::string::concat "::Op,"
                           (:wat::core::string::concat fqdn-str "::Reply>")))))
     ;; Client Peer'<fqdn::Op,fqdn::Reply> — connect'(Address'<Op,Reply>) → Peer'<Op,Reply>.
     ;; This is the client-side peer (sends Op, receives Reply); distinct from
     ;; peer-ty (Peer'<Reply,Op>) which is the server-side peer (accepts via listener').
     client-peer-ty (:wat::core::keyword/from-string
                      (:wat::core::string::concat "wat::kernel::Peer'<"
                        (:wat::core::string::concat fqdn-str
                          (:wat::core::string::concat "::Op,"
                            (:wat::core::string::concat fqdn-str "::Reply>")))))
     clauses       (:wat::core::ast->children ops)            ;; list of op-List nodes

     ;; ── C.3: per-op Request records ───────────────────────────────────────────────
     ;; Request = Record::def with the in-fields MINUS the leading s <- :State triple.
     ;; Emitted BEFORE the enums (record before the enum that references it as a field type).
     request-records (:wat::core::foldl
                       (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                        clause <- :wat::WatAST]
                         -> :wat::core::Vector<wat::WatAST>
                         (:wat::core::let
                           [ch          (:wat::core::ast->children clause)
                            opkw        (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first ch)
                                          "defservice request-records: op-clause has no head")
                            argvec      (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first (:wat::core::drop ch 1))
                                          "defservice request-records: op-clause has no arg-vec")
                            ;; in-fields = drop the leading s <- :State triple (3 nodes)
                            in-fieldch  (:wat::core::drop (:wat::core::ast->children argvec) 3)
                            op-str      (:wat::core::keyword/to-string opkw)
                            req-name    (:wat::core::keyword/from-string
                                          (:wat::core::string::concat fqdn-str
                                            (:wat::core::string::concat "::" op-str "Request")))
                            ;; Reuse argvec as the Vector carrier; with-children replaces children
                            req-fieldvec (:wat::core::with-children argvec in-fieldch)]
                           (:wat::core::conj acc
                             `(:wat::Record::def ~req-name ~req-fieldvec))))
                       (:wat::core::Vector :wat::WatAST)
                       clauses)

     ;; ── C.3: per-op Response records ──────────────────────────────────────────────
     ;; Response = Record::def with the out-fields (ch[3] verbatim).
     response-records (:wat::core::foldl
                        (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                         clause <- :wat::WatAST]
                          -> :wat::core::Vector<wat::WatAST>
                          (:wat::core::let
                            [ch           (:wat::core::ast->children clause)
                             opkw         (:wat::core::Option/expect -> :wat::WatAST
                                            (:wat::core::first ch)
                                            "defservice response-records: op-clause has no head")
                             out-fieldvec (:wat::core::Option/expect -> :wat::WatAST
                                            (:wat::core::first (:wat::core::drop ch 3))
                                            "defservice response-records: op-clause has no out-fieldvec")
                             out-fieldch  (:wat::core::ast->children out-fieldvec)
                             op-str       (:wat::core::keyword/to-string opkw)
                             resp-name    (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::concat "::" op-str "Response")))
                             ;; Reuse out-fieldvec as the Vector carrier
                             resp-fieldvec (:wat::core::with-children out-fieldvec out-fieldch)]
                            (:wat::core::conj acc
                              `(:wat::Record::def ~resp-name ~resp-fieldvec))))
                        (:wat::core::Vector :wat::WatAST)
                        clauses)

     ;; ── C.3: Op variants — each WRAPS the Request record (one field: req) ─────────
     ;; variant: <opkw> [req <- :<fqdn>::<Op>Request]
     ;; `[req <- ~req-ty]` is a Vector quasiquote inner; the checker doesn't recurse into
     ;; Vector nodes' children, so `req` as a field name label is fine.
     variants      (:wat::core::foldl
                     (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                      clause <- :wat::WatAST]
                       -> :wat::core::Vector<wat::WatAST>
                       (:wat::core::let
                         [ch      (:wat::core::ast->children clause)
                          opkw    (:wat::core::Option/expect -> :wat::WatAST
                                    (:wat::core::first ch)
                                    "defservice variants: op-clause has no head")
                          argvec  (:wat::core::Option/expect -> :wat::WatAST
                                    (:wat::core::first (:wat::core::drop ch 1))
                                    "defservice variants: op-clause has no arg-vec")
                          op-str  (:wat::core::keyword/to-string opkw)
                          req-ty  (:wat::core::keyword/from-string
                                    (:wat::core::string::concat fqdn-str
                                      (:wat::core::string::concat "::" op-str "Request")))
                          ;; Build [req <- <req-ty>] as a quasiquoted Vector node.
                          ;; Value::wat__WatAST(WatAST::Vector) — spliced via ~@variants correctly.
                          req-field-vec `[req <- ~req-ty]]
                         (:wat::core::conj (:wat::core::conj acc opkw)
                                           req-field-vec)))
                     (:wat::core::Vector :wat::WatAST)
                     clauses)

     ;; ── C.3: Reply variants — each WRAPS the Response record (one field: resp) ────
     ;; variant: <opkw> [resp <- :<fqdn>::<Op>Response]
     reply-variants (:wat::core::foldl
                      (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                       clause <- :wat::WatAST]
                        -> :wat::core::Vector<wat::WatAST>
                        (:wat::core::let
                          [ch      (:wat::core::ast->children clause)
                           opkw    (:wat::core::Option/expect -> :wat::WatAST
                                     (:wat::core::first ch)
                                     "defservice reply-variants: op-clause has no head")
                           out-fieldvec (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first (:wat::core::drop ch 3))
                                          "defservice reply-variants: op-clause has no out-fieldvec")
                           op-str  (:wat::core::keyword/to-string opkw)
                           resp-ty (:wat::core::keyword/from-string
                                     (:wat::core::string::concat fqdn-str
                                       (:wat::core::string::concat "::" op-str "Response")))
                           ;; Build [resp <- <resp-ty>] as a quasiquoted Vector node.
                           resp-field-vec `[resp <- ~resp-ty]]
                          (:wat::core::conj (:wat::core::conj acc opkw)
                                            resp-field-vec)))
                      (:wat::core::Vector :wat::WatAST)
                      clauses)

     ;; ── C.3: serve op-arms ───────────────────────────────────────────────────────
     ;; Each arm:
     ;;   ((Op::<Op> req) (match (let ~let-bindings body) -> nil
     ;;                     ((Outcome::Reply new-state resp)
     ;;                       (do (send' (nth clients idx) (Reply::<Op> resp)) (serve …)))))
     ;;
     ;; let-bindings is built as a WatAST::Vector (via with-children on argvec), so that
     ;; `~let-bindings` at expansion time uses value_to_watast → Value::wat__WatAST → the
     ;; Vector node. At definition time it's an Unquote → binder-slot check skipped.
     ;;
     ;; Hygiene notes:
     ;;   state-binder from user's argvec → unquoted (Unquote at def time).
     ;;   req in Op match pattern → match-arm binder (not let/fn) → fine as literal.
     ;;   new-state, resp in Outcome match pattern → match-arm binders → fine as literal.
     ;;   self, l, clients, idx in value positions → fine as literals.
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
                          ;; in-fields MINUS leading s <- :State triple (3 nodes)
                          fieldch       (:wat::core::drop (:wat::core::ast->children argvec) 3)
                          op-str        (:wat::core::keyword/to-string opkw)
                          op-variant-kw (:wat::core::keyword/from-string
                                          (:wat::core::string::concat fqdn-str
                                            (:wat::core::string::concat "::Op::" op-str)))
                          reply-variant-kw (:wat::core::keyword/from-string
                                             (:wat::core::string::concat fqdn-str
                                               (:wat::core::string::concat "::Reply::" op-str)))
                          ;; Extract state binder (e.g. `s`) from the first triple of argvec.
                          ;; Unquoted → Unquote node at definition time → passes hygiene check.
                          state-binder  (:wat::core::Option/expect -> :wat::WatAST
                                          (:wat::core::first (:wat::core::ast->children argvec))
                                          "defservice serve-arm: argvec has no state binder")
                          ;; Build accessor keywords: <fqdn>::<Op>Request/<field-name>
                          ;; Field names at positions 0, 3, 6, … in fieldch.
                          fieldch-len   (:wat::core::length fieldch)
                          n-args        (:wat::core::i64::/ fieldch-len 3)
                          arg-indices   (:wat::core::map
                                          (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
                                            (:wat::core::i64::* i 3))
                                          (:wat::core::range 0 n-args))
                          ;; The field name AST nodes (symbols like `n`)
                          arg-names     (:wat::core::map
                                          (:wat::core::fn [i <- :wat::core::i64] -> :wat::WatAST
                                            (:wat::core::Option/expect -> :wat::WatAST
                                              (:wat::core::get fieldch i)
                                              "defservice serve-arm: arg name out of bounds"))
                                          arg-indices)
                          ;; Accessor keyword nodes: <fqdn>::<Op>Request/<field-name>
                          ;; Field names are Symbol nodes (e.g. `n`), not Keywords.
                          ;; Use ast-name (handles both Symbol and Keyword) to get the text.
                          arg-accessors (:wat::core::map
                                          (:wat::core::fn [i <- :wat::core::i64] -> :wat::WatAST
                                            (:wat::core::let
                                              [name-node (:wat::core::Option/expect -> :wat::WatAST
                                                            (:wat::core::get fieldch i)
                                                            "defservice serve-arm: accessor out of bounds")
                                               name-str  (:wat::core::ast-name name-node)]
                                              (:wat::core::keyword/from-string
                                                (:wat::core::string::concat fqdn-str
                                                  (:wat::core::string::concat "::" op-str
                                                    (:wat::core::string::concat "Request/"
                                                      name-str))))))
                                          arg-indices)
                          ;; `req` symbol node — used as value reference in accessor calls:
                          ;; `(acc-kw req)`. The `req` match-binder in the arm pattern is a
                          ;; literal in the quasiquote, but it's in a match pattern (List whose
                          ;; head is op-variant-kw, not `let`/`fn`) → the checker skips it.
                          req-sym       (:wat::core::symbol-node "req")
                          ;; `state` symbol node — used as value reference in let-bindings:
                          ;; `[s state ...]` where `state` is the serve param.
                          state-sym     (:wat::core::symbol-node "state")
                          ;; Build the let-binding items as a Value::Vec:
                          ;;   [state-binder, state-sym, arg0, (acc0 req), arg1, (acc1 req), …]
                          binding-items (:wat::core::foldl
                                          (:wat::core::fn [bind-acc <- :wat::core::Vector<wat::WatAST>
                                                           i <- :wat::core::i64]
                                            -> :wat::core::Vector<wat::WatAST>
                                            (:wat::core::let
                                              [arg-name (:wat::core::Option/expect -> :wat::WatAST
                                                           (:wat::core::get arg-names i)
                                                           "defservice serve-arm: arg-name index")
                                               acc-kw   (:wat::core::Option/expect -> :wat::WatAST
                                                           (:wat::core::get arg-accessors i)
                                                           "defservice serve-arm: accessor index")]
                                              (:wat::core::conj
                                                (:wat::core::conj bind-acc arg-name)
                                                `(~acc-kw ~req-sym))))
                                          (:wat::core::conj
                                            (:wat::core::conj
                                              (:wat::core::Vector :wat::WatAST)
                                              state-binder)
                                            state-sym)
                                          (:wat::core::range 0 n-args))
                          ;; Convert binding items to a WatAST::Vector via with-children.
                          ;; Now let-bindings is Value::wat__WatAST(WatAST::Vector([...])),
                          ;; so `~let-bindings` in outcome-match uses value_to_watast correctly.
                          let-bindings  (:wat::core::with-children argvec binding-items)
                          outcome-match `(:wat::core::match
                                              (:wat::core::let ~let-bindings ~body)
                                              -> :wat::core::nil
                                            ((:wat::service::Outcome::Reply new-state resp)
                                              (:wat::core::do
                                                (:wat::kernel::send'
                                                  (:wat::core::nth clients idx)
                                                  (~reply-variant-kw resp))
                                                (~serve-name self l clients new-state))))]
                         (:wat::core::conj acc
                           `((~op-variant-kw req) ~outcome-match))))
                     (:wat::core::Vector :wat::WatAST)
                     clauses)

     ;; ── serve params argvec ───────────────────────────────────────────────────────
     ;; Template is a Vector node; checker does NOT recurse into Vector children.
     ;; self/l/clients/state in the Vector are fine as literal symbols.
     serve-params `[self    <- ~peer-ty
                    l       <- ~listener-ty
                    clients <- ~vector-ty
                    state   <- ~state-ty]

     ;; ── serve body: the poll'/ServiceEvent dispatch loop ─────────────────────────
     ;; All literals (self, l, clients, state, peer, idx, _cause) are in match patterns
     ;; or value positions — the checker only fires for let/fn binder Vectors.
     serve-body   `(:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
                     (:wat::spawn::ServiceEvent::Shutdown nil)
                     ((:wat::spawn::ServiceEvent::Connection peer)
                       (~serve-name self l (:wat::core::conj clients peer) state))
                     ((:wat::spawn::ServiceEvent::Message idx op)
                       (:wat::core::match op -> :wat::core::nil
                         ~@serve-op-arms))
                     ((:wat::spawn::ServiceEvent::Closed idx)
                       (~serve-name self l (:wat::std::list::remove-at clients idx) state))
                     ((:wat::spawn::ServiceEvent::Lost idx _cause)
                       (~serve-name self l (:wat::std::list::remove-at clients idx) state)))

     ;; ── C.3: request constructors ────────────────────────────────────────────────
     ;; For each op:
     ;;   (defn <fqdn>/<op-lower>-request [<in-fields>] -> :<fqdn>::<Op>Request
     ;;     (<fqdn>::<Op>Request <in-field-names>))
     ;; Constructor name uses pascal->kebab-in (namespace-aware; Arc 265) so a
     ;; namespace with declared acronyms gets correct kebab lowering (e.g.
     ;; :CreateWebACL → "create-web-acl" when "ACL" is declared for the service ns).
     ;; ctor-body = `(~req-ty ~@arg-names)`: head is Unquote → checker skips let/fn check.
     constructors  (:wat::core::foldl
                     (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                      clause <- :wat::WatAST]
                       -> :wat::core::Vector<wat::WatAST>
                       (:wat::core::let
                         [ch           (:wat::core::ast->children clause)
                          opkw         (:wat::core::Option/expect -> :wat::WatAST
                                         (:wat::core::first ch)
                                         "defservice constructors: op-clause has no head")
                          argvec       (:wat::core::Option/expect -> :wat::WatAST
                                         (:wat::core::first (:wat::core::drop ch 1))
                                         "defservice constructors: op-clause has no arg-vec")
                          ;; in-fields minus the leading s <- :State triple
                          in-fieldch   (:wat::core::drop (:wat::core::ast->children argvec) 3)
                          op-str       (:wat::core::keyword/to-string opkw)
                          op-lower     (:wat::core::string::pascal->kebab-in fqdn-kw op-str)
                          ctor-name    (:wat::core::keyword/from-string
                                         (:wat::core::string::concat fqdn-str
                                           (:wat::core::string::concat "/" op-lower "-request")))
                          req-ty       (:wat::core::keyword/from-string
                                         (:wat::core::string::concat fqdn-str
                                           (:wat::core::string::concat "::" op-str "Request")))
                          ;; Use argvec as Vector carrier for the parameter list
                          req-fieldvec  (:wat::core::with-children argvec in-fieldch)
                          ;; Extract field names for the constructor call body
                          fieldch-len  (:wat::core::length in-fieldch)
                          n-args       (:wat::core::i64::/ fieldch-len 3)
                          arg-indices  (:wat::core::map
                                         (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
                                           (:wat::core::i64::* i 3))
                                         (:wat::core::range 0 n-args))
                          arg-names    (:wat::core::map
                                         (:wat::core::fn [i <- :wat::core::i64] -> :wat::WatAST
                                           (:wat::core::Option/expect -> :wat::WatAST
                                             (:wat::core::get in-fieldch i)
                                             "defservice constructors: field name out of bounds"))
                                         arg-indices)
                          ;; ctor-body: (~req-ty ~@arg-names) — head is Unquote, so checker
                          ;; doesn't fire on any `let`/`fn` check; arg-names are user symbols.
                          ctor-body    `(~req-ty ~@arg-names)]
                         (:wat::core::conj acc
                           `(:wat::core::defn ~ctor-name ~req-fieldvec -> ~req-ty ~ctor-body))))
                     (:wat::core::Vector :wat::WatAST)
                     clauses)

     ;; ── C.3: methods ─────────────────────────────────────────────────────────────
     ;; For each op:
     ;;   (defn <fqdn>/<op-lower> [c <- Peer'<Op,Reply>  req <- :<fqdn>::<Op>Request]
     ;;     -> :<fqdn>::<Op>Response
     ;;     (let [_ (send' c (Op::<Op> req))  r (recv' c)]
     ;;       (match r -> <resp-ty> ((Reply::<Op> resp) resp))))
     ;; Method name uses namespace-aware pascal->kebab-in (Arc 265).
     ;;
     ;; Hygiene for method-body: `_` and `r` are let binders inside a nested quasiquote.
     ;; Fix: `discard-sym` = (symbol-node "_") and `r-sym` = (symbol-node "r") make them
     ;; Unquote nodes at definition time → checker skips them.
     ;; method-params `[c <- ~peer-ty req <- ~req-ty]` is a Vector → checker skips it.
     ;; `resp` in the match arm is a match-pattern binder (not let/fn) → fine as literal.
     methods       (:wat::core::foldl
                     (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                      clause <- :wat::WatAST]
                       -> :wat::core::Vector<wat::WatAST>
                       (:wat::core::let
                         [ch              (:wat::core::ast->children clause)
                          opkw            (:wat::core::Option/expect -> :wat::WatAST
                                            (:wat::core::first ch)
                                            "defservice methods: op-clause has no head")
                          out-fieldvec    (:wat::core::Option/expect -> :wat::WatAST
                                            (:wat::core::first (:wat::core::drop ch 3))
                                            "defservice methods: op-clause has no out-fieldvec")
                          op-str          (:wat::core::keyword/to-string opkw)
                          op-lower        (:wat::core::string::pascal->kebab-in fqdn-kw op-str)
                          method-name     (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::concat "/" op-lower)))
                          req-ty          (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::concat "::" op-str "Request")))
                          resp-ty         (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::concat "::" op-str "Response")))
                          op-variant-kw   (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::concat "::Op::" op-str)))
                          reply-variant-kw (:wat::core::keyword/from-string
                                             (:wat::core::string::concat fqdn-str
                                               (:wat::core::string::concat "::Reply::" op-str)))
                          ;; method params: [c <- Peer'<Op,Reply>  req <- <req-ty>]
                          ;; Client peer: connect'(Address'<Op,Reply>) → Peer'<Op,Reply>.
                          ;; Vector → checker skips Vector children.
                          method-params   `[c <- ~client-peer-ty req <- ~req-ty]
                          ;; `_` and `r` let binders in method-body's nested quasiquote:
                          ;; use symbol-node so they're Unquote nodes at definition time.
                          discard-sym     (:wat::core::symbol-node "_")
                          r-sym           (:wat::core::symbol-node "r")
                          method-body     `(:wat::core::let
                                             [~discard-sym (:wat::kernel::send' c (~op-variant-kw req))
                                              ~r-sym (:wat::kernel::recv' c)]
                                             (:wat::core::match ~r-sym -> ~resp-ty
                                               ((~reply-variant-kw resp) resp)
                                               (_ (:wat::kernel::assertion-failed!
                                                    "defservice method: misrouted reply variant (protocol violation)"
                                                    :wat::core::None
                                                    :wat::core::None))))]
                         (:wat::core::conj acc
                           `(:wat::core::defn ~method-name ~method-params -> ~resp-ty ~method-body))))
                     (:wat::core::Vector :wat::WatAST)
                     clauses)

     ;; ── host-parity-4a: host-agnostic start fn ────────────────────────────────────
     ;; (defn <fqdn>/start [host <- :wat::spawn::Host  state0 <- <state-ty>] -> <fqdn>::Handle
     ;;   (let [b    (listener' host Op Reply)                ; listener' accepts an abstract :Host
     ;;         l    (Bound/listener b)
     ;;         addr (Bound/address b)
     ;;         svc  (:wat::spawn::Host/launch host l (Vector Peer'<Reply,Op>) state0
     ;;                (keyword/from-string "<fqdn>::serve"))]  ; the protocol builds the per-tier prog
     ;;     (Handle svc addr)))
     ;;
     ;; C.3 baked `(spawn::thread)` + the serve CLOSURE into start. host-parity-4a makes
     ;; start host-blind: the thread-specific closure (capturing l + state0) moved INTO the
     ;; ThreadOpts `Host/launch` impl (wat/spawn.wat), and serve is passed by NAME (a runtime
     ;; keyword via keyword/from-string — a spliced literal `:fqdn::serve` would Arc-009-resolve
     ;; to a Fn, not a keyword) so the impl invokes it via apply. Process (4b) joins as one
     ;; extend-type, zero edit here.
     ;;
     ;; Hygiene for start-body:
     ;;   `lr` is the let binder for the Launched<S,R> value → symbol-node.
     ;;   `host`, `state0` are value references (start's params) → fine as literals.
     ;; start-params `[host <- :Host  state0 <- ~state-ty]` → Vector inner → checker skips it.
     ;;
     ;; arc 272 6b-ii-β: listener-minting moved INTO Host/launch (child-mints for process tier).
     ;; start calls Host/launch<Op,Reply> with EXPLICIT type-args (arc-232 dep) so the impl's
     ;; (listener' self :S :R) resolves S=Op, R=Reply. The call-head is built as a runtime keyword
     ;; via string::concat + keyword/from-string (no new primitives — no STOP trigger 1).
     ;; launch returns Launched<Op,Reply>{handle,address}; start unwraps into Handle.
     lr-sym        (:wat::core::symbol-node "lr")
     launch-head-kw (:wat::core::keyword/from-string
                      (:wat::core::string::concat "wat::spawn::Host/launch<"
                        (:wat::core::string::concat fqdn-str
                          (:wat::core::string::concat "::Op,"
                            (:wat::core::string::concat fqdn-str "::Reply>")))))
     start-params  `[host <- :wat::spawn::Host  state0 <- ~state-ty]
     start-body    `(:wat::core::let
                      [~lr-sym (~launch-head-kw host state0
                                 (:wat::core::keyword/from-string ~serve-name-str))]
                      (~handle-name (:wat::spawn::Launched/handle ~lr-sym)
                                    (:wat::spawn::Launched/address ~lr-sym)))
     start-fn      `(:wat::core::defn ~start-name ~start-params -> ~handle-name ~start-body)

     ;; ── C.3: Handle record ───────────────────────────────────────────────────────
     ;; (Record::def <fqdn>::Handle
     ;;   [handle <- :wat::spawn::Spawned
     ;;    addr   <- :wat::kernel::Address'<fqdn::Op,fqdn::Reply>])
     ;; handle is the host-agnostic spawn-handle marker: Thread'/Process'/future-remote
     ;; all derive :wat::spawn::Spawned so any concrete handle satisfies this field.
     ;; addr carries the typed Address'<Op,Reply> for client connect'.
     handle-fields `[handle <- :wat::spawn::Spawned addr <- ~addr-ty]
     handle-record `(:wat::Record::def ~handle-name ~handle-fields)]

    ;; Assemble the final `do`:
    ;;   request + response records (before enums — spliced as type-decls)
    ;;   Op + Reply enums (wrap the records)
    ;;   serve defn (the dispatch loop)
    ;;   constructors (per-op request constructors)
    ;;   methods (per-op type-safe methods)
    ;;   start fn (mints listener + spawns serve → Handle)
    ;;   Handle record (start's return type; emitted last)
    ;; Type-decl forms (records, enums, Handle) splice to top-level via splice_type_decl;
    ;; defns keep the `do` non-empty after type-decl stripping.
    `(:wat::core::do
       ~@request-records
       ~@response-records
       (:wat::core::defenum ~enum-name ~@variants)
       (:wat::core::defenum ~reply-name ~@reply-variants)
       (:wat::core::defn ~serve-name ~serve-params -> :wat::core::nil ~serve-body)
       ~@constructors
       ~@methods
       ~start-fn
       ~handle-record)))
