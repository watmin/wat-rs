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

;; ── The canonical clause order — the story a service tells ──────────────────────
;; A defservice reads top-to-bottom as a sentence about an actor. Order is
;; compiler-free (all-kwargs); this is house style. Foundation precedes, elaboration
;; follows — a parent founds the durable record (leads it); :calls elaborates the
;; ephemeral peers (trails them).
;;
;;   :durable-parent   what I'm built from   (optional — the durable record's parent; e.g. holon)
;;   :durable          what I remember       (the soul: EDN, crosses the wire, survives hibernation)
;;   :ephemeral        what I carry          (the body: resources + peer clients; never crosses)
;;   :calls            who I call            (install each callee's client contract — arc 291 4b-iv)
;;   :init             how I'm built         ((Record, …operating-inputs) -> State)
;;   :hibernate        how I rest            (State -> Record; optional, defaults)
;;   :stop             how I end             (State -> Resp; optional, defaults)
;;   :ops              what I do             (the typed message API)
(:wat::core::defmacro :wat::service::defservice
  [fqdn    <- :wat::WatAST     ;; :my::counter
   & clauses <- :wat::core::Vector<wat::WatAST>]  ;; all-kwargs: [:durable [..] :ephemeral [..] :ops [..] ...]
  -> :wat::WatAST
  ;; PROGRAM-BODY path: top-level `let`, params are node-values, nested quasiquote at the end.
  (:wat::core::let
    [fqdn-str      (:wat::core::keyword/to-string fqdn)
     ;; Arc 265 — reconstruct fqdn as a keyword value so pascal->kebab-in
     ;; can use it as the namespace for acronym-registry lookup.
     fqdn-kw       (:wat::core::keyword/from-string fqdn-str)

     ;; ── 4b-ii: fold ALL clauses into a kwargs MAP (all-kwargs surface) ──────
     ;; clauses is the rest param: a flat [:key val :key val …] list. We fold it into a
     ;; HashMap ONCE, rejecting any key not in `known-clauses` DIRECTLY — named.
     ;; known-clauses: durable, ephemeral, ops (REQUIRED), init, hibernate, stop, durable-parent.
     known-clauses  (:wat::core::HashMap/assoc
                      (:wat::core::HashMap/assoc
                        (:wat::core::HashMap/assoc
                          (:wat::core::HashMap/assoc
                            (:wat::core::HashMap/assoc
                              (:wat::core::HashMap/assoc
                                (:wat::core::HashMap/assoc
                                  (:wat::core::HashMap/assoc
                                    (:wat::core::HashMap :wat::core::String :wat::core::bool)
                                    "durable" true)
                                  "ephemeral" true)
                                "ops" true)
                              "init" true)
                            "hibernate" true)
                          "stop" true)
                        "durable-parent" true)
                      "calls" true)
     clauses-len    (:wat::core::length clauses)
     n-clause-pairs (:wat::core::i64::/ clauses-len 2)
     ;; even-length guard
     _clauses-even  (:wat::core::if
                      (:wat::core::= (:wat::core::i64::* n-clause-pairs 2) clauses-len)
                      -> :wat::core::nil
                      nil
                      (:wat::core::macro-error
                        "defservice: clauses must be :keyword value pairs"))
     ;; build + validate in one pass
     clause-map     (:wat::core::foldl
                      (:wat::core::fn [m <- :wat::core::HashMap<wat::core::String,wat::WatAST>
                                       i <- :wat::core::i64]
                        -> :wat::core::HashMap<wat::core::String,wat::WatAST>
                        (:wat::core::let
                          [k   (:wat::core::i64::* i 2)
                           key (:wat::core::keyword/to-string
                                 (:wat::core::Option/expect
                                   (:wat::core::get clauses k) "defservice: malformed clause key"))]
                          (:wat::core::if (:wat::core::HashMap/contains-key? known-clauses key)
                            -> :wat::core::HashMap<wat::core::String,wat::WatAST>
                            (:wat::core::HashMap/assoc m key
                              (:wat::core::Option/expect
                                (:wat::core::get clauses (:wat::core::i64::+ k 1))
                                "defservice: clause missing a value"))
                            (:wat::core::macro-error
                              (:wat::core::string::concat "defservice: unknown clause :"
                                (:wat::core::string::concat key
                                  " — recognized clauses: :durable :ephemeral :ops :init :hibernate :stop :durable-parent :calls"))))))
                      (:wat::core::HashMap :wat::core::String :wat::WatAST)
                      (:wat::core::range 0 n-clause-pairs))
     ;; :ops is REQUIRED
     _ops-required  (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "ops")
                      -> :wat::core::nil
                      nil
                      (:wat::core::macro-error "defservice: :ops clause is required"))
     ops            (:wat::core::Option/expect
                      (:wat::core::HashMap/get clause-map "ops")
                      "defservice: :ops clause missing value")

     ;; :durable [fields] — optional, default empty vector node []
     ;; The empty vector node is built by using with-children on a fresh Vector.
     ;; We need a Vector WatAST node; use the ops node as a shape carrier with empty children.
     empty-vec      (:wat::core::with-children ops (:wat::core::Vector :wat::WatAST))
     durable-fields (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "durable")
                      -> :wat::WatAST
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "durable")
                        "defservice: :durable needs a value")
                      empty-vec)

     ;; :ephemeral [fields] — optional, default empty vector node []
     ephemeral-fields (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "ephemeral")
                        -> :wat::WatAST
                        (:wat::core::Option/expect
                          (:wat::core::HashMap/get clause-map "ephemeral")
                          "defservice: :ephemeral needs a value")
                        empty-vec)
     ;; Is ephemeral non-empty? (child count > 0)
     ephemeral-len  (:wat::core::length (:wat::core::ast->children ephemeral-fields))
     has-ephemeral  (:wat::core::i64::> ephemeral-len 0)

     ;; :calls [svcs] — optional list of callee service keywords; their client-forms are
     ;; prepended to service-forms so the child process loads callee contracts before its own.
     calls-svcs     (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "calls")
                      -> :wat::WatAST
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "calls")
                        "defservice: :calls needs a value")
                      empty-vec)

     ;; :durable-parent — optional, default :wat::Record
     state-parent   (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "durable-parent")
                      -> :wat::WatAST
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "durable-parent")
                        "defservice: :durable-parent needs a value")
                      :wat::Record)

     ;; ── 4b-ii: mint state-ty as :<fqdn>::State, record-ty as :<fqdn>::Record ──
     state-ty       (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}::State" :fqdn-str fqdn-str))
     record-ty      (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}::Record" :fqdn-str fqdn-str))

     ;; ── 4b-ii: :init option ────────────────────────────────────────────────────
     ;; :init : Record → State. Default (fn [d <- ::Record] -> ::State (::State/new d))
     ;;   when :ephemeral is empty. When :ephemeral non-empty and :init absent → macro-error.
     ;; A synthetic symbol-node "record" for the default init param (hygiene: Unquote at def time).
     ;; arc 291 kwargs-start: renamed "d"→"record" so the default-init start kwarg is :record.
     d-sym          (:wat::core::symbol-node "record")
     s-sym          (:wat::core::symbol-node "s")
     ;; state-new-kw: :<fqdn>::State/new — the struct ctor
     state-new-kw   (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}::State/new" :fqdn-str fqdn-str))
     ;; init-fn-node: user-provided fn, or default, or macro-error
     init-fn-node   (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "init")
                      -> :wat::WatAST
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "init")
                        "defservice: :init needs a value")
                      (:wat::core::if has-ephemeral
                        -> :wat::WatAST
                        (:wat::core::macro-error
                          (:wat::core::string::concat fqdn-str
                            ": :ephemeral declares fields but no :init — the macro cannot construct ephemeral fields; provide :init : Record → State"))
                        `(:wat::core::fn [~d-sym <- ~record-ty] -> ~state-ty (~state-new-kw ~d-sym))))
     ;; Extract the param vector children [name <- :T] from the init fn node
     ;; init-fn-node structure: (fn [params] -> :RetTy body) → ast->children = [fn,params,->,:RetTy,body]
     init-fn-ch     (:wat::core::ast->children init-fn-node)
     init-params-vec (:wat::core::first (:wat::core::drop init-fn-ch 1))
     init-body      (:wat::core::first (:wat::core::drop init-fn-ch 4))
     ;; init-param: the children of the params vector — the 3-token binder [name <- :T]
     init-param     (:wat::core::ast->children init-params-vec)
     ;; init-arg-names: the list of param NAME nodes (tokens at indices 0, 3, 6, …)
     ;; init-param has 3 tokens per binder: [name <- :T]; extract the name at each i*3.
     init-arg-names (:wat::core::map
                      (:wat::core::fn [i <- :wat::core::i64] -> :wat::WatAST
                        (:wat::core::Option/expect
                          (:wat::core::get init-param (:wat::core::i64::* i 3))
                          "defservice: init param name out of bounds"))
                      (:wat::core::range 0 (:wat::core::i64::/ (:wat::core::length init-param) 3)))
     ;; init-name: :<fqdn>::init — the emitted defn's name keyword
     init-name-str  (:wat::core::string::interpolate "{fqdn-str}::init" :fqdn-str fqdn-str)
     init-name      (:wat::core::keyword/from-string init-name-str)
     ;; init-def: the emitted top-level defn for init
     init-def       `(:wat::core::defn ~init-name ~init-params-vec -> ~state-ty ~init-body)

     ;; ── 4b-ii: :stop option — projection hook ────────────────────────────────
     ;; Default: (fn [s <- ::State] -> ::Record (::State/durable s))
     ;; User-provided :stop keeps its own declared resp-ty (any EDN-portable type).
     state-durable-kw (:wat::core::keyword/from-string
                        (:wat::core::string::interpolate "{fqdn-str}::State/durable" :fqdn-str fqdn-str))
     stop-fn-node   (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "stop")
                      -> :wat::WatAST
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "stop")
                        "defservice: :stop needs a value")
                      `(:wat::core::fn [~s-sym <- ~state-ty] -> ~record-ty (~state-durable-kw ~s-sym)))
     stop-fn-ch     (:wat::core::ast->children stop-fn-node)
     stop-params-vec (:wat::core::first (:wat::core::drop stop-fn-ch 1))
     ;; resp-ty: index 3 = the :RetTy node in [fn, params, ->, :RetTy, body]
     resp-ty        (:wat::core::first (:wat::core::drop stop-fn-ch 3))
     stop-body      (:wat::core::first (:wat::core::drop stop-fn-ch 4))
     ;; stop-project-name: :<fqdn>::stop-project (distinct from <fqdn>/stop method)
     stop-project-name-str (:wat::core::string::interpolate "{fqdn-str}::stop-project" :fqdn-str fqdn-str)
     stop-project-name (:wat::core::keyword/from-string stop-project-name-str)
     ;; stop-project-def: the emitted top-level defn for stop projection
     stop-project-def `(:wat::core::defn ~stop-project-name ~stop-params-vec -> ~resp-ty ~stop-body)

     ;; ── 4b-ii: :hibernate option — projection hook (NEW, mirror of :stop) ────
     ;; Return type FORCED to ::Record (resume = :init consumes it).
     ;; Default: (fn [s <- ::State] -> ::Record (::State/durable s))
     ;; User-provided :hibernate: if it declares a different return type → macro-error.
     hibernate-fn-node (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "hibernate")
                         -> :wat::WatAST
                         (:wat::core::Option/expect
                           (:wat::core::HashMap/get clause-map "hibernate")
                           "defservice: :hibernate needs a value")
                         `(:wat::core::fn [~s-sym <- ~state-ty] -> ~record-ty (~state-durable-kw ~s-sym)))
     hibernate-fn-ch  (:wat::core::ast->children hibernate-fn-node)
     hibernate-params-vec (:wat::core::first (:wat::core::drop hibernate-fn-ch 1))
     ;; hib-ret-ty: the declared return type of the hibernate fn
     hib-ret-ty       (:wat::core::first (:wat::core::drop hibernate-fn-ch 3))
     hibernate-body   (:wat::core::first (:wat::core::drop hibernate-fn-ch 4))
     ;; Force the return type to ::Record — if user declared something else, macro-error
     hib-ret-str      (:wat::core::keyword/to-string hib-ret-ty)
     record-ty-str    (:wat::core::keyword/to-string record-ty)
     _hib-ty-check    (:wat::core::if (:wat::core::= hib-ret-str record-ty-str)
                        -> :wat::core::nil
                        nil
                        (:wat::core::macro-error
                          (:wat::core::string::concat fqdn-str
                            ": :hibernate return type must be ::Record (the resume seed); declared a different type")))
     hibernate-project-name-str (:wat::core::string::interpolate "{fqdn-str}::hibernate-project" :fqdn-str fqdn-str)
     hibernate-project-name (:wat::core::keyword/from-string hibernate-project-name-str)
     hibernate-project-def `(:wat::core::defn ~hibernate-project-name ~hibernate-params-vec -> ~record-ty ~hibernate-body)

     ;; ── 4b-ii: emit the Record def + State defstruct ─────────────────────────
     ;; record-def: (:wat::Record::def ::Record [durable-fields]) (or holon parent)
     ;; state-def:  (:wat::core::defstruct ::State [durable <- ::Record <ephemeral-fields...>])
     ;;   The 3 tokens `durable <- ~record-ty` are prepended to ephemeral children.
     state-parent-str (:wat::core::keyword/to-string state-parent)
     record-def   (:wat::core::if (:wat::core::= state-parent-str "wat::holon::Record")
                    -> :wat::WatAST
                    `(:wat::holon::Record::def ~record-ty ~durable-fields)
                    `(:wat::Record::def ~record-ty ~durable-fields))
     ;; Build the State struct field vector: prepend [durable <- ::Record] before ephemeral fields.
     ;; Strategy: use quasiquote to build the durable-field prefix vector `[durable <- ~record-ty]`,
     ;; extract its 3 children, then prepend them to the ephemeral children via foldl.
     ;; The quasiquote gives us WatAST nodes (incl. the `<-` keyword) rather than runtime values.
     durable-prefix-vec `[durable <- ~record-ty]
     durable-prefix-children (:wat::core::ast->children durable-prefix-vec)
     ephemeral-children (:wat::core::ast->children ephemeral-fields)
     ;; Concatenate: durable-prefix-children ++ ephemeral-children
     state-field-items (:wat::core::foldl
                         (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                          item <- :wat::WatAST]
                           -> :wat::core::Vector<wat::WatAST>
                           (:wat::core::conj acc item))
                         (:wat::core::foldl
                           (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                            item <- :wat::WatAST]
                             -> :wat::core::Vector<wat::WatAST>
                             (:wat::core::conj acc item))
                           (:wat::core::Vector :wat::WatAST)
                           durable-prefix-children)
                         ephemeral-children)
     ;; Build the state field vector as a WatAST::Vector using with-children on empty-vec
     state-field-vec (:wat::core::with-children empty-vec state-field-items)
     state-def    `(:wat::core::defstruct ~state-ty ~state-field-vec)

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

     ;; ── arc 291 3a-ii-α: lineage protocol types ──────────────────────────────
     ;; Admin enum:     :<fqdn>::Admin  — what the owner sends DOWN the lineage peer.
     ;;   :Init [seed <- :ship-ty]  — startup init-args (replaces raw ship)
     ;;   :Stop                     — owner-initiated stop (3a-ii-β dispatches this)
     ;; Status enum: :<fqdn>::Status — what the service sends UP the lineage peer.
     ;;   :Started [addr <- :addr-ty]    — startup address handoff (replaces raw addr)
     ;;   :Stopped   [state <- :state-ty]  — stop response (3a-ii-β uses this)
     ;;
     ;; self-peer type in child-main-form: Peer'<Status, Admin>
     ;;   child sends Status up, receives Admin down.
     ;;
     ;; dispatch-admin: fn [ai <- Admin] -> State
     ;;   wraps the startup handshake: matches Admin::Init, applies <fqdn>::init.
     ;;   Passed to Locus/launch by-name in place of the raw init keyword.
     ;;
     ;; extract-addr: fn [lu <- Status] -> addr-ty
     ;;   matches Status::Started, returns the Address'. Passed to launch as
     ;;   lu-addr-kw so the generic ProcessOpts impl can extract addr without
     ;;   naming per-service types.
     admin-ty-str   (:wat::core::string::interpolate "{fqdn-str}::Admin" :fqdn-str fqdn-str)
     admin-ty       (:wat::core::keyword/from-string admin-ty-str)
     status-ty-str (:wat::core::string::interpolate "{fqdn-str}::Status" :fqdn-str fqdn-str)
     status-ty  (:wat::core::keyword/from-string status-ty-str)
     ;; arc 291 3a-ii-β: the CHILD's lineage self-peer — sends Status UP, recvs Admin DOWN.
     ;; serve binds `self` to this (distinct from the client peer-ty Peer'<Reply,Op>).
     lineage-peer-ty (:wat::core::keyword/from-string
                       (:wat::core::string::concat "wat::kernel::Peer'<"
                         (:wat::core::string::concat fqdn-str
                           (:wat::core::string::concat "::Status,"
                             (:wat::core::string::concat fqdn-str "::Admin>")))))
     admin-init-kw  (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}::Admin::Init" :fqdn-str fqdn-str))
     admin-stop-kw  (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}::Admin::Stop" :fqdn-str fqdn-str))
     ;; arc 291 4a: Admin::Hibernate (unit, like Stop) + Admin::Resume (carries snapshot).
     admin-hibernate-kw (:wat::core::keyword/from-string
                          (:wat::core::string::interpolate "{fqdn-str}::Admin::Hibernate" :fqdn-str fqdn-str))
     admin-resume-kw  (:wat::core::keyword/from-string
                        (:wat::core::string::interpolate "{fqdn-str}::Admin::Resume" :fqdn-str fqdn-str))
     status-started-kw (:wat::core::keyword/from-string
                          (:wat::core::string::interpolate "{fqdn-str}::Status::Started" :fqdn-str fqdn-str))
     ;; arc 291 3a-ii-β: Status::Stopped — service replies with final state on admin stop.
     status-stopped-kw  (:wat::core::keyword/from-string
                          (:wat::core::string::interpolate "{fqdn-str}::Status::Stopped" :fqdn-str fqdn-str))
     ;; arc 291 4a: Status::Hibernated — service replies with full state on hibernate.
     status-hibernated-kw (:wat::core::keyword/from-string
                             (:wat::core::string::interpolate "{fqdn-str}::Status::Hibernated" :fqdn-str fqdn-str))
     dispatch-admin-name-str (:wat::core::string::interpolate "{fqdn-str}::dispatch-admin" :fqdn-str fqdn-str)
     dispatch-admin-name (:wat::core::keyword/from-string dispatch-admin-name-str)
     extract-addr-name-str (:wat::core::string::interpolate "{fqdn-str}::extract-addr" :fqdn-str fqdn-str)
     extract-addr-name (:wat::core::keyword/from-string extract-addr-name-str)

     ;; ── arc 291 3a-ii-α: Admin + Status defenums ──────────────────────────
     ;; Admin: Init carries the seed (ship-ty); Stop is unit (3a-ii-β dispatches it).
     ;; Status: Started carries the minted Address'; Final carries the final state.
     ;; :Stop and :Shutdown are unit variants (bare keyword, no field vector) —
     ;; matches as a bare keyword pattern (ev.fields.is_empty() ✓).
     ;; arc 291 4b-ii: Admin now has four variants:
     ;;   Init (startup seed), Stop (unit), Hibernate (unit), Resume (snapshot).
     ;;   Init and Resume both carry ::Record (not ::State — structs never cross the wire).
     admin-enum-def `(:wat::core::defenum ~admin-ty
                       :Init     ~init-params-vec
                       :Stop
                       :Hibernate
                       :Resume   ~init-params-vec)
     ;; arc 291 4b-ii: Status::Hibernated carries ::Record (not ::State).
     status-enum-def `(:wat::core::defenum ~status-ty
                             :Started   [addr     <- ~addr-ty]
                             :Stopped     [resp     <- ~resp-ty]
                             :Hibernated [snapshot <- ~record-ty])

     ;; ── arc 291 3a-ii-α: dispatch-admin defn ────────────────────────────────
     ;; fn [ai <- Admin] -> State
     ;;   (match ai ((Admin::Init seed) (<fqdn>::init seed))
     ;;             (Admin::Stop (assertion-failed! "Stop before Init")))
     ;; `ai` is a param in [ai <- admin-ty] Vector → checker does not recurse into
     ;; Vector children, so the literal symbol `ai` is hygienic.
     ;; `seed` and `_ignored` in match arms are match-arm binders → checker skips.
     ;; arc 291 4b-ii: dispatch-admin must stay exhaustive over all four Admin variants.
     ;;   Init(seed)       → (init seed)       — normal startup: init builds struct from record
     ;;   Resume(snapshot) → (init snapshot)   — resume: init rebuilds struct from saved record
     ;;   Stop             → assertion-failed! (not a startup message)
     ;;   Hibernate        → assertion-failed! (not a startup message)
     dispatch-admin-def `(:wat::core::defn ~dispatch-admin-name [ai <- ~admin-ty] -> ~state-ty
                            (:wat::core::match ai -> ~state-ty
                              ((~admin-init-kw ~@init-arg-names)   (~init-name ~@init-arg-names))
                              ((~admin-resume-kw ~@init-arg-names) (~init-name ~@init-arg-names))
                              (~admin-stop-kw
                                (:wat::kernel::assertion-failed!
                                  "defservice dispatch-admin: Stop received before Init/Resume (protocol error)"
                                  :wat::core::None
                                  :wat::core::None))
                              (~admin-hibernate-kw
                                (:wat::kernel::assertion-failed!
                                  "defservice dispatch-admin: Hibernate received before Init/Resume (protocol error)"
                                  :wat::core::None
                                  :wat::core::None))))

     ;; ── arc 291 3a-ii-α: extract-addr defn ───────────────────────────
     ;; fn [lu <- Status] -> addr-ty
     ;;   (match lu ((Status::Started addr) addr))
     ;; Passed to Locus/launch as lu-addr-kw so the generic ProcessOpts impl
     ;; can extract the Address' without naming per-service Status types.
     lu-sym     (:wat::core::symbol-node "lu")
     extract-addr-def `(:wat::core::defn ~extract-addr-name
                                  [lu <- ~status-ty] -> ~addr-ty
                                  (:wat::core::match lu -> ~addr-ty
                                    ((~status-started-kw addr) addr)
                                    (_ (:wat::kernel::assertion-failed!
                                         "defservice extract-addr: unexpected Status variant (expected Started)"
                                         :wat::core::None
                                         :wat::core::None))))

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

     ;; ── serve params argvec ───────────────────────────────────────────────────────
     ;; Template is a Vector node; checker does NOT recurse into Vector children.
     ;; self/l/clients/state in the Vector are fine as literal symbols.
     serve-params `[self    <- ~lineage-peer-ty
                    l       <- ~listener-ty
                    clients <- ~vector-ty
                    state   <- ~state-ty]

     ;; ── serve body: the poll'/ServiceEvent dispatch loop ─────────────────────────
     ;; All literals (self, l, clients, state, peer, idx, _cause) are in match patterns
     ;; or value positions — the checker only fires for let/fn binder Vectors.
     ;; arc 291 3a-ii-β: Admin::Stop arm — sends Status::Stopped(state) back up the
     ;; lineage peer (self), then terminates (returns nil, no recur). Admin::Init arriving
     ;; post-startup is a protocol error (assertion-failed!).
     ;; arc 291 4a: serve Admin dispatch must stay exhaustive over all four variants.
     ;;   Stop      → send Final(projected-state) up + terminate
     ;;   Hibernate → send Hibernated(full-state) up + terminate
     ;;   Init(_)   → assertion-failed! (startup-only message)
     ;;   Resume(_) → assertion-failed! (startup-only message)
     serve-body   `(:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
                     (:wat::spawn::ServiceEvent::Shutdown nil)
                     ((:wat::spawn::ServiceEvent::Connection peer)
                       (~serve-name self l (:wat::core::conj clients peer) state))
                     ((:wat::spawn::ServiceEvent::Admin admin-msg)
                       (:wat::core::match admin-msg -> :wat::core::nil
                         (~admin-stop-kw
                           (:wat::core::do
                             (:wat::kernel::send' self (~status-stopped-kw (~stop-project-name state)))
                             nil))
                         (~admin-hibernate-kw
                           (:wat::core::do
                             (:wat::kernel::send' self (~status-hibernated-kw (~hibernate-project-name state)))
                             nil))
                         ((~admin-init-kw ~@init-arg-names)
                           (:wat::kernel::assertion-failed!
                             "defservice serve: Admin::Init after startup (protocol error)"
                             :wat::core::None
                             :wat::core::None))
                         ((~admin-resume-kw ~@init-arg-names)
                           (:wat::kernel::assertion-failed!
                             "defservice serve: Admin::Resume after startup (protocol error)"
                             :wat::core::None
                             :wat::core::None))))
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
     ;; op-methods = per-op client methods only (no stop/hibernate); used for client-forms-def.
     op-methods    (:wat::core::foldl
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

     ;; ── arc 291 3a-ii-β: owner-only stop method (replaces the deleted client stop) ───
     ;; Method: (defn <fqdn>/stop [h <- Handle] -> state-ty ...)
     ;; Takes the Handle (unforgeable; never handed to clients); sends Admin::Stop down the
     ;; lineage peer (Handle/handle h); recv's Status::Stopped → extracts and returns state.
     ;; Uses symbol-node for `_` and `r` let binders (hygiene: Unquote at def time).
     stop-discard-sym  (:wat::core::symbol-node "_")
     stop-r-sym        (:wat::core::symbol-node "r")
     stop-method-name  (:wat::core::keyword/from-string
                         (:wat::core::string::interpolate "{fqdn-str}/stop" :fqdn-str fqdn-str))
     handle-handle-acc (:wat::core::keyword/from-string
                         (:wat::core::string::interpolate "{fqdn-str}::Handle/handle" :fqdn-str fqdn-str))
     stop-method-params `[h <- ~handle-name]
     stop-method-body  `(:wat::core::let
                          [~stop-discard-sym (:wat::kernel::send' (~handle-handle-acc h) ~admin-stop-kw)
                           ~stop-r-sym       (:wat::kernel::recv' (~handle-handle-acc h))]
                          (:wat::core::match ~stop-r-sym -> ~resp-ty
                            ((~status-stopped-kw resp) resp)
                            (_ (:wat::kernel::assertion-failed!
                                 "defservice stop: expected Status::Stopped"
                                 :wat::core::None
                                 :wat::core::None))))
     stop-method       `(:wat::core::defn ~stop-method-name ~stop-method-params -> ~resp-ty ~stop-method-body)
     ;; Extend op-methods with the owner-only stop (stop/hibernate are owner-only, NOT in client-forms).
     methods           (:wat::core::conj op-methods stop-method)

     ;; ── arc 291 4a: owner-only hibernate method (mirror of stop) ─────────────────
     ;; Method: (defn <fqdn>/hibernate [h <- Handle] -> state-ty ...)
     ;; Sends Admin::Hibernate (bare unit kw) down the lineage peer; recv's Status::Hibernated
     ;; which carries the WHOLE State (not a projection — that's what distinguishes hibernate from stop).
     ;; Uses symbol-node for `_` and `r` let binders (hygiene: Unquote at def time).
     hib-discard-sym   (:wat::core::symbol-node "_")
     hib-r-sym         (:wat::core::symbol-node "r")
     hibernate-method-name (:wat::core::keyword/from-string
                             (:wat::core::string::interpolate "{fqdn-str}/hibernate" :fqdn-str fqdn-str))
     hibernate-method-params `[h <- ~handle-name]
     hibernate-method-body  `(:wat::core::let
                               [~hib-discard-sym (:wat::kernel::send' (~handle-handle-acc h) ~admin-hibernate-kw)
                                ~hib-r-sym       (:wat::kernel::recv' (~handle-handle-acc h))]
                               (:wat::core::match ~hib-r-sym -> ~record-ty
                                 ((~status-hibernated-kw snapshot) snapshot)
                                 (_ (:wat::kernel::assertion-failed!
                                      "defservice hibernate: expected Status::Hibernated"
                                      :wat::core::None
                                      :wat::core::None))))
     hibernate-method  `(:wat::core::defn ~hibernate-method-name ~hibernate-method-params -> ~record-ty ~hibernate-method-body)
     ;; Extend methods with the owner-only hibernate (stop + hibernate, not in client-forms).
     methods           (:wat::core::conj methods hibernate-method)

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
     ;; arc 291 kwargs-start: locus-sym minted once so start-params + start-body (and resume pair)
     ;; share the same scope node — avoids HygieneScopeDivergence when kwargs-defn rebuilds $impl.
     locus-sym     (:wat::core::symbol-node "locus")
     ;; arc 291 3a-ii-β: launch<Op,Reply,State,Admin,Status> — Sh=Admin (ship), Lu=Status.
     launch-head-kw (:wat::core::keyword/from-string
                      (:wat::core::string::concat "wat::spawn::Locus/launch<"
                        (:wat::core::string::concat fqdn-str
                          (:wat::core::string::concat "::Op,"
                            (:wat::core::string::concat fqdn-str
                              (:wat::core::string::concat "::Reply,"
                                (:wat::core::string::concat fqdn-str
                                  (:wat::core::string::concat "::State,"
                                    (:wat::core::string::concat fqdn-str
                                      (:wat::core::string::concat "::Admin,"
                                        (:wat::core::string::concat fqdn-str "::Status>")))))))))))

     ;; ── arc 272 6b-ii-β: transport-agnostic service-forms ────────────────────────
     ;; service-forms-kw must be defined before start-body (which splices ~service-forms-kw).
     ;; service-forms-kw: the keyword :<fqdn>::service-forms — the name of the emitted def.
     service-forms-kw (:wat::core::keyword/from-string
                        (:wat::core::string::interpolate "{fqdn-str}::service-forms" :fqdn-str fqdn-str))
     ;; client-forms-kw: the keyword :<fqdn>::client-forms — the client face (per-op methods only).
     client-forms-kw  (:wat::core::keyword/from-string
                        (:wat::core::string::interpolate "{fqdn-str}::client-forms" :fqdn-str fqdn-str))
     ;; callee-cf-calls: for each svc keyword in :calls, a 0-arg call `(:<svc>::client-forms)`.
     ;; Used to prepend callee client contracts ahead of this service's own service-forms.
     callee-cf-calls  (:wat::core::map
                        (:wat::core::fn [svc-kw <- :wat::WatAST] -> :wat::WatAST
                          (:wat::core::let
                            [svc-str (:wat::core::keyword/to-string svc-kw)
                             cf-kw   (:wat::core::keyword/from-string
                                       (:wat::core::string::interpolate "{svc-str}::client-forms" :svc-str svc-str))]
                            `(~cf-kw)))
                        (:wat::core::ast->children calls-svcs))
     ;; The agnostic child :user::main: binds on :wat::spawn::service-locus (a FREE
     ;; name — defservice does NOT define it). The ProcessOpts launch arm prepends
     ;; `(def :wat::spawn::service-locus (process))` before spawning, so the child
     ;; universe resolves service-locus at startup to a ProcessOpts value.
     ;; self-peer S=addr-ty (child sends minted Address' up), R=ship-ty (parent sends
     ;; the EDN ship value down). The child recvs the ship value, applies init to build State,
     ;; then calls serve. serve is invoked via apply (dynamic keyword) — the child main
     ;; never statically names the per-service serve fn.
     ;; Hygiene: child main let binders (b/cm-self/_/ship/st) are synthetic names → must use
     ;; symbol-node + unquote so they appear as Unquote nodes in the template, not bare
     ;; Symbols that would trigger the ProgramBodyIntroducesName hygiene gate.
     cm-b-sym    (:wat::core::symbol-node "b")
     cm-self-sym (:wat::core::symbol-node "self")
     cm-und-sym  (:wat::core::symbol-node "_")
     cm-ship-sym (:wat::core::symbol-node "ship")
     cm-st-sym   (:wat::core::symbol-node "st")
     ;; arc 291 3a-ii-α: child-main-form uses the lineage protocol.
     ;; self-peer: Peer'<Status, Admin>
     ;;   child sends Status::Started(addr) UP, receives Admin DOWN.
     ;; The send' wraps addr in Status::Started (was: raw addr).
     ;; The recv' gets Admin; dispatch-admin applies to it (was: init applied to raw ship).
     child-main-form `(:wat::core::defn :user::main [] -> :wat::core::nil
                        (:wat::core::let
                          [~cm-b-sym    (:wat::kernel::listener' :wat::spawn::service-locus
                                            ~enum-name ~reply-name)
                           ~cm-self-sym (:wat::program::self-peer ~status-ty ~admin-ty)
                           ~cm-und-sym  (:wat::kernel::send' ~cm-self-sym
                                            (~status-started-kw (:wat::spawn::Bound/address ~cm-b-sym)))
                           ~cm-ship-sym (:wat::kernel::recv' ~cm-self-sym)
                           ~cm-st-sym   (:wat::core::apply -> ~state-ty
                                            (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                            ~cm-ship-sym [])]
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
     ;; own-forms-call: the full service-forms body (this service's server internals + child main).
     ;; When :calls is non-empty, callee client-forms are prepended via foldr+concat so they load
     ;; BEFORE this service's own forms (worker's State def references recorder::Op/Reply).
     own-forms-call  `(:wat::core::forms
                        ~record-def
                        ~state-def
                        ~@request-records
                        ~@response-records
                        (:wat::core::defenum ~enum-name ~@variants)
                        (:wat::core::defenum ~reply-name ~@reply-variants)
                        (:wat::core::defn ~serve-name ~serve-params
                          -> :wat::core::nil ~serve-body)
                        ~init-def
                        ~stop-project-def
                        ~hibernate-project-def
                        ~admin-enum-def
                        ~status-enum-def
                        ~dispatch-admin-def
                        ~extract-addr-def
                        ~child-main-form)
     ;; service-forms-body: if :calls is non-empty, foldr with concat prepends each callee's
     ;; client-forms ahead of own-forms-call. foldr(f, init=own-forms-call, xs=callee-cf-calls)
     ;; → (concat cf0 (concat cf1 … own-forms-call)) — callee forms first, correct load order.
     service-forms-body (:wat::core::if (:wat::core::i64::> (:wat::core::length callee-cf-calls) 0)
                          -> :wat::WatAST
                          (:wat::core::foldr
                            (:wat::core::fn [cf-call <- :wat::WatAST  acc <- :wat::WatAST] -> :wat::WatAST
                              `(:wat::core::concat ~cf-call ~acc))
                            own-forms-call
                            callee-cf-calls)
                          own-forms-call)
     service-forms-def `(:wat::core::defn ~service-forms-kw
                          [] -> :wat::core::Vector<wat::WatAST>
                          ~service-forms-body)
     ;; client-forms-def: the CLIENT face — request/response records, Op/Reply enums,
     ;; per-op constructors, per-op methods (op-methods only — no stop/hibernate).
     ;; Shipped to callee consumers via :calls so their child processes can resolve
     ;; callee/method and callee/x-request without carrying the server internals.
     client-forms-def `(:wat::core::defn ~client-forms-kw
                         [] -> :wat::core::Vector<wat::WatAST>
                         (:wat::core::forms
                           ~@request-records
                           ~@response-records
                           (:wat::core::defenum ~enum-name ~@variants)
                           (:wat::core::defenum ~reply-name ~@reply-variants)
                           ~@constructors
                           ~@op-methods))

     ;; arc 291: start-params uses the init fn's single param binder (name <- :T) so start
     ;; takes the EDN seed (or state0 for default) as its 2nd param. ship-ref is the symbol.
     ;; arc 291 3a-ii-α: ship is wrapped in Admin::Init so the lineage peer carries Admin values.
     ;; dispatch-admin-name is passed in place of init-name so both tiers apply it.
     ;; extract-addr-name is passed as lu-addr-kw for the ProcessOpts impl.
     ;; arc 291 kwargs-start: flip to Form A all-kwargs; locus-sym shared with body for hygiene.
     start-params  `[& [~locus-sym <- :wat::spawn::Locus  ~@init-param]]
     start-body    `(:wat::core::let
                      [~lr-sym (~launch-head-kw ~locus-sym
                                 (~admin-init-kw ~@init-arg-names)
                                 (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                 (:wat::core::keyword/from-string ~serve-name-str)
                                 (~service-forms-kw)
                                 (:wat::core::keyword/from-string ~extract-addr-name-str))]
                      (~handle-name (:wat::spawn::Launched/handle ~lr-sym)
                                    (:wat::spawn::Launched/address ~lr-sym)))
     start-fn      `(:wat::core::defn ~start-name ~start-params -> ~handle-name ~start-body)

     ;; ── arc 291 4b-ii: resume fn (mirror of start, ships Admin::Resume instead of Admin::Init) ──
     ;; (defn <fqdn>/resume [locus <- :wat::spawn::Locus  snapshot <- ~record-ty] -> ~handle-name
     ;;   (let [lr (launch<…> locus (Admin::Resume snapshot) dispatch-admin serve service-forms lu-addr)]
     ;;     (Handle (Launched/handle lr) (Launched/address lr))))
     ;; dispatch-admin routes Admin::Resume → (init snapshot) to rebuild the struct.
     ;; launch is UNCHANGED — resume reuses the same machinery.
     ;; `snapshot` param binder: use a symbol-node (hygiene: Unquote at def time).
     resume-name    (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}/resume" :fqdn-str fqdn-str))
     ;; arc 291 kwargs-start: mirrors start-params — kwargs Form A; locus-sym shared for hygiene.
     ;; All init binders are spliced in; resume re-accepts all live operating-inputs.
     resume-params  `[& [~locus-sym <- :wat::spawn::Locus  ~@init-param]]
     resume-body    `(:wat::core::let
                       [~lr-sym (~launch-head-kw ~locus-sym
                                  (~admin-resume-kw ~@init-arg-names)
                                  (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                  (:wat::core::keyword/from-string ~serve-name-str)
                                  (~service-forms-kw)
                                  (:wat::core::keyword/from-string ~extract-addr-name-str))]
                       (~handle-name (:wat::spawn::Launched/handle ~lr-sym)
                                     (:wat::spawn::Launched/address ~lr-sym)))
     resume-fn      `(:wat::core::defn ~resume-name ~resume-params -> ~handle-name ~resume-body)

     ;; ── C.3: Handle record ───────────────────────────────────────────────────────
     ;; (Record::def <fqdn>::Handle
     ;;   [handle <- Peer'<Admin,Status>
     ;;    addr   <- :wat::kernel::Address'<fqdn::Op,fqdn::Reply>])
     ;; arc 291 3a-ii-β: handle is the owner-only lineage peer (admin channel).
     ;; Peer'<Admin,Status> — owner sends Admin (down), receives Status (up).
     ;; Thread'<Admin,Status> and Process'<Admin,Status> both satisfy this field
     ;; (send'/recv' intrinsics accept Thread'|Process'|Peer' uniformly).
     ;; addr carries the typed Address'<Op,Reply> for client connect'.
     handle-peer-ty (:wat::core::keyword/from-string
                      (:wat::core::string::concat "wat::kernel::Peer'<"
                        (:wat::core::string::concat fqdn-str
                          (:wat::core::string::concat "::Admin,"
                            (:wat::core::string::concat fqdn-str "::Status>")))))
     handle-fields `[handle <- ~handle-peer-ty addr <- ~addr-ty]
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
       ~record-def
       ~state-def
       ~@request-records
       ~@response-records
       (:wat::core::defenum ~enum-name ~@variants)
       (:wat::core::defenum ~reply-name ~@reply-variants)
       ~admin-enum-def
       ~status-enum-def
       (:wat::core::defn ~serve-name ~serve-params -> :wat::core::nil ~serve-body)
       ~init-def
       ~stop-project-def
       ~hibernate-project-def
       ~dispatch-admin-def
       ~extract-addr-def
       ~@constructors
       ~@methods
       ~service-forms-def
       ~client-forms-def
       ~start-fn
       ~resume-fn
       ~handle-record)))
