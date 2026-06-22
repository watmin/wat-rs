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
  :Reply [state <- :S  reply <- :R]
  :Stop  [state <- :S  reply <- :R])

(:wat::core::defmacro :wat::service::defservice
  [fqdn         <- :wat::WatAST     ;; :my::counter
   _state-kw    <- :wat::WatAST     ;; the literal :state marker (ignored)
   state-fields <- :wat::WatAST     ;; the field vector [count <- :wat::core::i64] (minted into State record)
   _ops-kw      <- :wat::WatAST     ;; the literal :ops marker (ignored)
   ops          <- :wat::WatAST     ;; the [ (:Get …) (:Increment …) ] vector NODE
   & opts       <- :wat::core::Vector<wat::WatAST>]  ;; optional trailing: [] or [:record-parent <parent>]
  -> :wat::WatAST
  ;; PROGRAM-BODY path: top-level `let`, params are node-values, nested quasiquote at the end.
  (:wat::core::let
    [fqdn-str      (:wat::core::keyword/to-string fqdn)
     ;; Arc 265 — reconstruct fqdn as a keyword value so pascal->kebab-in
     ;; can use it as the namespace for acronym-registry lookup.
     fqdn-kw       (:wat::core::keyword/from-string fqdn-str)

     ;; ── rs-1: fold trailing opts into a kwargs MAP (honest + extensible) ──────
     ;; opts is the rest param: a flat [:key val :key val …] list. We fold it into a
     ;; HashMap ONCE (size-based pairs, not "exactly one pair"), rejecting any key not in
     ;; `known-opts` DIRECTLY — named (raise the bar: the user's mistake is reported, not
     ;; silently mis-read). Then each option is a plain `(HashMap/get opts-map "<key>")`.
     ;; To add an option later: one entry in `known-opts` + one more `get` below. Keys are
     ;; strings (a WatAST keyword node isn't reliably `=` a runtime keyword — `to-string` them).
     known-opts     (:wat::core::HashMap/assoc
                      (:wat::core::HashMap :wat::core::String :wat::core::bool)
                      "record-parent" true)
     opts-len       (:wat::core::length opts)
     n-opt-pairs    (:wat::core::i64::/ opts-len 2)
     ;; even-length guard (no modulo: (len/2)*2 == len iff even)
     _opts-even     (:wat::core::if
                      (:wat::core::= (:wat::core::i64::* n-opt-pairs 2) opts-len)
                      -> :wat::core::nil
                      nil
                      (:wat::core::macro-error
                        "defservice: trailing options must be :keyword value pairs"))
     ;; build + validate in one pass: assoc each recognized key; macro-error on an unknown one
     opts-map       (:wat::core::foldl
                      (:wat::core::fn [m <- :wat::core::HashMap<wat::core::String,wat::WatAST>
                                       i <- :wat::core::i64]
                        -> :wat::core::HashMap<wat::core::String,wat::WatAST>
                        (:wat::core::let
                          [k   (:wat::core::i64::* i 2)
                           key (:wat::core::keyword/to-string
                                 (:wat::core::Option/expect  
                                   (:wat::core::get opts k) "defservice: malformed trailing option key"))]
                          (:wat::core::if (:wat::core::HashMap/contains-key? known-opts key)
                            -> :wat::core::HashMap<wat::core::String,wat::WatAST>
                            (:wat::core::HashMap/assoc m key
                              (:wat::core::Option/expect  
                                (:wat::core::get opts (:wat::core::i64::+ k 1))
                                "defservice: trailing option missing a value"))
                            (:wat::core::macro-error
                              (:wat::core::string::concat "defservice: unknown trailing option :"
                                (:wat::core::string::concat key
                                  " — recognized options: :record-parent"))))))
                      (:wat::core::HashMap :wat::core::String :wat::WatAST)
                      (:wat::core::range 0 n-opt-pairs))
     ;; each option is now a plain get with a default
     state-parent   (:wat::core::if (:wat::core::HashMap/contains-key? opts-map "record-parent")
                      -> :wat::WatAST
                      (:wat::core::Option/expect  
                        (:wat::core::HashMap/get opts-map "record-parent")
                        "defservice: :record-parent needs a value")
                      :wat::Record)

     ;; ── rs-1: mint state-ty as :<fqdn>::State ───────────────────────────────
     ;; REBIND state-ty so every downstream ~state-ty use (serve param, StopResponse,
     ;; stop method, start params, self-peer) keeps working unchanged.
     state-ty       (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}::State" :fqdn-str fqdn-str))

     ;; ── rs-1: emit the State record def, branching on state-parent ──────────
     ;; :wat::holon::Record → (:wat::holon::Record::def ~state-ty ~state-fields)
     ;; else              → (:wat::Record::def          ~state-ty ~state-fields)
     ;; Compare via keyword/to-string since state-parent is a WatAST keyword node.
     state-parent-str (:wat::core::keyword/to-string state-parent)
     state-record   (:wat::core::if (:wat::core::= state-parent-str "wat::holon::Record")
                      -> :wat::WatAST
                      `(:wat::holon::Record::def ~state-ty ~state-fields)
                      `(:wat::Record::def ~state-ty ~state-fields))

     enum-name     (:wat::core::keyword/from-string
                     (:wat::core::string::interpolate "{fqdn-str}::Op" :fqdn-str fqdn-str))
     reply-name    (:wat::core::keyword/from-string
                     (:wat::core::string::interpolate "{fqdn-str}::Reply" :fqdn-str fqdn-str))
     serve-name    (:wat::core::keyword/from-string
                     (:wat::core::string::interpolate "{fqdn-str}::serve" :fqdn-str fqdn-str))
     ;; Arc 209 host-parity-4a — the serve fqdn as a STRING, spliced into start's
     ;; `(keyword/from-string …)` so Locus/launch receives serve by a RUNTIME keyword
     ;; (a spliced literal `:fqdn::serve` would Arc-009-resolve to a Fn, not a keyword).
     serve-name-str (:wat::core::string::interpolate "{fqdn-str}::serve" :fqdn-str fqdn-str)
     start-name    (:wat::core::keyword/from-string
                     (:wat::core::string::interpolate "{fqdn-str}/start" :fqdn-str fqdn-str))
     handle-name   (:wat::core::keyword/from-string
                     (:wat::core::string::interpolate "{fqdn-str}::Handle" :fqdn-str fqdn-str))
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
                            opkw        (:wat::core::first ch)
                            argvec      (:wat::core::first (:wat::core::drop ch 1))
                            ;; in-fields = drop the leading s <- :State triple (3 nodes)
                            in-fieldch  (:wat::core::drop (:wat::core::ast->children argvec) 3)
                            op-str      (:wat::core::keyword/to-string opkw)
                            req-name    (:wat::core::keyword/from-string
                                          (:wat::core::string::concat fqdn-str
                                            (:wat::core::string::interpolate "::{op-str}Request" :op-str op-str)))
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
                             opkw         (:wat::core::first ch)
                             out-fieldvec (:wat::core::first (:wat::core::drop ch 3))
                             out-fieldch  (:wat::core::ast->children out-fieldvec)
                             op-str       (:wat::core::keyword/to-string opkw)
                             resp-name    (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::interpolate "::{op-str}Response" :op-str op-str)))
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
                          opkw    (:wat::core::first ch)
                          argvec  (:wat::core::first (:wat::core::drop ch 1))
                          op-str  (:wat::core::keyword/to-string opkw)
                          req-ty  (:wat::core::keyword/from-string
                                    (:wat::core::string::concat fqdn-str
                                      (:wat::core::string::interpolate "::{op-str}Request" :op-str op-str)))
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
                           opkw    (:wat::core::first ch)
                           out-fieldvec (:wat::core::first (:wat::core::drop ch 3))
                           op-str  (:wat::core::keyword/to-string opkw)
                           resp-ty (:wat::core::keyword/from-string
                                     (:wat::core::string::concat fqdn-str
                                       (:wat::core::string::interpolate "::{op-str}Response" :op-str op-str)))
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
                          opkw          (:wat::core::first ch)
                          argvec        (:wat::core::first (:wat::core::drop ch 1))
                          body          (:wat::core::first (:wat::core::drop ch 4))
                          ;; in-fields MINUS leading s <- :State triple (3 nodes)
                          fieldch       (:wat::core::drop (:wat::core::ast->children argvec) 3)
                          op-str        (:wat::core::keyword/to-string opkw)
                          op-variant-kw (:wat::core::keyword/from-string
                                          (:wat::core::string::concat fqdn-str
                                            (:wat::core::string::interpolate "::Op::{op-str}" :op-str op-str)))
                          reply-variant-kw (:wat::core::keyword/from-string
                                             (:wat::core::string::concat fqdn-str
                                               (:wat::core::string::interpolate "::Reply::{op-str}" :op-str op-str)))
                          ;; Extract state binder (e.g. `s`) from the first triple of argvec.
                          ;; Unquoted → Unquote node at definition time → passes hygiene check.
                          state-binder  (:wat::core::first (:wat::core::ast->children argvec))
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
                                            (:wat::core::Option/expect  
                                              (:wat::core::get fieldch i)
                                              "defservice serve-arm: arg name out of bounds"))
                                          arg-indices)
                          ;; Accessor keyword nodes: <fqdn>::<Op>Request/<field-name>
                          ;; Field names are Symbol nodes (e.g. `n`), not Keywords.
                          ;; Use ast-name (handles both Symbol and Keyword) to get the text.
                          arg-accessors (:wat::core::map
                                          (:wat::core::fn [i <- :wat::core::i64] -> :wat::WatAST
                                            (:wat::core::let
                                              [name-node (:wat::core::Option/expect  
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
                                              [arg-name (:wat::core::Option/expect  
                                                           (:wat::core::get arg-names i)
                                                           "defservice serve-arm: arg-name index")
                                               acc-kw   (:wat::core::Option/expect  
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
                                                (~serve-name self l clients new-state)))
                                            ((:wat::service::Outcome::Stop final-state resp)
                                              (:wat::core::do
                                                (:wat::kernel::send'
                                                  (:wat::core::nth clients idx)
                                                  (~reply-variant-kw resp))
                                                nil)))]
                         (:wat::core::conj acc
                           `((~op-variant-kw req) ~outcome-match))))
                     (:wat::core::Vector :wat::WatAST)
                     clauses)

     ;; ── rs-2: AUTO stop op (standalone — not threaded through user-op folds) ───────
     ;; The stop op has a different shape at every fold: nullary request, state-carrying
     ;; response, auto serve body, state-typed client method. Built standalone and conj'd
     ;; into each collection before final assembly.
     ;;
     ;; StopRequest [] — nullary; client sends it to terminate the service.
     stop-req-name   (:wat::core::keyword/from-string
                       (:wat::core::string::interpolate "{fqdn-str}::StopRequest" :fqdn-str fqdn-str))
     stop-req-record `(:wat::Record::def ~stop-req-name [])
     ;; StopResponse [state <- <state-ty>] — carries the final state to the client.
     stop-resp-name  (:wat::core::keyword/from-string
                       (:wat::core::string::interpolate "{fqdn-str}::StopResponse" :fqdn-str fqdn-str))
     stop-resp-fields `[state <- ~state-ty]
     stop-resp-record `(:wat::Record::def ~stop-resp-name ~stop-resp-fields)
     ;; Op::Stop variant [req <- StopRequest]
     stop-op-variant-kw (:wat::core::keyword/from-string
                          (:wat::core::string::interpolate "{fqdn-str}::Op::Stop" :fqdn-str fqdn-str))
     stop-op-req-field `[req <- ~stop-req-name]
     ;; Reply::Stop variant [resp <- StopResponse]
     stop-reply-variant-kw (:wat::core::keyword/from-string
                             (:wat::core::string::interpolate "{fqdn-str}::Reply::Stop" :fqdn-str fqdn-str))
     stop-reply-resp-field `[resp <- ~stop-resp-name]
     ;; Auto serve arm for Op::Stop:
     ;;   ((Op::Stop req)
     ;;     (match (Outcome::Stop state (StopResponse state))
     ;;       ((Outcome::Stop final-state resp)
     ;;         (do (send' (nth clients idx) (Reply::Stop resp)) nil))))
     ;; `state` is the serve param (value position in match). The outer outcome-match
     ;; structure is duplicated here (no user body to bind; direct auto handler).
     stop-resp-acc (:wat::core::keyword/from-string
                     (:wat::core::string::interpolate "{fqdn-str}::StopResponse/state" :fqdn-str fqdn-str))
     stop-serve-arm `((~stop-op-variant-kw req)
                       (:wat::core::match
                         (:wat::service::Outcome::Stop state (~stop-resp-name state))
                         -> :wat::core::nil
                         ((:wat::service::Outcome::Reply _ _) nil)
                         ((:wat::service::Outcome::Stop final-state resp)
                           (:wat::core::do
                             (:wat::kernel::send'
                               (:wat::core::nth clients idx)
                               (~stop-reply-variant-kw resp))
                             nil))))
     ;; Extend the record/enum/serve-arm collections now (before serve-body + service-forms-def
     ;; use them). constructors/methods are extended after their user-op folds complete below.
     request-records  (:wat::core::conj request-records stop-req-record)
     response-records (:wat::core::conj response-records stop-resp-record)
     variants         (:wat::core::conj (:wat::core::conj variants :Stop) stop-op-req-field)
     reply-variants   (:wat::core::conj (:wat::core::conj reply-variants :Stop) stop-reply-resp-field)
     serve-op-arms    (:wat::core::conj serve-op-arms stop-serve-arm)

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
                          opkw         (:wat::core::first ch)
                          argvec       (:wat::core::first (:wat::core::drop ch 1))
                          ;; in-fields minus the leading s <- :State triple
                          in-fieldch   (:wat::core::drop (:wat::core::ast->children argvec) 3)
                          op-str       (:wat::core::keyword/to-string opkw)
                          op-lower     (:wat::core::string::pascal->kebab-in fqdn-kw op-str)
                          ctor-name    (:wat::core::keyword/from-string
                                         (:wat::core::string::concat fqdn-str
                                           (:wat::core::string::interpolate "/{op-lower}-request" :op-lower op-lower)))
                          req-ty       (:wat::core::keyword/from-string
                                         (:wat::core::string::concat fqdn-str
                                           (:wat::core::string::interpolate "::{op-str}Request" :op-str op-str)))
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
                                           (:wat::core::Option/expect  
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
                          opkw            (:wat::core::first ch)
                          out-fieldvec    (:wat::core::first (:wat::core::drop ch 3))
                          op-str          (:wat::core::keyword/to-string opkw)
                          op-lower        (:wat::core::string::pascal->kebab-in fqdn-kw op-str)
                          method-name     (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::interpolate "/{op-lower}" :op-lower op-lower)))
                          req-ty          (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::interpolate "::{op-str}Request" :op-str op-str)))
                          resp-ty         (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::interpolate "::{op-str}Response" :op-str op-str)))
                          op-variant-kw   (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::interpolate "::Op::{op-str}" :op-str op-str)))
                          reply-variant-kw (:wat::core::keyword/from-string
                                             (:wat::core::string::concat fqdn-str
                                               (:wat::core::string::interpolate "::Reply::{op-str}" :op-str op-str)))
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

     ;; ── rs-2: stop constructor + method (extended after user-op folds complete) ───
     ;; Constructor: (defn <fqdn>/stop-request [] -> StopRequest (StopRequest))
     stop-ctor-name  (:wat::core::keyword/from-string
                       (:wat::core::string::interpolate "{fqdn-str}/stop-request" :fqdn-str fqdn-str))
     stop-ctor       `(:wat::core::defn ~stop-ctor-name [] -> ~stop-req-name (~stop-req-name))
     ;; Method: (defn <fqdn>/stop [c <- client-peer-ty] -> state-ty ...)
     ;; Sends Op::Stop(StopRequest) over c, recv's Reply, matches Reply::Stop → extracts state.
     ;; Uses symbol-node for `_` and `r` let binders (hygiene: Unquote at def time).
     stop-discard-sym (:wat::core::symbol-node "_")
     stop-r-sym       (:wat::core::symbol-node "r")
     stop-method-name (:wat::core::keyword/from-string
                        (:wat::core::string::interpolate "{fqdn-str}/stop" :fqdn-str fqdn-str))
     stop-method-params `[c <- ~client-peer-ty]
     stop-method-body `(:wat::core::let
                          [~stop-discard-sym (:wat::kernel::send' c (~stop-op-variant-kw (~stop-req-name)))
                           ~stop-r-sym       (:wat::kernel::recv' c)]
                          (:wat::core::match ~stop-r-sym -> ~state-ty
                            ((~stop-reply-variant-kw resp) (~stop-resp-acc resp))
                            (_ (:wat::kernel::assertion-failed!
                                 "defservice stop method: unexpected reply variant (protocol violation)"
                                 :wat::core::None
                                 :wat::core::None))))
     stop-method      `(:wat::core::defn ~stop-method-name ~stop-method-params -> ~state-ty ~stop-method-body)
     ;; Extend constructors and methods with the auto stop op.
     constructors     (:wat::core::conj constructors stop-ctor)
     methods          (:wat::core::conj methods stop-method)

     ;; ── host-parity-4a: locus-agnostic start fn ──────────────────────────────────
     ;; (defn <fqdn>/start [locus <- :wat::spawn::Locus  state0 <- <state-ty>] -> <fqdn>::Handle
     ;;   (let [b    (listener' locus Op Reply)              ; listener' accepts an abstract :Locus
     ;;         l    (Bound/listener b)
     ;;         addr (Bound/address b)
     ;;         svc  (:wat::spawn::Locus/launch locus l (Vector Peer'<Reply,Op>) state0
     ;;                (keyword/from-string "<fqdn>::serve"))]  ; the protocol builds the per-tier prog
     ;;     (Handle svc addr)))
     ;;
     ;; C.3 baked `(spawn::thread)` + the serve CLOSURE into start. host-parity-4a makes
     ;; start locus-blind: the thread-specific closure (capturing l + state0) moved INTO the
     ;; ThreadOpts `Locus/launch` impl (wat/spawn.wat), and serve is passed by NAME (a runtime
     ;; keyword via keyword/from-string — a spliced literal `:fqdn::serve` would Arc-009-resolve
     ;; to a Fn, not a keyword) so the impl invokes it via apply. Process (4b) joins as one
     ;; extend-type, zero edit here.
     ;;
     ;; Hygiene for start-body:
     ;;   `lr` is the let binder for the Launched<S,R> value → symbol-node.
     ;;   `locus`, `state0` are value references (start's params) → fine as literals.
     ;; start-params `[locus <- :Locus  state0 <- ~state-ty]` → Vector inner → checker skips it.
     ;;
     ;; arc 272 6b-ii-β: listener-minting moved INTO Locus/launch (child-mints for process tier).
     ;; start calls Locus/launch<Op,Reply> with EXPLICIT type-args (arc-232 dep) so the impl's
     ;; (listener' self :S :R) resolves S=Op, R=Reply. The call-head is built as a runtime keyword
     ;; via string::concat + keyword/from-string (no new primitives — no STOP trigger 1).
     ;; launch returns Launched<Op,Reply>{handle,address}; start unwraps into Handle.
     lr-sym        (:wat::core::symbol-node "lr")
     launch-head-kw (:wat::core::keyword/from-string
                      (:wat::core::string::concat "wat::spawn::Locus/launch<"
                        (:wat::core::string::concat fqdn-str
                          (:wat::core::string::concat "::Op,"
                            (:wat::core::string::concat fqdn-str "::Reply>")))))

     ;; ── arc 272 6b-ii-β: transport-agnostic service-forms ────────────────────────
     ;; service-forms-kw must be defined before start-body (which splices ~service-forms-kw).
     ;; service-forms-kw: the keyword :<fqdn>::service-forms — the name of the emitted def.
     service-forms-kw (:wat::core::keyword/from-string
                        (:wat::core::string::interpolate "{fqdn-str}::service-forms" :fqdn-str fqdn-str))
     ;; The agnostic child :user::main: binds on :wat::spawn::service-locus (a FREE
     ;; name — defservice does NOT define it). The ProcessOpts launch arm prepends
     ;; `(def :wat::spawn::service-locus (process))` before spawning, so the child
     ;; universe resolves service-locus at startup to a ProcessOpts value.
     ;; self-peer S=addr-ty (child sends minted Address' up), R=state-ty (parent sends
     ;; state0 down). serve is invoked via apply (dynamic keyword) — the child main
     ;; never statically names the per-service serve fn.
     ;; Hygiene: child main let binders (b/cm-self/_/st) are synthetic names → must use
     ;; symbol-node + unquote so they appear as Unquote nodes in the template, not bare
     ;; Symbols that would trigger the ProgramBodyIntroducesName hygiene gate.
     cm-b-sym    (:wat::core::symbol-node "b")
     cm-self-sym (:wat::core::symbol-node "self")
     cm-und-sym  (:wat::core::symbol-node "_")
     cm-st-sym   (:wat::core::symbol-node "st")
     child-main-form `(:wat::core::defn :user::main [] -> :wat::core::nil
                        (:wat::core::let
                          [~cm-b-sym    (:wat::kernel::listener' :wat::spawn::service-locus
                                            ~enum-name ~reply-name)
                           ~cm-self-sym (:wat::program::self-peer ~addr-ty ~state-ty)
                           ~cm-und-sym  (:wat::kernel::send' ~cm-self-sym
                                            (:wat::spawn::Bound/address ~cm-b-sym))
                           ~cm-st-sym   (:wat::kernel::recv' ~cm-self-sym)]
                          (:wat::core::apply -> :wat::core::nil
                            (:wat::core::keyword/from-string ~serve-name-str) ~cm-self-sym
                            (:wat::spawn::Bound/listener ~cm-b-sym)
                            (:wat::core::Vector ~peer-ty)
                            ~cm-st-sym [])))
     ;; The transport-agnostic service-forms defn: Op/Reply/records/serve + agnostic child
     ;; main. Emitted as `(defn :<fqdn>::service-forms [] -> Vector<WatAST> (forms …))`.
     ;; A 0-arg fn so the checker can type-check call sites: `(:my::counter::service-forms)`
     ;; returns Vector<WatAST>. Registered into sym.functions at step 6 via
     ;; preregister_fn_defs_in_do, so the checker sees it before checking start-fn.
     ;; The ProcessOpts launch arm receives the Vector value (the runtime evaluates the
     ;; call before dispatch, so it arrives as the actual Vec).
     service-forms-def `(:wat::core::defn ~service-forms-kw
                          [] -> :wat::core::Vector<wat::WatAST>
                          (:wat::core::forms
                            ~state-record
                            ~@request-records
                            ~@response-records
                            (:wat::core::defenum ~enum-name ~@variants)
                            (:wat::core::defenum ~reply-name ~@reply-variants)
                            (:wat::core::defn ~serve-name ~serve-params
                              -> :wat::core::nil ~serve-body)
                            ~child-main-form))

     start-params  `[locus <- :wat::spawn::Locus  state0 <- ~state-ty]
     start-body    `(:wat::core::let
                      [~lr-sym (~launch-head-kw locus state0
                                 (:wat::core::keyword/from-string ~serve-name-str)
                                 (~service-forms-kw))]
                      (~handle-name (:wat::spawn::Launched/handle ~lr-sym)
                                    (:wat::spawn::Launched/address ~lr-sym)))
     start-fn      `(:wat::core::defn ~start-name ~start-params -> ~handle-name ~start-body)

     ;; ── C.3: Handle record ───────────────────────────────────────────────────────
     ;; (Record::def <fqdn>::Handle
     ;;   [handle <- :wat::spawn::Spawned
     ;;    addr   <- :wat::kernel::Address'<fqdn::Op,fqdn::Reply>])
     ;; handle is the locus-agnostic spawn-handle marker: Thread'/Process'/future-remote
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
    ;;   service-forms def (transport-agnostic fragment; emitted last so all
    ;;     referenced names are already declared in the top-level scope)
    ;; Type-decl forms (records, enums, Handle) splice to top-level via splice_type_decl;
    ;; defns keep the `do` non-empty after type-decl stripping.
    `(:wat::core::do
       ~state-record
       ~@request-records
       ~@response-records
       (:wat::core::defenum ~enum-name ~@variants)
       (:wat::core::defenum ~reply-name ~@reply-variants)
       (:wat::core::defn ~serve-name ~serve-params -> :wat::core::nil ~serve-body)
       ~@constructors
       ~@methods
       ~service-forms-def
       ~start-fn
       ~handle-record)))
