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
(:wat::core::defenum :wat::service::Outcome<S,R> :wat::enum::Pure
  :Reply [state <- :S  reply <- :R]
  :Stop  [state <- :S  reply <- :R])

;; ── The canonical clause order — the story a service tells ──────────────────────
;; A defservice reads top-to-bottom as a sentence about an actor. Order is
;; compiler-free (all-kwargs); this is house style. Foundation precedes, elaboration
;; follows — a parent founds the durable record (leads it).
;;
;;   :durable-parent   what I'm built from   (optional — the durable record's parent; e.g. holon)
;;   :durable          what I remember       (the soul: EDN, crosses the wire, survives hibernation)
;;   :ephemeral        what I carry          (the body: resources + peer clients; never crosses)
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
     ;; Arc 293 S2: :satisfies (name a surface — reference its S1-synthesized protocol) and
     ;; :impls (bodies-only op implementations, in place of :ops) join the recognized clauses.
     known-clauses  (:wat::core::HashMap/assoc
                      (:wat::core::HashMap/assoc
                       (:wat::core::HashMap/assoc
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
                        "satisfies" true)
                      "impls" true)
                      ;; Arc 278 S4d: :peers — the explicit s2s dependency DAG (dialed peer surfaces).
                      "peers" true)
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
                                  " — recognized clauses: :durable :ephemeral :ops :init :hibernate :stop :durable-parent :satisfies :impls :peers"))))))
                      (:wat::core::HashMap :wat::core::String :wat::WatAST)
                      (:wat::core::range 0 n-clause-pairs))
     ;; ── Arc 293 S2: :ops vs :satisfies mode ────────────────────────────────────
     ;; A service EITHER mints its own protocol (:ops) OR wears a surface's (:satisfies +
     ;; :impls). Exactly one of {:ops, :satisfies}; :impls iff :satisfies; :ops iff not.
     satisfies?     (:wat::core::HashMap/contains-key? clause-map "satisfies")
     has-ops?       (:wat::core::HashMap/contains-key? clause-map "ops")
     has-impls?     (:wat::core::HashMap/contains-key? clause-map "impls")
     ;; ── Arc 278 S4c: :satisfies + :impls MANDATORY; :ops RETIRED (illegal) ─────────
     ;; Every service declares a surface and wears it (the AWS service model). :ops
     ;; (mint-your-own-protocol) is annihilated — a heretic screams and migrates.
     _ops-retired   (:wat::core::if has-ops?
                      -> :wat::core::nil
                      (:wat::core::macro-error "defservice: :ops is RETIRED — declare a surface (defsurface :nature :wat::kernel::Peer' with method members + per-op Request record and <Op>Response) and :satisfies it with :impls (bodies only). Exemplar: wat/query.wat + wat/query/mem.wat.")
                      nil)
     _needs-surface (:wat::core::if satisfies?
                      -> :wat::core::nil
                      nil
                      (:wat::core::macro-error "defservice: :satisfies is required — name the surface this service wears (see wat/query.wat for the exemplar)."))
     _needs-impls   (:wat::core::if has-impls?
                      -> :wat::core::nil
                      nil
                      (:wat::core::macro-error "defservice: :satisfies requires :impls (the op bodies, bodies-only)."))
     ;; ops := the op-bearing clause value — :impls when satisfies, else :ops.
     ops            (:wat::core::if satisfies?
                      -> :wat::WatAST
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "impls")
                        "defservice: :impls clause missing value")
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "ops")
                        "defservice: :ops clause missing value"))
     ;; The protocol namespace: the surface's when :satisfies (its S1 ::Op/::Reply +
     ;; user-declared request/response records), else the service's own fqdn.
     surface-node   (:wat::core::if satisfies?
                      -> :wat::WatAST
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "satisfies")
                        "defservice: :satisfies needs a surface")
                      fqdn)
     proto-str      (:wat::core::keyword/to-string surface-node)
     surface-kw     (:wat::core::keyword/from-string proto-str)

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

     ;; :durable-parent — optional, default :wat::core::Record
     state-parent   (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "durable-parent")
                      -> :wat::WatAST
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "durable-parent")
                        "defservice: :durable-parent needs a value")
                      :wat::core::Record)

     ;; ── 4b-ii: mint state-ty as :<fqdn>::State, record-ty as :<fqdn>::Record ──
     state-ty       (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}::State" :fqdn-str fqdn-str))
     record-ty      (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}::Record" :fqdn-str fqdn-str))

     ;; ── 4b-ii: :init option ────────────────────────────────────────────────────
     ;; :init : Record → State. Default (fn [d <- ::Record] -> ::State (::State d))
     ;;   when :ephemeral is empty. When :ephemeral non-empty and :init absent → macro-error.
     ;; A synthetic symbol-node "record" for the default init param (hygiene: Unquote at def time).
     ;; arc 291 kwargs-start: renamed "d"→"record" so the default-init start kwarg is :record.
     d-sym          (:wat::core::symbol-node "record")
     s-sym          (:wat::core::symbol-node "s")
     ;; state-new-kw: :<fqdn>::State — the bare struct ctor (arc 293.R2.3: /new annihilated)
     state-new-kw   (:wat::core::keyword/from-string
                      (:wat::core::string::interpolate "{fqdn-str}::State" :fqdn-str fqdn-str))
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
     ;; Arc 118.2a — spliced via ~@init-arg-names into quasiquote match arms below (and
     ;; in start-body/resume-body); unquote-splicing needs a concrete Vec. `mapv` is a
     ;; wat-level defn (wat/seq.wat) — program-body macro-expand-time eval runs BEFORE
     ;; any user/stdlib defn registration (see src/macros/eval.rs's load-bearing-invariant
     ;; doc), so it is UnknownFunction here regardless of the pure-total allow-list.
     ;; Build the Vec eagerly via foldl + conj instead (both Rust-native, always safe) —
     ;; same pattern as Record.wat's defrecord / core.wat's format macro fixes.
     init-arg-names (:wat::core::foldl
                      (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                       i <- :wat::core::i64]
                        -> :wat::core::Vector<wat::WatAST>
                        (:wat::core::conj acc
                          (:wat::core::Option/expect
                            (:wat::core::get init-param (:wat::core::i64::* i 3))
                            "defservice: init param name out of bounds")))
                      (:wat::core::Vector :wat::WatAST)
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
     ;; record-def: (:wat::core::Record::def ::Record [durable-fields]) (or holon parent)
     ;; state-def:  (:wat::core::defstruct ::State [durable <- ::Record <ephemeral-fields...>])
     ;;   The 3 tokens `durable <- ~record-ty` are prepended to ephemeral children.
     state-parent-str (:wat::core::keyword/to-string state-parent)
     record-def   (:wat::core::if (:wat::core::= state-parent-str "wat::holon::Record")
                    -> :wat::WatAST
                    `(:wat::holon::defrecord ~record-ty ~durable-fields)
                    `(:wat::core::defrecord ~record-ty ~durable-fields))
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

     ;; ── Arc 278 S4d: :peers — the s2s dependency DAG + cross-fork manifest ──────────
     ;; A :satisfies service that DIALS another service holds a client Peer'<S::Op,S::Reply>
     ;; in a ROOT :ephemeral field and calls S's surface methods on it. :peers [:S1 …] is the
     ;; EXPLICIT declaration of those dialed surfaces.
     ;;
     ;; BIJECTION (set equality, by surface): the SET of :peers surfaces MUST EQUAL the SET of
     ;; root-ephemeral peer-field surfaces. A peer field is any root :ephemeral field whose type
     ;; is `:wat::kernel::Peer'<S::Op,S::Reply>` — its surface is S (the first type-arg minus the
     ;; trailing `::Op`). :peers entry with no matching ephemeral peer field → macro-error (missing);
     ;; ephemeral peer field whose surface is not in :peers → macro-error (extra/undeclared).
     ;; Two ephemeral peers of the same surface → one :peers entry (set equality). Only ROOT
     ;; ephemeral fields are walked, so a peer must be a top-level :ephemeral field.
     ;;
     ;; MANIFEST: for each :peers surface S, (S::surface-forms) is concatenated into the child
     ;; bundle (below, in service-forms-def) so a forked child resolves the DIALED surface's
     ;; Op/Reply/records (else the child StartupErrors on S's types when serve/methods reference them).
     ;;
     ;; :peers is OPTIONAL: a service with no dialed peers omits it. But if it has ephemeral peer
     ;; fields, :peers is REQUIRED to match them (an unmatched ephemeral peer → the "extra" error).
     peers-node     (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "peers")
                      -> :wat::WatAST
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "peers")
                        "defservice: :peers needs a value")
                      empty-vec)
     peers-children (:wat::core::ast->children peers-node)
     ;; peers-surfaces: Vector<String> — the declared peer surface fqdns (keyword/to-string each).
     peers-surfaces (:wat::core::foldl
                      (:wat::core::fn [acc <- :wat::core::Vector<wat::core::String>
                                       pk  <- :wat::WatAST]
                        -> :wat::core::Vector<wat::core::String>
                        (:wat::core::conj acc (:wat::core::keyword/to-string pk)))
                      (:wat::core::Vector :wat::core::String)
                      peers-children)
     ;; ephemeral-peer-surfaces: Vector<String> — the surface of each ROOT ephemeral peer field.
     ;; ephemeral-children is the flat token vec [name <- :Type name <- :Type …]; the type node
     ;; of field i is at index i*3+2. A peer field's type is a keyword containing `wat::kernel::Peer'<`
     ;; whose first type-arg ends in `::Op`; the surface is that arg minus `::Op`.
     ephemeral-peer-surfaces
                    (:wat::core::foldl
                      (:wat::core::fn [acc <- :wat::core::Vector<wat::core::String>
                                       i   <- :wat::core::i64]
                        -> :wat::core::Vector<wat::core::String>
                        (:wat::core::let
                          [ty-node (:wat::core::Option/expect
                                     (:wat::core::get ephemeral-children
                                       (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                                     "defservice: ephemeral field type out of bounds")]
                          (:wat::core::if (:wat::core::= (:wat::core::ast-kind ty-node) "keyword")
                            -> :wat::core::Vector<wat::core::String>
                            (:wat::core::let
                              [ty-str (:wat::core::keyword/to-string ty-node)]
                              (:wat::core::if (:wat::core::string::contains? ty-str "wat::kernel::Peer'<")
                                -> :wat::core::Vector<wat::core::String>
                                (:wat::core::let
                                  ;; tail := everything after the first "Peer'<"; = "S::Op,S::Reply>"
                                  [tail      (:wat::core::second (:wat::core::string::split ty-str "Peer'<"))
                                   first-arg (:wat::core::first (:wat::core::string::split tail ","))]
                                  (:wat::core::if (:wat::core::string::ends-with? first-arg "::Op")
                                    -> :wat::core::Vector<wat::core::String>
                                    (:wat::core::conj acc
                                      (:wat::core::string::subs first-arg 0
                                        (:wat::core::i64::- (:wat::core::string::length first-arg) 4)))
                                    acc))
                                acc))
                            acc)))
                      (:wat::core::Vector :wat::core::String)
                      (:wat::core::range 0 (:wat::core::i64::/ ephemeral-len 3)))
     ;; BIJECTION check 1 (missing): every :peers surface must have a matching ephemeral peer field.
     _peers-missing (:wat::core::foldl
                      (:wat::core::fn [ok <- :wat::core::bool  ps <- :wat::core::String]
                        -> :wat::core::bool
                        (:wat::core::if (:wat::core::Vector/contains? ephemeral-peer-surfaces ps)
                          -> :wat::core::bool
                          ok
                          (:wat::core::macro-error
                            (:wat::core::string::concat fqdn-str
                              (:wat::core::string::concat ": :peers declares surface :"
                                (:wat::core::string::concat ps
                                  (:wat::core::string::concat
                                    " but no :ephemeral field is typed :wat::kernel::Peer'<"
                                    (:wat::core::string::concat ps
                                      "::Op,…::Reply> — add the dialed peer as a root :ephemeral field, or drop it from :peers"))))))))
                      true
                      peers-surfaces)
     ;; BIJECTION check 2 (extra/undeclared): every ephemeral peer field's surface must be in :peers.
     _peers-extra   (:wat::core::foldl
                      (:wat::core::fn [ok <- :wat::core::bool  es <- :wat::core::String]
                        -> :wat::core::bool
                        (:wat::core::if (:wat::core::Vector/contains? peers-surfaces es)
                          -> :wat::core::bool
                          ok
                          (:wat::core::macro-error
                            (:wat::core::string::concat fqdn-str
                              (:wat::core::string::concat ": :ephemeral holds a dialed Peer'<"
                                (:wat::core::string::concat es
                                  (:wat::core::string::concat "::Op,…::Reply> but surface :"
                                    (:wat::core::string::concat es
                                      (:wat::core::string::concat
                                        " is not declared in :peers — add :peers [… :"
                                        (:wat::core::string::concat es " …] (the explicit s2s dependency DAG)"))))))))))
                      true
                      ephemeral-peer-surfaces)
     ;; peer-forms-calls: Vector<WatAST> of `(:S::surface-forms)` call nodes — one per :peers surface.
     ;; Spliced into the service-forms concat (below) so each dialed surface's forms cross the fork.
     peer-forms-calls (:wat::core::foldl
                        (:wat::core::fn [acc   <- :wat::core::Vector<wat::WatAST>
                                         s-str <- :wat::core::String]
                          -> :wat::core::Vector<wat::WatAST>
                          (:wat::core::let
                            [sf-kw (:wat::core::keyword/from-string
                                     (:wat::core::string::concat s-str "::surface-forms"))]
                            (:wat::core::conj acc `(~sf-kw))))
                        (:wat::core::Vector :wat::WatAST)
                        peers-surfaces)

     ;; Arc 293 S2 — Op/Reply live under the PROTOCOL namespace (proto-str): the surface's
     ;; when :satisfies, else this service's own fqdn (identical to pre-S2 for the :ops path).
     enum-name     (:wat::core::keyword/from-string
                     (:wat::core::string::interpolate "{proto-str}::Op" :proto-str proto-str))
     reply-name    (:wat::core::keyword/from-string
                     (:wat::core::string::interpolate "{proto-str}::Reply" :proto-str proto-str))
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
     ;; Parametric type keywords for serve's typed params. Arc 293 S2 — Op/Reply are the
     ;; PROTOCOL's (proto-str), so a :satisfies service's serve/client peers share the
     ;; surface's uniform Address'<S::Op,S::Reply>. (proto-str = fqdn-str for the :ops path.)
     ;; Peer'<proto::Reply,proto::Op>
     peer-ty       (:wat::core::keyword/from-string
                     (:wat::core::string::concat "wat::kernel::Peer'<"
                       (:wat::core::string::concat proto-str
                         (:wat::core::string::concat "::Reply,"
                           (:wat::core::string::concat proto-str "::Op>")))))
     ;; Listener'<proto::Op,proto::Reply>
     listener-ty   (:wat::core::keyword/from-string
                     (:wat::core::string::concat "wat::kernel::Listener'<"
                       (:wat::core::string::concat proto-str
                         (:wat::core::string::concat "::Op,"
                           (:wat::core::string::concat proto-str "::Reply>")))))
     ;; Vector<Peer'<proto::Reply,proto::Op>>
     vector-ty     (:wat::core::keyword/from-string
                     (:wat::core::string::concat "wat::core::Vector<wat::kernel::Peer'<"
                       (:wat::core::string::concat proto-str
                         (:wat::core::string::concat "::Reply,"
                           (:wat::core::string::concat proto-str "::Op>>")))))
     ;; Address'<proto::Op,proto::Reply>
     addr-ty       (:wat::core::keyword/from-string
                     (:wat::core::string::concat "wat::kernel::Address'<"
                       (:wat::core::string::concat proto-str
                         (:wat::core::string::concat "::Op,"
                           (:wat::core::string::concat proto-str "::Reply>")))))
     ;; Client Peer'<proto::Op,proto::Reply> — connect'(Address'<Op,Reply>) → Peer'<Op,Reply>.
     ;; This is the client-side peer (sends Op, receives Reply); distinct from
     ;; peer-ty (Peer'<Reply,Op>) which is the server-side peer (accepts via listener').
     client-peer-ty (:wat::core::keyword/from-string
                      (:wat::core::string::concat "wat::kernel::Peer'<"
                        (:wat::core::string::concat proto-str
                          (:wat::core::string::concat "::Op,"
                            (:wat::core::string::concat proto-str "::Reply>")))))

     ;; ── arc 291 3a-ii-α: lineage protocol types ──────────────────────────────
     ;; Admin enum:     :<fqdn>::Admin  — what the owner sends DOWN the lineage peer.
     ;;   :Init [seed <- :ship-ty]  — startup init-args (replaces raw ship)
     ;;   :Stop                     — owner-initiated stop (3a-ii-β dispatches this)
     ;; Status enum: :<fqdn>::Status — what the service sends UP the lineage peer.
     ;;   :Started [addr <- :addr-ty]    — startup address handoff (replaces raw addr)
     ;;   :Stopped   [state <- :state-ty]  — stop response (3a-ii-β uses this)
     ;;
     ;; self-peer type in child-main-form: ThreadSelfPeer'<Status, Admin>
     ;;   child sends Status up, receives Admin down.
     ;;   Arc 293.W.2d: thread-tier uses ThreadSelfPeer' (any I/O); process-tier `apply`
     ;;   bypasses the type check so the same serve fn works for both tiers.
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
     ;; Arc 293.W.2d: serve's self is ThreadSelfPeer'<Status,Admin> for thread-tier.
     ;; Process-tier calls serve via `apply` (Locus/launch child-main-form), which bypasses
     ;; the type check — the process-tier Peer'<Status,Admin> from self-peer is accepted at
     ;; runtime without a static mismatch.
     lineage-peer-ty (:wat::core::keyword/from-string
                       (:wat::core::string::concat "wat::kernel::ThreadSelfPeer'<"
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
     ;; arc 278: Admin::AllowPeer[pids] — owner grants a vec of caller pids to the callee's
     ;; process-tier accept-gate (the circuit builder wiring process peers). Status::PeersAllowed
     ;; is the request/reply ack — the owner blocks on it so the grant is applied before the
     ;; caller dials (grant-before-dial ordering). Both cross the owner-only lineage peer.
     admin-allow-peer-kw (:wat::core::keyword/from-string
                           (:wat::core::string::interpolate "{fqdn-str}::Admin::AllowPeer" :fqdn-str fqdn-str))
     status-peers-allowed-kw (:wat::core::keyword/from-string
                               (:wat::core::string::interpolate "{fqdn-str}::Status::PeersAllowed" :fqdn-str fqdn-str))
     ;; arc 278: fold binders for the serve AllowPeer arm's (allow' l pid) sweep — synthetic
     ;; fn binders introduced in the serve template → symbol-node + unquote for hygiene.
     allow-acc-sym (:wat::core::symbol-node "acc")
     allow-pid-sym (:wat::core::symbol-node "pid")
     ;; arc 293: Admin::DenyPeer[pids] — mirror of AllowPeer, owner revokes a vec of caller
     ;; pids from the callee's process-tier accept-gate. Status::PeersDenied is the
     ;; request/reply ack — the owner blocks on it so the revoke is applied before it returns.
     admin-deny-peer-kw (:wat::core::keyword/from-string
                          (:wat::core::string::interpolate "{fqdn-str}::Admin::DenyPeer" :fqdn-str fqdn-str))
     status-peers-denied-kw (:wat::core::keyword/from-string
                              (:wat::core::string::interpolate "{fqdn-str}::Status::PeersDenied" :fqdn-str fqdn-str))
     ;; arc 293: fold binders for the serve DenyPeer arm's (deny' l pid) sweep — synthetic
     ;; fn binders introduced in the serve template → symbol-node + unquote for hygiene.
     deny-acc-sym (:wat::core::symbol-node "acc")
     deny-pid-sym (:wat::core::symbol-node "pid")
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
     ;; arc 278: Admin::AllowPeer[pids] — a vec of caller pids to grant to the accept-gate.
     ;; arc 293: Admin::DenyPeer[pids] — mirror, a vec of caller pids to revoke from it.
     admin-enum-def `(:wat::core::defenum ~admin-ty :wat::enum::Pure
                       :Init     ~init-params-vec
                       :Stop
                       :Hibernate
                       :Resume   ~init-params-vec
                       :AllowPeer [pids <- (:wat::core::Vector :wat::core::i64)]
                       :DenyPeer [pids <- (:wat::core::Vector :wat::core::i64)])
     ;; arc 291 4b-ii: Status::Hibernated carries ::Record (not ::State).
     ;; arc 278: Status::PeersAllowed (unit) — the AllowPeer request/reply ack.
     ;; arc 293: Status::PeersDenied (unit) — the DenyPeer request/reply ack.
     status-enum-def `(:wat::core::defenum ~status-ty :wat::enum::Pure
                             :Started   [addr     <- ~addr-ty]
                             :Stopped     [resp     <- ~resp-ty]
                             :Hibernated [snapshot <- ~record-ty]
                             :PeersAllowed
                             :PeersDenied)

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
                                  :wat::core::None))
                              ((~admin-allow-peer-kw pids)
                                (:wat::kernel::assertion-failed!
                                  "defservice dispatch-admin: AllowPeer received before Init/Resume (protocol error)"
                                  :wat::core::None
                                  :wat::core::None))
                              ((~admin-deny-peer-kw pids)
                                (:wat::kernel::assertion-failed!
                                  "defservice dispatch-admin: DenyPeer received before Init/Resume (protocol error)"
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
     impl-clauses  (:wat::core::if satisfies?
                     -> :wat::core::Vector<wat::WatAST>
                     clauses
                     (:wat::core::Vector :wat::WatAST))

     ;; ── Arc 293 S2: serve op-arms for :impls (bodies-only over the surface's protocol) ──
     ;; Each impl is `(<op> [s req] body)` — `s` = the :State (server self), `req` = the WHOLE
     ;; request record (bound straight from the <S>::Op::<Op> variant's `req` field). The arm:
     ;;   ((<S>::Op::<Op> req) (match (let [s state] body) -> nil
     ;;      ((Outcome::Reply new-state resp) (do (send' … (<S>::Reply::<Op> resp)) (serve …)))
     ;;      ((Outcome::Stop  final    resp) (do (send' … (<S>::Reply::<Op> resp)) nil))))
     ;; COVERAGE IS FREE: this match is over <S>::Op (S1's enum); a missing :impl leaves a
     ;; variant unhandled → non-exhaustive match → compile error (no coverage check to write).
     ;; Hygiene: `req` in the pattern comes from ~req-binder (the impl's own binder, unquoted →
     ;; Unquote node → checker skips); let-bindings [s state] built via with-children → ~-spliced.
     serve-op-arms (:wat::core::foldl
                     (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                      clause <- :wat::WatAST]
                       -> :wat::core::Vector<wat::WatAST>
                       (:wat::core::let
                         [ch            (:wat::core::ast->children clause)
                          op-node       (:wat::core::first ch)
                          param-vec     (:wat::core::first (:wat::core::drop ch 1))
                          body          (:wat::core::first (:wat::core::drop ch 2))
                          param-ch      (:wat::core::ast->children param-vec)
                          s-binder      (:wat::core::first param-ch)
                          req-binder    (:wat::core::first (:wat::core::rest param-ch))
                          op-str        (:wat::core::ast-name op-node)
                          op-pascal     (:wat::core::string::kebab->pascal-in surface-kw op-str)
                          op-variant-kw (:wat::core::keyword/from-string
                                          (:wat::core::string::concat proto-str
                                            (:wat::core::string::interpolate "::Op::{op-pascal}" :op-pascal op-pascal)))
                          reply-variant-kw (:wat::core::keyword/from-string
                                             (:wat::core::string::concat proto-str
                                               (:wat::core::string::interpolate "::Reply::{op-pascal}" :op-pascal op-pascal)))
                          state-sym     (:wat::core::symbol-node "state")
                          ;; let-bindings [s-binder state] — bind the impl's state param to serve's `state`.
                          binding-items (:wat::core::conj
                                          (:wat::core::conj
                                            (:wat::core::Vector :wat::WatAST)
                                            s-binder)
                                          state-sym)
                          let-bindings  (:wat::core::with-children param-vec binding-items)
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
                           `((~op-variant-kw ~req-binder) ~outcome-match))))
                     (:wat::core::Vector :wat::WatAST)
                     impl-clauses)

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
                         ;; arc 278: AllowPeer[pids] — fold (allow' l pid) over the vec on the
                         ;; serve loop's OWN listener l (process-tier gate), ack PeersAllowed up
                         ;; the lineage peer (request/reply — owner blocks so grant-before-dial
                         ;; ordering holds), then CONTINUE serving (recur — no state change).
                         ((~admin-allow-peer-kw pids)
                           (:wat::core::do
                             (:wat::core::foldl
                               (:wat::core::fn [~allow-acc-sym <- :wat::core::nil
                                                ~allow-pid-sym <- :wat::core::i64] -> :wat::core::nil
                                 (:wat::kernel::allow' l ~allow-pid-sym))
                               nil
                               pids)
                             (:wat::kernel::send' self ~status-peers-allowed-kw)
                             (~serve-name self l clients state)))
                         ;; arc 293: DenyPeer[pids] — mirror, fold (deny' l pid) over the vec on
                         ;; the serve loop's OWN listener l (process-tier gate), ack PeersDenied up
                         ;; the lineage peer (request/reply — owner blocks so revoke-before-return
                         ;; ordering holds), then CONTINUE serving (recur — no state change).
                         ((~admin-deny-peer-kw pids)
                           (:wat::core::do
                             (:wat::core::foldl
                               (:wat::core::fn [~deny-acc-sym <- :wat::core::nil
                                                ~deny-pid-sym <- :wat::core::i64] -> :wat::core::nil
                                 (:wat::kernel::deny' l ~deny-pid-sym))
                               nil
                               pids)
                             (:wat::kernel::send' self ~status-peers-denied-kw)
                             (~serve-name self l clients state)))
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

     ;; ── Arc 293 S2: client methods for :impls (over the surface's protocol) ─────────────
     ;; `(defn <fqdn>/<op> [c <- Peer'<S::Op,S::Reply>  req <- <S>::<Op>Request] -> <S>::<Op>Response
     ;;    (let [_ (send' c (<S>::Op::<Op> req))  r (recv' c)]
     ;;      (match r ((<S>::Reply::<Op> resp) resp) …)))
     ;; The client fn is SERVICE-namespaced (<fqdn>/<op>) — the SURFACE-namespaced name <S>/<op>
     ;; is already the surface's method-dispatch stub (defsurface registers it; receiver = a Store
     ;; satisfier). The blind/uniform side is the shared Op/Reply protocol + Address'<S::Op,S::Reply>
     ;; type; the surface method <S>/<op> becomes the blind entry once a satisfier extend-type wires
     ;; it to this concrete client fn (S4). Request/response records are the surface's own
     ;; (user-declared `<S>::<Op>Request` / `<S>::<Op>Response` — the S1/gRPC naming convention).
     op-methods    (:wat::core::foldl
                     (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                      clause <- :wat::WatAST]
                       -> :wat::core::Vector<wat::WatAST>
                       (:wat::core::let
                         [ch              (:wat::core::ast->children clause)
                          op-node         (:wat::core::first ch)
                          op-str          (:wat::core::ast-name op-node)
                          op-pascal       (:wat::core::string::kebab->pascal-in surface-kw op-str)
                          method-name     (:wat::core::keyword/from-string
                                            (:wat::core::string::concat fqdn-str
                                              (:wat::core::string::interpolate "/{op-str}" :op-str op-str)))
                          req-ty          (:wat::core::keyword/from-string
                                            (:wat::core::string::concat proto-str
                                              (:wat::core::string::interpolate "::{op-pascal}Request" :op-pascal op-pascal)))
                          resp-ty         (:wat::core::keyword/from-string
                                            (:wat::core::string::concat proto-str
                                              (:wat::core::string::interpolate "::{op-pascal}Response" :op-pascal op-pascal)))
                          op-variant-kw   (:wat::core::keyword/from-string
                                            (:wat::core::string::concat proto-str
                                              (:wat::core::string::interpolate "::Op::{op-pascal}" :op-pascal op-pascal)))
                          reply-variant-kw (:wat::core::keyword/from-string
                                             (:wat::core::string::concat proto-str
                                               (:wat::core::string::interpolate "::Reply::{op-pascal}" :op-pascal op-pascal)))
                          method-params   `[c <- ~client-peer-ty req <- ~req-ty]
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
                     impl-clauses)

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
     ;; Extend op-methods with the owner-only stop (stop/hibernate are owner-only, not per-op).
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
     ;; Extend methods with the owner-only hibernate (stop + hibernate, not per-op).
     methods           (:wat::core::conj methods hibernate-method)

     ;; ── arc 278: owner-only grant method (mirror of stop) ────────────────────────
     ;; Method: (defn <fqdn>/grant [h <- Handle  pids <- (Vector i64)] -> nil ...)
     ;; Takes the Handle (unforgeable; never handed to clients — clients hold only a client
     ;; Peer', so a client has NO grant path). Sends Admin::AllowPeer[pids] down the lineage
     ;; peer; recv's Status::PeersAllowed → the grant is applied before this returns (so the
     ;; circuit builder's post-spawn grant lands before the caller dials). Callable any time,
     ;; repeatedly, mid-life. Uses symbol-node for `_`/`r` binders (hygiene: Unquote at def time).
     grant-discard-sym (:wat::core::symbol-node "_")
     grant-r-sym       (:wat::core::symbol-node "r")
     grant-method-name (:wat::core::keyword/from-string
                         (:wat::core::string::interpolate "{fqdn-str}/grant" :fqdn-str fqdn-str))
     grant-method-params `[h <- ~handle-name  pids <- (:wat::core::Vector :wat::core::i64)]
     grant-method-body `(:wat::core::let
                          [~grant-discard-sym (:wat::kernel::send' (~handle-handle-acc h) (~admin-allow-peer-kw pids))
                           ~grant-r-sym       (:wat::kernel::recv' (~handle-handle-acc h))]
                          (:wat::core::match ~grant-r-sym -> :wat::core::nil
                            (~status-peers-allowed-kw nil)
                            (_ (:wat::kernel::assertion-failed!
                                 "defservice grant: expected Status::PeersAllowed"
                                 :wat::core::None
                                 :wat::core::None))))
     grant-method      `(:wat::core::defn ~grant-method-name ~grant-method-params -> :wat::core::nil ~grant-method-body)
     ;; Extend methods with the owner-only grant (stop + hibernate + grant, not per-op).
     methods           (:wat::core::conj methods grant-method)

     ;; ── arc 293: owner-only revoke method (mirror of grant) ──────────────────────
     ;; Method: (defn <fqdn>/revoke [h <- Handle  pids <- (Vector i64)] -> nil ...)
     ;; Takes the Handle (unforgeable; never handed to clients — clients hold only a client
     ;; Peer', so a client has NO revoke path). Sends Admin::DenyPeer[pids] down the lineage
     ;; peer; recv's Status::PeersDenied → the revoke is applied before this returns. Callable
     ;; any time, repeatedly, mid-life. Uses symbol-node for `_`/`r` binders (hygiene: Unquote
     ;; at def time).
     revoke-discard-sym (:wat::core::symbol-node "_")
     revoke-r-sym       (:wat::core::symbol-node "r")
     revoke-method-name (:wat::core::keyword/from-string
                          (:wat::core::string::interpolate "{fqdn-str}/revoke" :fqdn-str fqdn-str))
     revoke-method-params `[h <- ~handle-name  pids <- (:wat::core::Vector :wat::core::i64)]
     revoke-method-body `(:wat::core::let
                           [~revoke-discard-sym (:wat::kernel::send' (~handle-handle-acc h) (~admin-deny-peer-kw pids))
                            ~revoke-r-sym       (:wat::kernel::recv' (~handle-handle-acc h))]
                           (:wat::core::match ~revoke-r-sym -> :wat::core::nil
                             (~status-peers-denied-kw nil)
                             (_ (:wat::kernel::assertion-failed!
                                  "defservice revoke: expected Status::PeersDenied"
                                  :wat::core::None
                                  :wat::core::None))))
     revoke-method      `(:wat::core::defn ~revoke-method-name ~revoke-method-params -> :wat::core::nil ~revoke-method-body)
     ;; Extend methods with the owner-only revoke (stop + hibernate + grant + revoke, not per-op).
     methods           (:wat::core::conj methods revoke-method)

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
     ;; Arc 293 S2 — Op/Reply are the protocol's (proto-str); State/Admin/Status stay per-service.
     launch-head-kw (:wat::core::keyword/from-string
                      (:wat::core::string::concat "wat::spawn::Locus/launch<"
                        (:wat::core::string::concat proto-str
                          (:wat::core::string::concat "::Op,"
                            (:wat::core::string::concat proto-str
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
     own-forms-call  `(:wat::core::forms
                        ~record-def
                        ~state-def
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
     ;; ── Arc 278 S4c: the surface OWNS its protocol; SHIP it. ──────────────────────
     ;; The satisfied surface's `<S>::surface-forms` carrier (emitted by defsurface in Rust) is
     ;; a Vector<WatAST> of the surface's own forms (its :messages records/enums + the defsurface
     ;; that re-synthesizes ::Op/::Reply at the child's fresh startup). Concat it AHEAD of this
     ;; service's own forms so a forked child resolves the protocol its serve loop references.
     ;; proto-str = the surface fqdn (`:satisfies` is mandatory; `:ops` is retired), so the carrier
     ;; name is `<surface>::surface-forms`.
     surface-forms-kw (:wat::core::keyword/from-string
                        (:wat::core::string::concat proto-str "::surface-forms"))
     ;; Arc 278 S4d: concat the OWN surface's forms + every :peers surface's forms + own internals.
     ;; `concat` is strictly binary, so we build a LEFT-nested chain (order-preserving):
     ;;   (concat (concat … (concat (OwnSurface::surface-forms) (S1::surface-forms)) …) own-forms-call)
     ;; peers-forms-node folds each `(:Si::surface-forms)` onto the own-surface call; empty :peers
     ;; → peers-forms-node is just `(OwnSurface::surface-forms)` (identical to the pre-S4d concat).
     peers-forms-node (:wat::core::foldl
                        (:wat::core::fn [acc       <- :wat::WatAST
                                         call-node <- :wat::WatAST]
                          -> :wat::WatAST
                          `(:wat::core::concat ~acc ~call-node))
                        `(~surface-forms-kw)
                        peer-forms-calls)
     service-forms-def `(:wat::core::defn ~service-forms-kw
                          [] -> :wat::core::Vector<wat::WatAST>
                          (:wat::core::concat ~peers-forms-node ~own-forms-call))

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
     handle-record `(:wat::core::defrecord ~handle-name ~handle-fields)]

    ;; Assemble the final `do`:
    ;;   record + state defs (durable/ephemeral projections)
    ;;   Admin + Status enums (the lineage protocol)
    ;;   serve defn (the dispatch loop over the surface's ::Op)
    ;;   methods (per-op type-safe client methods, over the surface protocol)
    ;;   start fn (mints listener + spawns serve → Handle)
    ;;   Handle record (start's return type; emitted last)
    ;;   service-forms def (transport-agnostic fragment; emitted last so all
    ;;     referenced names are already declared in the top-level scope)
    ;; Type-decl forms (records, enums, Handle) splice to top-level via splice_type_decl;
    ;; defns keep the `do` non-empty after type-decl stripping.
    `(:wat::core::do
       ~record-def
       ~state-def
       ~admin-enum-def
       ~status-enum-def
       (:wat::core::defn ~serve-name ~serve-params -> :wat::core::nil ~serve-body)
       ~init-def
       ~stop-project-def
       ~hibernate-project-def
       ~dispatch-admin-def
       ~extract-addr-def
       ~@methods
       ~service-forms-def
       ~start-fn
       ~resume-fn
       ~handle-record)))
