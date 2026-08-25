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

;; ── Outcome :- [S R] — the handler result (the gen_server callback-return model) ──────
;;
;; A handler is a PURE transform `(s <- :State, …in) -> (:Outcome :- [:State <fqdn>::Reply])`.
;; It returns what to DO: reply-and-continue (C.2), and later (C.4) no-reply / stop. This
;; is OTP gen_server's `{reply,R,S} | {noreply,S} | {stop,…}` re-derived as a wat tagged sum
;; (named — NOT a bare `(:Tuple state reply)`; a structured result with distinct roles is a
;; record/sum, per the ADT identity, not an order-convention pair). Generic + stdlib: every
;; service reuses it (not minted per-service). C.4 GROWS it by ADDING variants — no reshape.
;; Arc 278 Stone 2-A (self-scheduling) — GROW to :- [S R O]: a third type param O (the
;; service's concrete Op type — the synthesized `<service>::Op` superset). Only the
;; arm-carrying variants use it (phantom for Reply/Stop/NoReply). A handler schedules a
;; self-message by emitting `Alarm`s: `after` a Duration, deliver `op` (an `<service>::Op`
;; value — armed into the service's own `select'` set as a `(Peer :- [Never O])` timer).
;;   :NoReply       — a cast / a fired self-op with no client to reply to (OTP {noreply,S}).
;;   :ReplyAndArm   — reply to the client AND arm one/more timers.
;;   :NoReplyAndArm — no reply, arm one/more timers (a re-arming heartbeat).
(:wat::core::defrecord :wat::service::Alarm :- [O] [after <- :wat::time::Duration  op <- :O])

(:wat::core::defenum :wat::service::Outcome :- [S R O] :wat::enum::Pure
  :Reply         [state <- :S  reply <- :R]
  :Stop          [state <- :S  reply <- :R]
  :NoReply       [state <- :S]
  :ReplyAndArm   [state <- :S  reply <- :R  arms <- (:wat::core::Vector :- [(:wat::service::Alarm :- [O])])]
  :NoReplyAndArm [state <- :S  arms <- (:wat::core::Vector :- [(:wat::service::Alarm :- [O])])])

;; ── Invocation — the MANDATORY third arm param, `[s ctx req]` (arc 278 the call context) ──
;;
;; ★ SUPERSEDED 2026-08-09 by DESIGN-STONE-mandatory-ctx-and-lifecycle-ops.md: ctx is no longer
;; opt-in (see "THE SHAPE, RULED" + "PRECONDITION (a)" there). Every public arm now takes
;; `[s ctx req]` unconditionally; every internal (`-`) arm takes `[s ctx]` and receives a
;; `SelfInvocation` (see the InvocationCore/SelfInvocation/LifecycleInvocation/Invocation family
;; just below this block, and STOP-0 there — the internal branch used to silently drop ctx). The
;; "OPT-IN"/arity-dispatch history below is kept for the field-level detail that still holds
;; (conn-id/namespace/operation semantics); do not read it as describing current arm shapes.
;;
;; NAME RATIFIED 2026-08-09 (was the placeholder `CallCtx`; STOP-5's owed intueri cast is now
;; PAID). The ward judged `CallCtx` a Level-2 mumble on two axes: `Ctx` fails intueri's own
;; carve-out ("ctx is acceptable when the TYPE speaks" — here `Ctx` IS the type, so the
;; abbreviation stands in for nothing), and the record braids THREE lifetimes, so no
;; field-lifetime name can be honest. `Invocation` names the EVENT instead — one dispatch of one
;; operation — and an invocation legitimately HAS a caller without claiming to BE one.
;; The user-facing binder stays `ctx`, judged separately: it earns its brevity by SCOPE (the
;; fixed middle slot of `[s ctx req]`, a positional role name exactly like `s` and `req`), not by
;; mirroring the type. Type name and binder deliberately do not track each other.
;; Migration recorded: wat-scripts/fixes/rename-call-ctx-to-invocation.wat.
;;
;; The five-field floor, pinned by the builder (DESIGN-STONE-the-call-context.md, "the ctx
;; floor — five fields, all pure scalars"): every field here is a pure scalar (i64/keyword/
;; String/Uuid), so Invocation itself is pure — wire-crossable, `:durable`-legal — even though it
;; is PRODUCED at an impure boundary (the serve loop: a fresh Uuid, a clock read, the live
;; connection table) and only ever CONSUMED by a pure handler. Concrete (no type params): every
;; field's type is the same scalar shape regardless of which service declares the ctx-arm, so
;; this is minted ONCE here, not synthesized per-service the way State/Op/Reply are.
;;   conn-id    — the stable monotonic i64 minted in the serve loop (never reused; STOP-2: it
;;                travels WITH its peer in `selectables`, never a parallel position-keyed vec).
;;                NAMED FOR THE CONNECTION, not the caller: it is not a principal, carries no
;;                authz, and does NOT survive a reconnect — a client that drops and redials gets
;;                a fresh one. Nor is it a POSITION: `idx` is the round-scoped seat number the
;;                transport hands up (crossbeam's registration order; see `poll`), valid only
;;                within the round that issued it. `conn-id` is the name that outlives the round,
;;                which is the whole reason it exists — anything keyed on it (a per-connection
;;                world) survives the vector mutations that shift every `idx`.
;;   namespace  — the service's own fqdn (`fqdn-kw`), a compile-time literal spliced by the macro.
;;   operation  — the op arm's own kebab name (`op-str`), a compile-time literal spliced by the
;;                macro.
;;   invocation-id — one `Uuid/v4`, minted fresh per call in the serve loop (renamed from
;;                   `request-id`, 2026-08-09 intueri cast — see InvocationCore below).
;;   start-ns      — one clock read (`epoch-nanos (now)`), stamped fresh per call in the serve loop.
;; SUPERSEDED (was STOP-3/STOP-6 in the opt-in brief): internal (`-`) ops now DO get a ctx — a
;; `SelfInvocation`, never an `Invocation` (it has no connection, so it has no `conn-id` field to
;; populate — structural, not a runtime `Option`). Lifecycle hooks (`-on-connect`/`-on-disconnect`)
;; are still not built by this strike — `LifecycleInvocation` is declared and unused on purpose.
;; ── InvocationCore — the four facts EVERY invocation has, spliced into all three ─────────
;;
;; There are THREE kinds of invocation and their field sets are a strict NESTING, so the shared
;; head is a surface spliced with `~@` (the shipped idiom — `Scope` into Metric/Log,
;; wat/telemetry.wat:84). A telemetry consumer that wants "any invocation" reads THIS and does not
;; care which kind it got.
;;
;; ★ `invocation-id`, NOT `request-id` (intueri, 2026-08-09). The old name was honest while this
;; record was singular and always per-call. The moment the core is shared with SelfInvocation and
;; LifecycleInvocation — two records DEFINED by having no request — a field called `request-id`
;; asserts something false for two of the three kinds. The structure made a good name wrong; the
;; name follows the structure.
(:wat::core::defsurface :wat::service::InvocationCore
  :nature :wat::core::Record
  :features [namespace     <- :wat::core::keyword   ;; the service's own fqdn — compile-time literal
             operation     <- :wat::core::String    ;; the op arm's own name — compile-time literal
             invocation-id <- :wat::core::Uuid      ;; minted by THIS service, per dispatch
             start-ns      <- :wat::core::i64])     ;; clock read, per dispatch

;; SELF-originated: a self-scheduled alarm fired. No connection, no caller, no request — the
;; service asked for this itself. It still gets a ctx because it is still an INVOCATION: a thing
;; the service DID, at a time, which must be visible to telemetry exactly like a client call.
(:wat::core::defrecord :wat::service::SelfInvocation
  [~@:wat::service::InvocationCore])

;; CONNECTION-LIFECYCLE-originated: `-on-connect` / `-on-disconnect`. The connection's STATE
;; changed; nobody sent a message. Named for the lifecycle, NOT the connection — a plain
;; `Invocation` is *also* connection-originated and also carries a conn-id, so `ConnectionInvocation`
;; (the ward's pick) could not distinguish the two. What separates this is that it is about the
;; connection changing rather than a call OVER it.
(:wat::core::defrecord :wat::service::LifecycleInvocation
  [~@:wat::service::InvocationCore
   conn-id <- :wat::core::i64])

;; A CLIENT CALL. Splice-first field order per the arc-293 house rule (wat/telemetry.wat:82).
;;
;; ⚠ OWED, and deliberately absent: `caller-invocation-id` — the id of the invocation on the
;; CALLER's side that caused this one, used ONLY to draw an edge (never keyed on, never trusted,
;; never adopted; this service mints `invocation-id` regardless, so a hostile value costs at most a
;; false edge in the client's own trace). It is MANDATORY by design — our generated client always
;; sends one, so a request lacking it is malformed, not legitimately absent — which is exactly why
;; it CANNOT be declared until the client-side mechanism that populates it exists. A mandatory field
;; with nothing to fill it is a lie from birth. It lands with that strike and costs nothing here:
;; adding a FIELD does not touch an arm's binders. NOT `parent-id` — that name is taken, 44 sites in
;; `wat/rete.wat`, meaning a join-node's tree parent (an i64, not a Uuid).
(:wat::core::defrecord :wat::service::Invocation
  [~@:wat::service::InvocationCore
   conn-id <- :wat::core::i64])

;; ── Capability — the uniform capability surface (arc 170 capability circuit) ──────
;;
;; RELOCATED (stone 2) + RENAMED (stone A, was Grantable): the :wat::capability::Capability
;; surface now lives in wat/capability.wat, which loads EARLY (before spawn.wat/bracket.wat,
;; both of which name it). The defservice macro's auto-emitted extend-type (grantable-extend,
;; below) routes each <fqdn>::Handle to :wat::capability::Capability.

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
   & clauses <- (:wat::core::Vector :- [:wat::WatAST])]  ;; all-kwargs: [:durable [..] :ephemeral [..] :ops [..] ...]
  -> :wat::WatAST
  ;; PROGRAM-BODY path: top-level `let`, params are node-values, nested quasiquote at the end.
  (:wat::core::let
    [fqdn-str      (:wat::core::keyword/to-string fqdn)
     ;; Arc 265 — reconstruct fqdn as a keyword value so pascal->kebab-in
     ;; can use it as the namespace for acronym-registry lookup.
     fqdn-kw       (:wat::core::keyword/from-string fqdn-str)

     ;; ── Arc 278 parametric defservice: the name / type-param SPLIT ─────────────
     ;; A service fqdn MAY carry type params (`:my::box-svc<T>`). Every companion name
     ;; this macro mints is `<base><suffix>` — the suffix appends to the BASE and the
     ;; params RE-ATTACH at the end: `:my::box-svc::Record<T>`, NEVER the naive
     ;; `:my::box-svc<T>::Record` (a genuinely malformed type name — the parser is right
     ;; to reject it; the macro was building it).
     ;;
     ;; THE IDENTITY PROPERTY (load-bearing — the nine concrete services ride on it):
     ;; with no type params `fqdn-tp-syms` is empty and `fqdn-base` IS `fqdn-str`, so
     ;; every derived name is byte-identical to the pre-split concatenation.
     ;;
     ;; WHICH companion carries the params is decided per-name by whether a type param
     ;; can REACH it, not blanket-applied:
     ;;   ::Record / ::State / ::Admin / ::Status / ::Handle  → PARAMETRIC (the durable
     ;;     fields may name `:T`, and State/Admin/Status/Handle each carry the Record).
     ;;   ::Op (the service superset) → CONCRETE: its variants wrap the SURFACE's
     ;;     `<Op>Request` records (:messages stay concrete) and internal ops are nullary,
     ;;     so no type param can reach it.
     ;;   defn DECLARATIONS whose signature names a parametric companion → PARAMETRIC.
     ;;     `::service-forms` ([] -> (Vector :- [WatAST])) and the per-op CLIENT methods
     ;;     (`[c <- (Peer :- [S::Op S::Reply])  req <- S::<Op>Request] -> (RecvOutcome :- […])`)
     ;;     name no companion → concrete.
     ;;   CONSTRUCTOR / ACCESSOR / VARIANT / runtime-name-string keywords (`::State'`,
     ;;     `::State/durable`, `::Admin::Init`, `"<fqdn>::serve"`) → BASE, no params:
     ;;     construction and by-name runtime resolution both key on the bare name
     ;;     (`split_name_and_type_params` registers a generic defn under its base).
     ;; ── Arc 109 β-ii-a′: THE BINDER, peeled BEFORE the clause fold ─────────────
     ;; A service may now declare its type params as a real binder, `:- [K V]`, arriving
     ;; as the FIRST two elements of `clauses` (a `:-` keyword node, then a WatAST::Vector
     ;; of symbol nodes). It MUST be peeled here — before the clause fold below (:290-ish)
     ;; — because that fold calls `macro-error` on any key not in `known-clauses` and would
     ;; reject `:-` as unrecognized (DESIGN-STONE-binder-beta-ii.md, "hazards measured").
     ;;
     ;; THE CONTRACT: the binder is the SOURCE OF TRUTH; `fqdn-tp-syms` below is DERIVED
     ;; from it, never the reverse — two independent derivations off the name could
     ;; disagree, and the disagreement would surface ~50 sites from the cause.
     ;; STONE-the-last-mint — the bracketed `<K,V>` STRING derivation (`fqdn-tp`) that
     ;; used to sit alongside `fqdn-tp-syms` is RETIRED; the syms list is the only
     ;; derivation left.
     binder-present? (:wat::core::if (:wat::core::empty? clauses)

                       false
                       (:wat::core::= "-"
                         (:wat::core::keyword/to-string (:wat::core::first clauses))))
     ;; `clauses-body` is `clauses` with the `:-` pair stripped when present, unchanged
     ;; otherwise. Every downstream reader of the rest-arg (the even-length guard, the
     ;; clause-map fold) reads `clauses-body` — never the raw `clauses` param — from here on.
     clauses-body   (:wat::core::if binder-present?

                      (:wat::core::rest (:wat::core::rest clauses))
                      clauses)
     ;; DERIVED — the name with any `<…>` stripped. Uniform: it does not matter whether the
     ;; params arrived via the binder (no brackets in the name) or the legacy suffix.
     fqdn-base     (:wat::core::if (:wat::string::ends-with? fqdn-str ">")

                     (:wat::core::first (:wat::string::split fqdn-str "<"))
                     fqdn-str)
     ;; ★ ONE SOURCE OF TRUTH: the param SYMBOL list. Everything else is derived from it.
     ;;
     ;; Builder, 2026-08-21: *"let's make the `:- []` assumed default state."* So there is no
     ;; monomorphic-vs-parametric distinction to track — there is a param list, and it is
     ;; usually empty. A declaration with no binder, a declaration written `:- []`, and a bare
     ;; name are the SAME state, reached by the same path. `fqdn-parametric?` survives only as
     ;; a DERIVED convenience for its 7 readers (β-ii-b retires them); it is not a concept.
     ;;
     ;; Dual-read on where the params come from — binder first, then the legacy name suffix:
     ;;   `:- [K V]`        → ast->children of the binder vector.
     ;;   `lru-svc<K,V>`    → parsed out of the name (③ retires this branch).
     ;;   neither / `:- []` → empty.
     fqdn-tp-syms   (:wat::core::if binder-present?

                      (:wat::core::if (:wat::core::empty? (:wat::core::rest clauses))

                        (:wat::core::macro-error
                          "defservice: :- binder must be followed by a parameter vector, e.g. :- [K V]")
                        (:wat::core::ast->children
                          (:wat::core::first (:wat::core::rest clauses))))
                      (:wat::core::if (:wat::string::ends-with? fqdn-str ">")

                        ;; Legacy name spelling: strip the brackets by INDEX (never split on
                        ;; "<" — that leaves a leading "" and yields an empty-named symbol),
                        ;; split on ",", trim, symbolize. `foldl`+`conj` rather than `mapv`:
                        ;; a program-body macro may not pass a bare primitive keyword as a
                        ;; value (measured: `expected "wat::core::fn"`) — the same shape
                        ;; `:wat::core::keyword/of` used before its retirement
                        ;; (STONE-defservice-emits-the-binder, arc 109; see `wat/core.wat`).
                        (:wat::core::foldl
                          (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                           nm  <- :wat::core::String]
                            -> (:wat::core::Vector :- [:wat::WatAST])
                            (:wat::core::conj acc
                              (:wat::core::symbol-node (:wat::string::trim nm))))
                          (:wat::core::Vector :wat::WatAST)
                          (:wat::string::split
                            ;; The names lie between the brackets. `fqdn-base` is the name up to
                            ;; "<", so its LENGTH is the "<" index — no `string::index-of`, which
                            ;; the F5 pure-combinator allow-list refuses (measured: refused AT
                            ;; DEFINITION, taking the whole stdlib down). This is the same
                            ;; arithmetic the original suffix derivation used.
                            (:wat::string::subs fqdn-str
                              (:wat::core::i64::+ (:wat::string::length fqdn-base) 1)
                              (:wat::core::i64::- (:wat::string::length fqdn-str) 1))
                            ","))
                        (:wat::core::Vector :wat::WatAST)))
     ;; DERIVED — "has params", not "a binder was written". An empty binder (`:- []`) is the
     ;; same first-class empty rung as `(Tuple :- [])` and lands where a bare name lands.
     ;; Measured when this branched on binder-present? and answered plain `true`: `:- []`
     ;; crashed the macro in `string::subs` — "index out of range: start=0, end=-1,
     ;; char-length=0" — because a downstream reader trusted the flag and did bracket
     ;; arithmetic on "".
     fqdn-parametric? (:wat::core::if (:wat::core::empty? fqdn-tp-syms) false true)
     ;; STONE-the-last-mint — `fqdn-tp`, the bracketed `<K,V>` suffix STRING compatibility
     ;; shim, is RETIRED (its last two consumers, `transport-param` and `method-name`
     ;; below, now read `fqdn-tp-syms` structurally); mirrors `proto-tp`'s retirement
     ;; (commit c6c614fe2). `fqdn-tp-syms` (the param SYMBOL list, above) is the one
     ;; source of truth from here on.

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
                      ;; Arc 278 Stone 1: :max-frame-bytes — the per-service hard frame limit `FOO`
                      ;; (bytes-per-read), threaded to the accepted-connection receivers. Optional;
                      ;; default DEFAULT_MAX_FRAME_BYTES (512 KiB).
                      ;; ⚠ Arc 278 Stone 2 — there is NO clause here for opting into request
                      ;; sanitization, and there never will be. Stone 1 shipped one (`:all | :none`,
                      ;; defaulting to `:none`) to stage the corpus rollout; the builder annihilated
                      ;; it on sight: "who would opt into crashing on bad input — why would this ever
                      ;; be an option to consider." A knob whose off-position is "die on a malformed
                      ;; frame, for every client at once" is a non-option surfaced as a choice. Input
                      ;; sanitization is not something a service opts into — it is what a service IS.
                      ;; See serve-op-arms below: the request-SHAPE guard is generated for EVERY op
                      ;; of EVERY service, always.
                      "max-frame-bytes" true)
     clauses-len    (:wat::core::length clauses-body)
     n-clause-pairs (:wat::core::i64::/ clauses-len 2)
     ;; even-length guard
     _clauses-even  (:wat::core::if
                      (:wat::core::= (:wat::core::i64::* n-clause-pairs 2) clauses-len)
                      
                      nil
                      (:wat::core::macro-error
                        "defservice: clauses must be :keyword value pairs"))
     ;; build + validate in one pass
     clause-map     (:wat::core::foldl
                      (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::String :wat::WatAST])
                                       i <- :wat::core::i64]
                        -> (:wat::core::HashMap :- [:wat::core::String :wat::WatAST])
                        (:wat::core::let
                          [k   (:wat::core::i64::* i 2)
                           key (:wat::core::keyword/to-string
                                 (:wat::core::Option/expect
                                   (:wat::core::get clauses-body k) "defservice: malformed clause key"))]
                          (:wat::core::if (:wat::core::HashMap/contains-key? known-clauses key)

                            (:wat::core::HashMap/assoc m key
                              (:wat::core::Option/expect
                                (:wat::core::get clauses-body (:wat::core::i64::+ k 1))
                                "defservice: clause missing a value"))
                            (:wat::core::macro-error
                              (:wat::string::concat "defservice: unknown clause :"
                                (:wat::string::concat key
                                  " — recognized clauses: :durable :ephemeral :ops :init :hibernate :stop :durable-parent :satisfies :impls :peers :max-frame-bytes"))))))
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
                      
                      (:wat::core::macro-error "defservice: :ops is RETIRED — declare a surface (defsurface :nature :wat::kernel::Peer with method members + per-op Request record and <Op>Response) and :satisfies it with :impls (bodies only). Exemplar: wat/query.wat + wat/query/mem.wat.")
                      nil)
     _needs-surface (:wat::core::if satisfies?
                      
                      nil
                      (:wat::core::macro-error "defservice: :satisfies is required — name the surface this service wears (see wat/query.wat for the exemplar)."))
     _needs-impls   (:wat::core::if has-impls?
                      
                      nil
                      (:wat::core::macro-error "defservice: :satisfies requires :impls (the op bodies, bodies-only)."))
     ;; ops := the op-bearing clause value — :impls when satisfies, else :ops.
     ops            (:wat::core::if satisfies?
                      
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "impls")
                        "defservice: :impls clause missing value")
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "ops")
                        "defservice: :ops clause missing value"))
     ;; The protocol namespace: the surface's when :satisfies (its S1 ::Op/::Reply +
     ;; user-declared request/response records), else the service's own fqdn.
     surface-node   (:wat::core::if satisfies?
                      
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "satisfies")
                        "defservice: :satisfies needs a surface")
                      fqdn)
     ;; BRIEF-STONE-defservice-compares-types-as-data.md — `surface-node` may be a Keyword
     ;; (`:S<K,V>`) or already a form (`(S :- [K V])`); `keyword/to-type-form-colon` takes a
     ;; Keyword only, so a List is branched on ast-kind and used as-is.
     surface-form   (:wat::core::if (:wat::core::= (:wat::core::ast-kind surface-node) "keyword")

                      (:wat::core::keyword/to-type-form-colon surface-node)
                      surface-node)
     ;; ── Arc 278 THE PARAMETRIC PROTOCOL: the surface's base / type-ARG split ────────────
     ;; `:satisfies :S<K,V>` is the CHANNEL. The suffix is written at the satisfies site in the
     ;; SERVICE's own binders, so re-attaching it to a protocol-namespaced name yields exactly
     ;; the instantiation this service wears (`:S::Op<K,V>`, `:S::GetRequest<K,V>`). Same split
     ;; helper as `fqdn-base`/`fqdn-tp-syms` above — one spelling, two sides.
     ;;
     ;; `proto-base` (params STRIPPED) is the identity a NAME keys on: the acronym registry, the
     ;; runtime `retag-op'` discriminator, every ctor/accessor/variant keyword, and `derive`
     ;; (subtype edges are registered between BASE names). `proto-args` (the raw arg-node list)
     ;; re-attaches in TYPE positions only — STONE-defservice-emits-the-binder retired the `<…>`
     ;; suffix-string mint (`proto-tp`/`proto-args-str`) in favour of minting the reference FORM
     ;; `(Head :- [~@proto-args])` directly; `proto-args` empty ⇒ `(Head :- [])` ≡ `Head`.
     ;;
     ;; THE IDENTITY PROPERTY: a monomorphic surface has `proto-args` = `[]` and `proto-base` IS
     ;; `keyword/to-string surface-node`, so every derived reference is identical to the
     ;; pre-split name.
     ;;
     ;; ⛔ RETIRED 2026-08-21 — "THE MESSAGE CONVENTION" said a parametric surface's `:messages` are
     ;; parametric in ALL of the surface's params "even when a given message uses only some (or none)
     ;; of them", and called itself checker-locked in `synthesize_surface_protocol`. BOTH HALVES WERE
     ;; FALSE. Nothing locked it: `wat/cache.wat:169` declares `(Cache :- [K V])` and `:171` declares
     ;; `(Cache::GetRequest :- [K])` — one param, not all — and the stdlib loads and passes.
     ;;
     ;; THE RULE, builder 2026-08-21: a surface declares its own params; each MESSAGE declares only
     ;; what IT consumes, and the surface's list is the union rather than a quota each must restate.
     ;; `cache.wat` already reads exactly that way and is the exemplar:
     ;;   (Cache :- [K V])  ⇒  (Cache::GetRequest :- [K])   (keys)      (Cache::GetResult :- [V])   (values)
     ;;                        (Cache::GetResponse :- [V])  (values)    (Cache::PutRequest :- [K V]) ((Entry :- [K V]), both)
     ;; Enforced by the param-spec consumption wall: a declared param no member type mentions is
     ;; illegal. `[[feedback_a_comment_can_ship_a_gap_as_a_law]]` — this comment asserted a lock
     ;; that did not exist, and the corpus it cited as conformant was the deviation.
     ;; It is what gives the derivation below a representation for a message's type arguments —
     ;; the surface's own params — where before there was none, and it is what keeps the
     ;; surface's `<S>::Op` and this service's `<fqdn>::Op` superset field-for-field identical
     ;; (the `derive` edge and `retag-op'` both require that).
     ;; STOP-3 fix (an earlier stone): the old derivation rendered `surface-node` to a string and
     ;; split on `"<"`/`","` — dead the moment `surface-node` is a form. Read head/args
     ;; structurally instead: `surface-form` is a bare Keyword (non-parametric) or a List
     ;; `(Head :- [args])`; `proto-args` is that args vector's own children, spliced directly
     ;; into every downstream reference-FORM mint (`~@proto-args`) — never re-serialized to text.
     proto-parametric? (:wat::core::= (:wat::core::ast-kind surface-form) "list")
     proto-base     (:wat::core::if proto-parametric?

                      (:wat::core::keyword/to-string
                        (:wat::core::first (:wat::core::ast->children surface-form)))
                      (:wat::core::keyword/to-string surface-form))
     proto-args     (:wat::core::if proto-parametric?

                      (:wat::core::ast->children
                        (:wat::core::nth (:wat::core::ast->children surface-form) 2))
                      (:wat::core::Vector :wat::WatAST))
     ;; A type ARG may be a concrete FQDN (renders as a Keyword — `keyword/to-string` strips its
     ;; leading colon) or a bare type-VARIABLE (`K`/`V`/`T` — renders as a Symbol, never
     ;; colon-spelled at all; `ast-name` reads it as-is). `(Cache :- [K V])` exercises exactly this:
     ;; both args are type-vars, not FQDNs.
     ;;
     ;; STONE-exactly-one-call-position — `proto-args-str`/`proto-tp` (the `<a,b,…>`
     ;; angle-string mint) are RETIRED. Their one remaining consumer was `launch-head-kw`
     ;; below (`Locus/launch`'s NAME-embedded type-arg suffix); it now peels a position-4
     ;; `:-` binder from `args` exactly as the generic call arm does (the hoist that fixed
     ;; this taught BOTH arms at once), so the head is bare and the type args ride as
     ;; call-site siblings — nothing left anywhere in this file mints an angle name.
     surface-kw     (:wat::core::keyword/from-string proto-base)

     ;; :durable [fields] — optional, default empty vector node []
     ;; The empty vector node is built by using with-children on a fresh Vector.
     ;; We need a Vector WatAST node; use the ops node as a shape carrier with empty children.
     empty-vec      (:wat::core::with-children ops (:wat::core::Vector :wat::WatAST))
     durable-fields (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "durable")
                      
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "durable")
                        "defservice: :durable needs a value")
                      empty-vec)

     ;; :ephemeral [fields] — optional, default empty vector node []
     ephemeral-fields (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "ephemeral")
                        
                        (:wat::core::Option/expect
                          (:wat::core::HashMap/get clause-map "ephemeral")
                          "defservice: :ephemeral needs a value")
                        empty-vec)
     ;; ── THE SHAPE WALL: :durable / :ephemeral take a FIELD VECTOR ────────────────
     ;; Both clause values flow straight into `(defrecord ~record-ty ~durable-fields)` /
     ;; the ::State defstruct. A NON-vector (e.g. a bare type keyword `:durable
     ;; :wat::core::i64`) is UNEXPRESSIBLE: a service's soul is a set of named fields, not
     ;; a scalar. Unwalled, the bad shape flowed into the emitted decl and was tolerated
     ;; downstream instead of screaming here, at the site the author wrote.
     _durable-shape   (:wat::core::if (:wat::core::= (:wat::core::ast-kind durable-fields) "vector")
                        
                        nil
                        (:wat::core::macro-error
                          "defservice: :durable takes a FIELD VECTOR [name <- :Type …] — a bare type keyword / scalar durable is unexpressible; the durable IS the soul: a set of named fields that crosses the wire and survives hibernation"))
     _ephemeral-shape (:wat::core::if (:wat::core::= (:wat::core::ast-kind ephemeral-fields) "vector")
                        
                        nil
                        (:wat::core::macro-error
                          "defservice: :ephemeral takes a FIELD VECTOR [name <- :Type …] — a bare type keyword / scalar ephemeral is unexpressible"))

     ;; Is ephemeral non-empty? (child count > 0)
     ephemeral-len  (:wat::core::length (:wat::core::ast->children ephemeral-fields))
     has-ephemeral  (:wat::core::i64::> ephemeral-len 0)

     ;; :durable-parent — optional, default :wat::core::Record. The user-supplied branch is
     ;; already a `:wat::WatAST` node (every clause value in this macro is); the default must
     ;; be minted as one too — `type-equal?` (below) requires a node on both sides, unlike
     ;; `keyword/to-string`'s old two-representation leniency.
     state-parent   (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "durable-parent")

                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "durable-parent")
                        "defservice: :durable-parent needs a value")
                      (:wat::core::keyword-node ":wat::core::Record"))

     ;; ── Arc 278 Stone 1: :max-frame-bytes — the per-service hard frame limit `FOO` ──
     ;; Optional; default DEFAULT_MAX_FRAME_BYTES (512 KiB = 524288). The declared value
     ;; (a bare i64 literal node) is threaded into the process child-main's `listener'`
     ;; call as the 4th arg, so the accepted-connection receivers read client requests at
     ;; this budget. A frame over it → RecvError::FrameTooLarge → ServiceEvent::Lost (a
     ;; reasoned close), never a mute clean-hangup. Thread tier has no byte frames → no-op.
     max-frame-bytes-node (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "max-frame-bytes")
                            
                            (:wat::core::Option/expect
                              (:wat::core::HashMap/get clause-map "max-frame-bytes")
                              "defservice: :max-frame-bytes needs a value")
                            `524288)

     ;; ── Arc 109 β-ii-c: per-type param CONSUMPTION — a generated companion carries only
     ;; the params ITS OWN field/member vector actually mentions, not the service's full
     ;; declared list (`fqdn-tp-syms`). `type-params-used-in` (arc 109 β-ii-c intrinsic)
     ;; answers "which of `fqdn-tp-syms` appear anywhere in this AST?"; each "-tp-syms"
     ;; below is that SUBSET, in the same order. STONE-the-last-mint — `fqdn-tp` (and the
     ;; sibling "-tp" bracket-suffix STRING renders this comment used to describe) is
     ;; RETIRED; every companion now splices its `-tp-syms` subset structurally as
     ;; DECLARATION siblings / a `(Head :- [args])` reference FORM, never a re-rendered
     ;; bracket string. An empty subset ⇒ no binder / `(Head :- [])` ≡ `Head` — the
     ;; byte-identity property, one level down.
     ;;
     ;; Record's own field vector is `durable-fields` verbatim (no union needed).
     record-tp-syms (:wat::core::type-params-used-in fqdn-tp-syms durable-fields)
     ;; State wraps the durable ref + its own ephemeral fields — search the union of the
     ;; RAW `durable-fields`/`ephemeral-fields` (not the derived `record-ty`, so this has
     ;; no ordering dependency on it) built via the same ast->children/with-children idiom
     ;; the REAL `state-field-vec` below uses. This is a separate, EARLIER search node —
     ;; the real one is not assembled until after `record-ty` (it prepends `record-ty`
     ;; itself, downstream); searching the raw durable field types directly finds the same
     ;; params, since whatever reaches `record-ty`'s own text came from `durable-fields`.
     state-search-items (:wat::core::foldl
                           (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                            item <- :wat::WatAST]
                             -> (:wat::core::Vector :- [:wat::WatAST])
                             (:wat::core::conj acc item))
                           (:wat::core::ast->children durable-fields)
                           (:wat::core::ast->children ephemeral-fields))
     state-search-node (:wat::core::with-children empty-vec state-search-items)
     state-tp-syms  (:wat::core::type-params-used-in fqdn-tp-syms state-search-node)

     ;; ── 4b-ii: mint state-ty as :<fqdn>::State, record-ty as :<fqdn>::Record ──
     ;; Arc 109 ③ — angle brackets are ILLEGAL for types (both parse doors wall them); the
     ;; `<a,b>` suffix-string mint (`record-tp`/`state-tp`, above) is RETIRED. `*-ty-str`/
     ;; `*-ty-decl` now carry the BARE name only — no `<…>` — and `state-def`/`record-def`
     ;; (below) splice `:- [~@*-tp-syms]` as DECLARATION siblings when the syms list is
     ;; non-empty. The ANNOTATION spelling mints the reference FORM `(Head :- [args])`
     ;; directly via quasiquote — no round-trip through an angle-bracket string and the
     ;; (now-illegal) type parser. identity 2c's role split (DECL-NAME vs ANNOTATION) still
     ;; holds; only the SPELLING each role emits has changed.
     state-ty-str   (:wat::string::interpolate "{b}::State" :b fqdn-base)
     state-ty-decl  (:wat::core::keyword/from-string state-ty-str)
     state-ty-base-kw (:wat::core::keyword-node (:wat::string::concat ":" state-ty-str))
     state-ty-ann   (:wat::core::if (:wat::core::empty? state-tp-syms)
                      state-ty-base-kw
                      `(~state-ty-base-kw :- [~@state-tp-syms]))
     record-ty-str  (:wat::string::interpolate "{b}::Record" :b fqdn-base)
     record-ty-decl (:wat::core::keyword/from-string record-ty-str)
     ;;
     ;; ✅ 2c's STOP-2 IS CLOSED (BRIEF-STONE-defservice-compares-types-as-data). It recorded that
     ;; the `:hibernate` check below consumed this bare keyword through `keyword/to-string` as a
     ;; type-identity STRING compare, fitting none of the four roles. That was the missing DOOR,
     ;; not a missing role: the check now reads `type-equal? hib-ret-ty record-ty-ann` — two type
     ;; NODES, spelling-agnostic — so nothing renders a type to compare it any more.
     record-ty-base-kw (:wat::core::keyword-node (:wat::string::concat ":" record-ty-str))
     record-ty-ann  (:wat::core::if (:wat::core::empty? record-tp-syms)
                      record-ty-base-kw
                      `(~record-ty-base-kw :- [~@record-tp-syms]))

     ;; ── 4b-ii: :init option ────────────────────────────────────────────────────
     ;; :init : Record → State. Default (fn [d <- ::Record] -> ::State (::State d))
     ;;   when :ephemeral is empty. When :ephemeral non-empty and :init absent → macro-error.
     ;; A synthetic symbol-node "record" for the default init param (hygiene: Unquote at def time).
     ;; arc 291 kwargs-start: renamed "d"→"record" so the default-init start kwarg is :record.
     d-sym          (:wat::core::symbol-node "record")
     s-sym          (:wat::core::symbol-node "s")
     ;; state-new-kw: :<fqdn>::State' — the PRIME positional ctor (arc 294 item 9a: the bare
     ;; `:<fqdn>::State` is now the kwargs UX macro; generated machinery constructs via the prime,
     ;; exactly as kwargs-lower does for its `::Kwargs` bundle).
     state-new-kw   (:wat::core::keyword/from-string
                      (:wat::string::interpolate "{b}::State'" :b fqdn-base))
     ;; init-fn-node: user-provided fn, or default, or macro-error
     init-fn-node   (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "init")
                      
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "init")
                        "defservice: :init needs a value")
                      (:wat::core::if has-ephemeral
                        
                        (:wat::core::macro-error
                          (:wat::string::interpolate "{fqdn-str}: :ephemeral declares fields but no :init — the macro cannot construct ephemeral fields; provide :init : Record → State" :fqdn-str fqdn-str))
                        `(:wat::core::fn [~d-sym <- ~record-ty-ann] -> ~state-ty-ann (~state-new-kw ~d-sym))))
     ;; Extract the param vector children [name <- :T] from the init fn node
     ;; init-fn-node structure: (fn [params] -> :RetTy body) → ast->children = [fn,params,->,:RetTy,body]
     init-fn-ch     (:wat::core::ast->children init-fn-node)
     init-params-vec (:wat::core::nth init-fn-ch 1)
     init-body      (:wat::core::nth init-fn-ch 4)
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
                      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                       i <- :wat::core::i64]
                        -> (:wat::core::Vector :- [:wat::WatAST])
                        (:wat::core::conj acc
                          (:wat::core::Option/expect
                            (:wat::core::get init-param (:wat::core::i64::* i 3))
                            "defservice: init param name out of bounds")))
                      (:wat::core::Vector :wat::WatAST)
                      (:wat::core::range 0 (:wat::core::i64::/ (:wat::core::length init-param) 3)))
     ;; init-name: :<fqdn>::init — the emitted defn's name keyword
     init-name-str  (:wat::string::interpolate "{b}::init" :b fqdn-base)
     init-name      (:wat::core::keyword/from-string init-name-str)
     ;; init-def: the emitted top-level defn for init
     init-def       `(:wat::core::defn ~init-name ~init-params-vec -> ~state-ty-ann ~init-body)

     ;; ── 4b-ii: :stop option — projection hook ────────────────────────────────
     ;; Default: (fn [s <- ::State] -> ::Record (::State/durable s))
     ;; User-provided :stop keeps its own declared resp-ty (any EDN-portable type).
     state-durable-kw (:wat::core::keyword/from-string
                        (:wat::string::interpolate "{b}::State/durable" :b fqdn-base))
     stop-fn-node   (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "stop")
                      
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "stop")
                        "defservice: :stop needs a value")
                      `(:wat::core::fn [~s-sym <- ~state-ty-ann] -> ~record-ty-ann (~state-durable-kw ~s-sym)))
     stop-fn-ch     (:wat::core::ast->children stop-fn-node)
     stop-params-vec (:wat::core::nth stop-fn-ch 1)
     ;; resp-ty: index 3 = the :RetTy node in [fn, params, ->, :RetTy, body]
     resp-ty        (:wat::core::nth stop-fn-ch 3)
     stop-body      (:wat::core::nth stop-fn-ch 4)
     ;; stop-project-name: :<fqdn>::stop-project (distinct from <fqdn>/stop method)
     stop-project-name-str (:wat::string::interpolate "{b}::stop-project" :b fqdn-base)
     stop-project-name (:wat::core::keyword/from-string stop-project-name-str)
     ;; stop-project-def: the emitted top-level defn for stop projection
     stop-project-def `(:wat::core::defn ~stop-project-name ~stop-params-vec -> ~resp-ty ~stop-body)

     ;; ── 4b-ii: :hibernate option — projection hook (NEW, mirror of :stop) ────
     ;; Return type FORCED to ::Record (resume = :init consumes it).
     ;; Default: (fn [s <- ::State] -> ::Record (::State/durable s))
     ;; User-provided :hibernate: if it declares a different return type → macro-error.
     hibernate-fn-node (:wat::core::if (:wat::core::HashMap/contains-key? clause-map "hibernate")
                         
                         (:wat::core::Option/expect
                           (:wat::core::HashMap/get clause-map "hibernate")
                           "defservice: :hibernate needs a value")
                         `(:wat::core::fn [~s-sym <- ~state-ty-ann] -> ~record-ty-ann (~state-durable-kw ~s-sym)))
     hibernate-fn-ch  (:wat::core::ast->children hibernate-fn-node)
     hibernate-params-vec (:wat::core::nth hibernate-fn-ch 1)
     ;; hib-ret-ty: the declared return type of the hibernate fn
     hib-ret-ty       (:wat::core::nth hibernate-fn-ch 3)
     hibernate-body   (:wat::core::nth hibernate-fn-ch 4)
     ;; Force the return type to ::Record — if user declared something else, macro-error.
     ;;
     ;; ⚠ GUARDED on the SAME predicate that chose the branch above. When the user supplies no
     ;; `:hibernate`, `hibernate-fn-node` is the DEFAULT this macro just emitted, whose return
     ;; slot IS `record-ty-ann` — so the check would be comparing the macro's own output against
     ;; the macro's own value. Vacuous by construction; kept anyway as documentation of intent
     ;; and because it costs nothing to skip the (also-vacuous) comparison in that branch.
     ;;
     ;; BRIEF-STONE-defservice-compares-types-as-data.md — this used to be `keyword/to-string`
     ;; on both sides, RAISING the moment either side became a form (`(Head :- [args])`) rather
     ;; than a bare keyword (identity 2c: six `service-parametric-*` tests went red here, every
     ;; one a service declaring no `:hibernate`). `type-equal?` reads both sides AS TYPES —
     ;; spelling-agnostic — so a user who writes `-> (::Record :- [K V])` after ②-iii now
     ;; compares correctly too; this closes the caveat the old comment recorded here.
     hib-user-supplied? (:wat::core::HashMap/contains-key? clause-map "hibernate")
     _hib-ty-check    (:wat::core::if hib-user-supplied?

                        (:wat::core::if (:wat::core::type-equal? hib-ret-ty record-ty-ann)

                          nil
                          (:wat::core::macro-error
                            (:wat::string::interpolate "{fqdn-str}: :hibernate return type must be ::Record (the resume seed); declared a different type" :fqdn-str fqdn-str)))
                        nil)
     hibernate-project-name-str (:wat::string::interpolate "{b}::hibernate-project" :b fqdn-base)
     hibernate-project-name (:wat::core::keyword/from-string hibernate-project-name-str)
     hibernate-project-def `(:wat::core::defn ~hibernate-project-name ~hibernate-params-vec -> ~record-ty-ann ~hibernate-body)

     ;; ── 4b-ii: emit the Record def + State defstruct ─────────────────────────
     ;; record-def: (:wat::core::Record::def ::Record [durable-fields]) (or holon parent)
     ;; state-def:  (:wat::core::defstruct ::State [durable <- ::Record <ephemeral-fields...>])
     ;;   The 3 tokens `durable <- ~record-ty` are prepended to ephemeral children.
     ;; BRIEF-STONE-defservice-compares-types-as-data.md — EQUALITY against a literal, via the
     ;; door instead of a render+compare; `state-parent` is user-declared (`:durable-parent`
     ;; clause, defaulted to `:wat::core::Record`) and, like any declared type, could in
     ;; principle arrive as a form rather than a bare keyword.
     ;; Arc 109 ③ — `record-ty-decl` is now the BARE name (no `<…>`); a non-empty
     ;; `record-tp-syms` splices the `:- [T …]` binder as DECLARATION SIBLINGS
     ;; (no parens — the corpus spelling, e.g. `wat/cache.wat:71`'s
     ;; `(:wat::core::defrecord :wat::cache::Entry :- [K V] ...)`), immediately
     ;; after the name, before the field vector.
     record-def   (:wat::core::if (:wat::core::type-equal? state-parent (:wat::core::keyword-node ":wat::holon::Record"))

                    (:wat::core::if (:wat::core::empty? record-tp-syms)
                      `(:wat::holon::defrecord ~record-ty-decl ~durable-fields)
                      `(:wat::holon::defrecord ~record-ty-decl :- [~@record-tp-syms] ~durable-fields))
                    (:wat::core::if (:wat::core::empty? record-tp-syms)
                      `(:wat::core::defrecord ~record-ty-decl ~durable-fields)
                      `(:wat::core::defrecord ~record-ty-decl :- [~@record-tp-syms] ~durable-fields)))
     ;; Build the State struct field vector: prepend [durable <- ::Record] before ephemeral fields.
     ;; Strategy: use quasiquote to build the durable-field prefix vector `[durable <- ~record-ty]`,
     ;; extract its 3 children, then prepend them to the ephemeral children via foldl.
     ;; The quasiquote gives us WatAST nodes (incl. the `<-` keyword) rather than runtime values.
     durable-prefix-vec `[durable <- ~record-ty-ann]
     durable-prefix-children (:wat::core::ast->children durable-prefix-vec)
     ephemeral-children (:wat::core::ast->children ephemeral-fields)
     ;; Concatenate: durable-prefix-children ++ ephemeral-children
     state-field-items (:wat::core::foldl
                         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                          item <- :wat::WatAST]
                           -> (:wat::core::Vector :- [:wat::WatAST])
                           (:wat::core::conj acc item))
                         (:wat::core::foldl
                           (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                            item <- :wat::WatAST]
                             -> (:wat::core::Vector :- [:wat::WatAST])
                             (:wat::core::conj acc item))
                           (:wat::core::Vector :wat::WatAST)
                           durable-prefix-children)
                         ephemeral-children)
     ;; Build the state field vector as a WatAST::Vector using with-children on empty-vec
     state-field-vec (:wat::core::with-children empty-vec state-field-items)
     ;; Arc 109 ③ — same siblings-binder splice as `record-def` above, over `state-tp-syms`.
     state-def    (:wat::core::if (:wat::core::empty? state-tp-syms)
                    `(:wat::core::defstruct ~state-ty-decl ~state-field-vec)
                    `(:wat::core::defstruct ~state-ty-decl :- [~@state-tp-syms] ~state-field-vec))

     ;; ── Arc 278 S4d: :peers — the s2s dependency DAG + cross-fork manifest ──────────
     ;; A :satisfies service that DIALS another service holds a client (Peer :- [S::Op S::Reply])
     ;; in a ROOT :ephemeral field and calls S's surface methods on it. :peers [:S1 …] is the
     ;; EXPLICIT declaration of those dialed surfaces.
     ;;
     ;; BIJECTION (set equality, by surface): the SET of :peers surfaces MUST EQUAL the SET of
     ;; root-ephemeral peer-field surfaces. A peer field is any root :ephemeral field whose type
     ;; is `(:wat::kernel::Peer :- [S::Op S::Reply])` — its surface is S (the first type-arg minus the
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
                      
                      (:wat::core::Option/expect
                        (:wat::core::HashMap/get clause-map "peers")
                        "defservice: :peers needs a value")
                      empty-vec)
     peers-children (:wat::core::ast->children peers-node)
     ;; peers-surfaces: (Vector :- [String]) — the declared peer surface fqdns (keyword/to-string each).
     peers-surfaces (:wat::core::foldl
                      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                                       pk  <- :wat::WatAST]
                        -> (:wat::core::Vector :- [:wat::core::String])
                        (:wat::core::conj acc (:wat::core::keyword/to-string pk)))
                      (:wat::core::Vector :wat::core::String)
                      peers-children)
     ;; ephemeral-peer-surfaces: (Vector :- [String]) — the surface of each ROOT ephemeral peer field.
     ;; ephemeral-children is the flat token vec [name <- :Type name <- :Type …]; the type node
     ;; of field i is at index i*3+2.
     ;;
     ;; ★ BRIEF-STONE-defservice-compares-types-as-data.md — the site that blocked ②-iii. This
     ;; used to hand-parse the type: render to a string, split on `"Peer<"`, split on `","`,
     ;; strip 4 chars for `"::Op"`. Every step is structural instead: normalize to a form
     ;; (branching on ast-kind, same as `surface-form` above — a Keyword needs
     ;; `keyword/to-type-form-colon`, a List is already the form), read the head and first
     ;; arg off `ast->children`, and check/strip the `"::Op"` suffix on the arg's OWN name —
     ;; not on a hand-sliced substring of the whole rendered type.
     ephemeral-peer-surfaces
                    (:wat::core::foldl
                      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                                       i   <- :wat::core::i64]
                        -> (:wat::core::Vector :- [:wat::core::String])
                        (:wat::core::let
                          [ty-node (:wat::core::Option/expect
                                     (:wat::core::get ephemeral-children
                                       (:wat::core::i64::+ (:wat::core::i64::* i 3) 2))
                                     "defservice: ephemeral field type out of bounds")
                           ty-form (:wat::core::if (:wat::core::= (:wat::core::ast-kind ty-node) "keyword")
                                     (:wat::core::keyword/to-type-form-colon ty-node)
                                     ty-node)]
                          (:wat::core::if (:wat::core::= (:wat::core::ast-kind ty-form) "list")

                            (:wat::core::let
                              [ty-ch    (:wat::core::ast->children ty-form)
                               head-str (:wat::core::keyword/to-string (:wat::core::first ty-ch))]
                              (:wat::core::if (:wat::core::= head-str "wat::kernel::Peer")

                                (:wat::core::let
                                  [arg-ch        (:wat::core::ast->children (:wat::core::nth ty-ch 2))
                                   first-arg-str (:wat::core::keyword/to-string (:wat::core::first arg-ch))]
                                  (:wat::core::if (:wat::string::ends-with? first-arg-str "::Op")

                                    (:wat::core::conj acc
                                      (:wat::string::subs first-arg-str 0
                                        (:wat::core::i64::- (:wat::string::length first-arg-str) 4)))
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
                          
                          ok
                          (:wat::core::macro-error
                            (:wat::string::concat fqdn-str
                              (:wat::string::concat ": :peers declares surface :"
                                (:wat::string::concat ps
                                  (:wat::string::concat
                                    " but no :ephemeral field is typed :wat::kernel::Peer<"
                                    (:wat::string::concat ps
                                      "::Op,…::Reply> — add the dialed peer as a root :ephemeral field, or drop it from :peers"))))))))
                      true
                      peers-surfaces)
     ;; BIJECTION check 2 (extra/undeclared): every ephemeral peer field's surface must be in :peers.
     _peers-extra   (:wat::core::foldl
                      (:wat::core::fn [ok <- :wat::core::bool  es <- :wat::core::String]
                        -> :wat::core::bool
                        (:wat::core::if (:wat::core::Vector/contains? peers-surfaces es)
                          
                          ok
                          (:wat::core::macro-error
                            (:wat::string::concat fqdn-str
                              (:wat::string::concat ": :ephemeral holds a dialed Peer<"
                                (:wat::string::concat es
                                  (:wat::string::concat "::Op,…::Reply> but surface :"
                                    (:wat::string::concat es
                                      (:wat::string::concat
                                        " is not declared in :peers — add :peers [… :"
                                        (:wat::string::concat es " …] (the explicit s2s dependency DAG)"))))))))))
                      true
                      ephemeral-peer-surfaces)
     ;; peer-forms-calls: (Vector :- [WatAST]) of `(:S::surface-forms)` call nodes — one per :peers surface.
     ;; Spliced into the service-forms concat (below) so each dialed surface's forms cross the fork.
     ;; DESIGN-STONE the-child-needs-the-entry-not-the-library: each contributor to
     ;; service-forms ships only when the child cannot already have it. A `:wat::`-rooted
     ;; peer surface can only have been declared by baked stdlib source (`:wat::` is
     ;; reserved — `src/resolve/reserved.rs:25-27` — so the gate refuses a user-privilege
     ;; registration under it); the child's own bake already has it, so its
     ;; `::surface-forms` call is dropped here rather than concatenated.
     peer-forms-calls (:wat::core::foldl
                        (:wat::core::fn [acc   <- (:wat::core::Vector :- [:wat::WatAST])
                                         s-str <- :wat::core::String]
                          -> (:wat::core::Vector :- [:wat::WatAST])
                          (:wat::core::if (:wat::string::starts-with? s-str "wat::")

                            acc
                            (:wat::core::let
                              [sf-kw (:wat::core::keyword/from-string
                                       (:wat::string::interpolate "{s-str}::surface-forms" :s-str s-str))]
                              (:wat::core::conj acc `(~sf-kw)))))
                        (:wat::core::Vector :wat::WatAST)
                        peers-surfaces)

     ;; Arc 293 S2 — Op/Reply live under the PROTOCOL namespace (proto-str): the surface's
     ;; when :satisfies, else this service's own fqdn (identical to pre-S2 for the :ops path).
     ;;
     ;; Arc 278 the parametric protocol — enum-name / reply-name stay at the BASE. They are the
     ;; NAME-identity spellings, not type positions: `derive` registers a subtype edge between
     ;; base names, `retag-op'` compares them against a runtime value's `type_path` (which is the
     ;; base), and the child-main `listener'` re-resolves them in a freshly-started child. The
     ;; TYPE-position spellings are `proto-op-ty-ann` / `proto-reply-ty-ann` below. Their
     ;; NAME-embedded `<a,b,…>` siblings (`proto-op-ty-str`/`proto-reply-ty-str`) are
     ;; RETIRED, STONE-exactly-one-call-position — `launch-head-kw` was their one
     ;; remaining consumer and it now takes a bare head + call-site `:-` siblings, same as
     ;; every other call position.
     enum-name     (:wat::core::keyword/from-string
                     (:wat::string::interpolate "{proto-base}::Op" :proto-base proto-base))
     ;; Arc 278 no-hidden-failures — the reserved PROTOCOL-TIER failure variant. Synthesized
     ;; onto every `<S>::Reply` by `synthesize_surface_protocol` (src/types.rs). The serve loop
     ;; replies `(Reply::Failed cause)` to a client whose message could not be decoded, and the
     ;; generated client method surfaces it as an unignorable raise carrying the cause's reason.
     reply-failed-kw (:wat::core::keyword/from-string
                       (:wat::string::interpolate "{proto-base}::Reply::Failed" :proto-base proto-base))
     serve-name    (:wat::core::keyword/from-string
                     (:wat::string::interpolate "{b}::serve" :b fqdn-base))
     ;; Arc 209 host-parity-4a — the serve fqdn as a STRING, spliced into start's
     ;; `(keyword/from-string …)` so Locus/launch receives serve by a RUNTIME keyword
     ;; (a spliced literal `:fqdn::serve` would Arc-009-resolve to a Fn, not a keyword).
     serve-name-str (:wat::string::interpolate "{b}::serve" :b fqdn-base)
     ;; 293.W.2f — (Handle :- [T]) / (Status :- [T]) carry the transport marker (Shared | Wire).
     ;; Bare `::Handle{p}` / `::Status{p}` (T unknown) remain the residual.
     ;; Transport param is `T` unless the service already binds `T` (`box-svc :- [T]`,
     ;; via the `:- [T]` binder), in which case it is `Xt` so the two slots do not
     ;; collide. STONE-the-last-mint — `fqdn-tp` (the angle-string mint this used to
     ;; `string::contains?` against) is RETIRED; the check is now structural, directly
     ;; over `fqdn-tp-syms` (a symbol named "T" among the declared params), never a
     ;; re-serialized `<a,b>` string.
     binds-t?     (:wat::core::foldl
                     (:wat::core::fn [acc <- :wat::core::bool sym <- :wat::WatAST] -> :wat::core::bool
                       (:wat::core::if acc true (:wat::core::= (:wat::core::ast-name sym) "T")))
                     false
                     fqdn-tp-syms)
     transport-param (:wat::core::if binds-t? "Xt" "T")
     ;; Arc 109 ③ — angle brackets are ILLEGAL for types; the `<…>` suffix-string mint
     ;; (`handle-t-suffix`/`handle-shared-suffix`/`handle-wire-suffix`) is RETIRED in favour
     ;; of extending `fqdn-tp-syms` structurally (`conj` one more arg node) and minting the
     ;; reference FORM `(Head :- [args])` directly via quasiquote.
     ;;
     ;; `handle-bare-name` USED to keep a narrower arg list (`fqdn-tp-syms` only, no transport
     ;; param) matching a documented "T unknown residual" role. That role does not survive
     ;; structural typing: `handle-record` (below) registers the REAL `Handle` at arity 3
     ;; (`handle-tp-syms` — K,V,T), so a 2-arg reference is not a valid partial instantiation of
     ;; it, just an under-arity mismatch — `parse_type_form`'s structural `Parametric{head,args}`
     ;; enforces that where the old angle-string round-trip apparently never got exercised
     ;; end-to-end (`wat --check` on `wat/cache.wat` hit exactly this: `:wat::cache::lru-svc/grant`
     ;; and `Handle/addr` both expect the 3-arg `(Handle :- [K V T])` `handle-bare-name` is the RECEIVER
     ;; type for). `handle-bare-name` now carries the full `handle-tp-syms` arg list — byte-
     ;; identical to `handle-name-ann` — closing that gap.
     handle-ty-str    (:wat::string::interpolate "{b}::Handle" :b fqdn-base)
     handle-base-kw   (:wat::core::keyword-node (:wat::string::concat ":" handle-ty-str))
     handle-name-decl (:wat::core::keyword/from-string handle-ty-str)
     handle-tp-syms   (:wat::core::conj fqdn-tp-syms (:wat::core::symbol-node transport-param))
     ;; identity 2c: handle-name split by role — DECL-NAME (Handle defstruct's own name slot,
     ;; below) stays the bare keyword (the binder splices as siblings there); ANNOTATION
     ;; (start/resume$impl return types) mints the reference FORM.
     handle-name-ann  `(~handle-base-kw :- [~@handle-tp-syms])
     handle-bare-name handle-name-ann
     ;; identity 2c: handle-shared-name / handle-wire-name are ANNOTATION-only (ann-form
     ;; ascriptions + start/resume$impl-thread/-process return types) — mint the reference FORM.
     handle-shared-tp-syms (:wat::core::conj fqdn-tp-syms (:wat::core::keyword-node ":wat::kernel::Shared"))
     handle-shared-name `(~handle-base-kw :- [~@handle-shared-tp-syms])
     handle-wire-tp-syms (:wat::core::conj fqdn-tp-syms (:wat::core::keyword-node ":wat::kernel::Wire"))
     handle-wire-name `(~handle-base-kw :- [~@handle-wire-tp-syms])
     ;; handle-new-kw: :<fqdn>::Handle' — the PRIME positional ctor (arc 294 item 9a: the bare
     ;; `:<fqdn>::Handle` is now the kwargs UX macro; generated machinery constructs via the prime,
     ;; exactly as state-new-kw does for the State struct — see start-body/resume-body below).
     handle-new-kw (:wat::core::keyword/from-string
                     (:wat::string::interpolate "{b}::Handle'" :b fqdn-base))
     ;; Parametric type keywords for serve's typed params. Arc 293 S2 — Op/Reply are the
     ;; PROTOCOL's (proto-str), so a :satisfies service's serve/client peers share the
     ;; surface's uniform (Address :- [S::Op S::Reply]). (proto-str = fqdn-str for the :ops path.)
     ;; (Peer :- [proto::Reply proto::Op])
     ;; (Listener :- [proto::Op proto::Reply])
     ;; identity 2c: listener-ty / addr-ty / client-peer-ty are ANNOTATION-only — mint the
     ;; reference FORM directly.
     ;;
     ;; Arc 109 ③ — angle brackets are ILLEGAL for types. NAME-construction consumers
     ;; (method-name interpolation, the runtime `retag-op` discriminator) key on `proto-base`
     ;; directly — those never reach the type parser. TYPE positions here mint their own
     ;; reference FORM structurally off `proto-args` (already a (Vector :- [WatAST]) of arg nodes —
     ;; see `proto-args`'s own derivation above), never a re-serialized `<a,b>` string.
     proto-op-base-kw    (:wat::core::keyword-node
                            (:wat::string::concat ":"
                              (:wat::string::interpolate "{b}::Op" :b proto-base)))
     proto-reply-base-kw (:wat::core::keyword-node
                            (:wat::string::concat ":"
                              (:wat::string::interpolate "{b}::Reply" :b proto-base)))
     ;; STONE-exactly-one-call-position — UNCONDITIONAL. `(Head :- [])` now IS `Head`:
     ;; `parse_type_form`'s `:-` arm normalises an empty peeled binder to `TypeExpr::Path`
     ;; (the same variant the bare reference already parses to), so a monomorphic service's
     ;; `(~proto-op-base-kw :- [])` and its bare `proto-op-base-kw` are structurally
     ;; identical from here on — no second branch needed to keep them that way. This was
     ;; MEASURED conditional before that normalisation landed (245 type-check errors on
     ;; stdlib load, a genuine `TypeMismatch: parameter #1 expects :u::Plain; got :u::Plain`
     ;; — a type that did not match itself); the fix is at the parser, not here.
     proto-op-ty-ann    `(~proto-op-base-kw :- [~@proto-args])
     proto-reply-ty-ann `(~proto-reply-base-kw :- [~@proto-args])
     listener-ty   `(:wat::kernel::Listener :- [~proto-op-ty-ann ~proto-reply-ty-ann])
     ;; (Vector :- [(Peer :- [proto::Reply proto::Op])])
     ;; (Address :- [proto::Op proto::Reply T]) — T is Handle/Status's transport marker (293.W.2f).
     addr-ty       `(:wat::kernel::Address :- [~proto-op-ty-ann ~proto-reply-ty-ann ~(:wat::core::symbol-node transport-param)])
     ;; Client (Peer :- [proto::Op proto::Reply]) — connect'((Address :- [Op Reply])) → (Peer :- [Op Reply]).
     ;; This is the client-side peer (sends Op, receives Reply); distinct from
     ;; peer-ty ((Peer :- [Reply Op])) which is the server-side peer (accepts via listener').
     client-peer-ty `(:wat::kernel::Peer :- [~proto-op-ty-ann ~proto-reply-ty-ann])

     ;; ── arc 291 3a-ii-α: lineage protocol types ──────────────────────────────
     ;; Admin enum:     :<fqdn>::Admin  — what the owner sends DOWN the lineage peer.
     ;;   :Init [seed <- :ship-ty]  — startup init-args (replaces raw ship)
     ;;   :Stop                     — owner-initiated stop (3a-ii-β dispatches this)
     ;; Status enum: :<fqdn>::Status — what the service sends UP the lineage peer.
     ;;   :Started [addr <- :addr-ty]    — startup address handoff (replaces raw addr)
     ;;   :Stopped   [state <- :state-ty]  — stop response (3a-ii-β uses this)
     ;;
     ;; self-peer type in child-main-form: (ThreadSelfPeer :- [Status Admin])
     ;;   child sends Status up, receives Admin down.
     ;;   Arc 293.W.2d: thread-tier uses ThreadSelfPeer (any I/O); process-tier `apply`
     ;;   bypasses the type check so the same serve fn works for both tiers.
     ;;
     ;; dispatch-admin: fn [ai <- Admin] -> State
     ;;   wraps the startup handshake: matches Admin::Init, applies <fqdn>::init.
     ;;   Passed to Locus/launch by-name in place of the raw init keyword.
     ;;
     ;; extract-addr: fn [lu <- Status] -> addr-ty
     ;;   matches Status::Started, returns the Address. Passed to launch as
     ;;   lu-addr-kw so the generic ProcessOpts impl can extract addr without
     ;;   naming per-service types.
     ;; Arc 109 β-ii-c — Admin's own field set: :Init/:Resume both carry `init-params-vec`
     ;; verbatim (the user's :init signature, already available here — β-ii-a′); :Stop/
     ;; :Hibernate are nullary and :AllowPeer/:DenyPeer carry `(Vector :- [i64])` (concrete), so
     ;; neither contributes a param. Searching `init-params-vec` alone is therefore exact.
     admin-tp-syms  (:wat::core::type-params-used-in fqdn-tp-syms init-params-vec)
     admin-ty-str   (:wat::string::interpolate "{b}::Admin" :b fqdn-base)
     admin-ty-decl  (:wat::core::keyword/from-string admin-ty-str)
     admin-base-kw  (:wat::core::keyword-node (:wat::string::concat ":" admin-ty-str))
     ;; identity 2c: admin-ty split by role — DECL-NAME (`defenum`'s own name slot, below)
     ;; stays the bare keyword (the binder splices as siblings there); ANNOTATION
     ;; (dispatch-admin-def's param type) and RUNTIME-ARG (`self-peer`'s arg) both mint the
     ;; reference FORM — `parse_peer_pair_type_arg` (src/check.rs) now accepts it (Arc 109 ③
     ;; closed the checker-side gap that made RUNTIME-ARG Keyword-only).
     admin-ty-ann     (:wat::core::if (:wat::core::empty? admin-tp-syms)
                        admin-base-kw
                        `(~admin-base-kw :- [~@admin-tp-syms]))
     admin-ty-runtime admin-ty-ann
     ;; 293.W.2f — (Status :- [T]) so Started's addr-ty T is a real type parameter
     ;; (not a rigid leftover name). Process launch unifies T:=Wire; thread T:=Shared.
     status-ty-str (:wat::string::interpolate "{b}::Status" :b fqdn-base)
     status-ty-decl  (:wat::core::keyword/from-string status-ty-str)
     status-base-kw  (:wat::core::keyword-node (:wat::string::concat ":" status-ty-str))
     ;; Status carries the same transport marker as Handle (293.W.2f) — `handle-tp-syms`
     ;; (fqdn-tp-syms + transport-param) is the identical arg list.
     ;; identity 2c STOP-2 — PREVIOUSLY left unconverted (a documented `wat::services`
     ;; 128/128 -> 61/128 `UnresolvedReference` regression). CLOSED: `src/resolve/walk.rs`'s
     ;; `check_form` already carries the `is_type_reference` / `is_binder_marker` guard (the
     ;; resolver-side SIBLING of the `macros/expand.rs` guard) that declines a `(Head :- [args])`
     ;; form as a call head — the exact mechanism the regression needed. Converts clean.
     status-ty-ann     `(~status-base-kw :- [~@handle-tp-syms])
     status-ty-runtime status-ty-ann
     ;; arc 291 3a-ii-β: the CHILD's lineage self-peer — sends Status UP, recvs Admin DOWN.
     ;; serve binds `self` to this (distinct from the client peer-ty (Peer :- [Reply Op])).
     ;; Arc 293.W.2d: serve's self is (ThreadSelfPeer :- [Status Admin]) for thread-tier.
     ;; Process-tier calls serve via `apply` (Locus/launch child-main-form), which bypasses
     ;; the type check — the process-tier (Peer :- [Status Admin]) from self-peer is accepted at
     ;; runtime without a static mismatch.
     ;; identity 2c STOP-2 — CLOSED, same finding as status-ty-ann above (the resolver guard
     ;; in `src/resolve/walk.rs` already covers the "signature captured as a first-class value"
     ;; path this comment used to flag as the open question).
     lineage-peer-ty `(:wat::kernel::ThreadSelfPeer :- [~status-ty-ann ~admin-ty-ann])
     admin-init-kw  (:wat::core::keyword/from-string
                      (:wat::string::interpolate "{b}::Admin::Init" :b fqdn-base))
     admin-stop-kw  (:wat::core::keyword/from-string
                      (:wat::string::interpolate "{b}::Admin::Stop" :b fqdn-base))
     ;; arc 291 4a: Admin::Hibernate (unit, like Stop) + Admin::Resume (carries snapshot).
     admin-hibernate-kw (:wat::core::keyword/from-string
                          (:wat::string::interpolate "{b}::Admin::Hibernate" :b fqdn-base))
     admin-resume-kw  (:wat::core::keyword/from-string
                        (:wat::string::interpolate "{b}::Admin::Resume" :b fqdn-base))
     status-started-kw (:wat::core::keyword/from-string
                          (:wat::string::interpolate "{b}::Status::Started" :b fqdn-base))
     ;; arc 278: the Status::Started ctor as a colon-free STRING (mirror of extract-addr-name-str),
     ;; so start/resume pass it as a runtime `(keyword/from-string …)` — an opaque :keyword the
     ;; launch surface accepts — rather than the resolved literal (which the checker would type as
     ;; the variant ctor Fn). The thread tier resolves it via `apply` at runtime → Status::Started.
     status-started-str (:wat::string::interpolate "{b}::Status::Started" :b fqdn-base)
     ;; arc 291 3a-ii-β: Status::Stopped — service replies with final state on admin stop.
     status-stopped-kw  (:wat::core::keyword/from-string
                          (:wat::string::interpolate "{b}::Status::Stopped" :b fqdn-base))
     ;; arc 291 4a: Status::Hibernated — service replies with full state on hibernate.
     status-hibernated-kw (:wat::core::keyword/from-string
                             (:wat::string::interpolate "{b}::Status::Hibernated" :b fqdn-base))
     ;; arc 278: Admin::AllowPeer[pids] — owner grants a vec of caller pids to the callee's
     ;; process-tier accept-gate (the circuit builder wiring process peers). Status::PeersAllowed
     ;; is the request/reply ack — the owner blocks on it so the grant is applied before the
     ;; caller dials (grant-before-dial ordering). Both cross the owner-only lineage peer.
     admin-allow-peer-kw (:wat::core::keyword/from-string
                           (:wat::string::interpolate "{b}::Admin::AllowPeer" :b fqdn-base))
     status-peers-allowed-kw (:wat::core::keyword/from-string
                               (:wat::string::interpolate "{b}::Status::PeersAllowed" :b fqdn-base))
     ;; arc 278: fold binders for the serve AllowPeer arm's (allow' l pid) sweep — synthetic
     ;; fn binders introduced in the serve template → symbol-node + unquote for hygiene.
     allow-acc-sym (:wat::core::symbol-node "acc")
     allow-pid-sym (:wat::core::symbol-node "pid")
     ;; arc 293: Admin::DenyPeer[pids] — mirror of AllowPeer, owner revokes a vec of caller
     ;; pids from the callee's process-tier accept-gate. Status::PeersDenied is the
     ;; request/reply ack — the owner blocks on it so the revoke is applied before it returns.
     admin-deny-peer-kw (:wat::core::keyword/from-string
                          (:wat::string::interpolate "{b}::Admin::DenyPeer" :b fqdn-base))
     status-peers-denied-kw (:wat::core::keyword/from-string
                              (:wat::string::interpolate "{b}::Status::PeersDenied" :b fqdn-base))
     ;; arc 293: fold binders for the serve DenyPeer arm's (deny' l pid) sweep — synthetic
     ;; fn binders introduced in the serve template → symbol-node + unquote for hygiene.
     deny-acc-sym (:wat::core::symbol-node "acc")
     deny-pid-sym (:wat::core::symbol-node "pid")
     dispatch-admin-name-str (:wat::string::interpolate "{b}::dispatch-admin" :b fqdn-base)
     dispatch-admin-name (:wat::core::keyword/from-string (:wat::string::interpolate "{b}::dispatch-admin" :b fqdn-base))
     extract-addr-name-str (:wat::string::interpolate "{b}::extract-addr" :b fqdn-base)
     extract-addr-name (:wat::core::keyword/from-string (:wat::string::interpolate "{b}::extract-addr" :b fqdn-base))

     ;; ── arc 291 3a-ii-α: Admin + Status defenums ──────────────────────────
     ;; Admin: Init carries the seed (ship-ty); Stop is unit (3a-ii-β dispatches it).
     ;; Status: Started carries the minted Address; Final carries the final state.
     ;; :Stop and :Shutdown are unit variants (bare keyword, no field vector) —
     ;; matches as a bare keyword pattern (ev.fields.is_empty() ✓).
     ;; arc 291 4b-ii: Admin now has four variants:
     ;;   Init (startup seed), Stop (unit), Hibernate (unit), Resume (snapshot).
     ;;   Init and Resume both carry ::Record (not ::State — structs never cross the wire).
     ;; arc 278: Admin::AllowPeer[pids] — a vec of caller pids to grant to the accept-gate.
     ;; arc 293: Admin::DenyPeer[pids] — mirror, a vec of caller pids to revoke from it.
     ;; Arc 109 ③ — `admin-ty-decl`/`status-ty-decl` are the BARE names; splice the `:-`
     ;; binder as declaration siblings (immediately after the name, before the purity
     ;; keyword) when their tp-syms are non-empty.
     admin-enum-def (:wat::core::if (:wat::core::empty? admin-tp-syms)
                      `(:wat::core::defenum ~admin-ty-decl :wat::enum::Pure
                         :Init     ~init-params-vec
                         :Stop
                         :Hibernate
                         :Resume   ~init-params-vec
                         :AllowPeer [pids <- (:wat::core::Vector :wat::core::i64)]
                         :DenyPeer [pids <- (:wat::core::Vector :wat::core::i64)])
                      `(:wat::core::defenum ~admin-ty-decl :- [~@admin-tp-syms] :wat::enum::Pure
                         :Init     ~init-params-vec
                         :Stop
                         :Hibernate
                         :Resume   ~init-params-vec
                         :AllowPeer [pids <- (:wat::core::Vector :wat::core::i64)]
                         :DenyPeer [pids <- (:wat::core::Vector :wat::core::i64)]))
     ;; arc 291 4b-ii: Status::Hibernated carries ::Record (not ::State).
     ;; arc 278: Status::PeersAllowed (unit) — the AllowPeer request/reply ack.
     ;; arc 293: Status::PeersDenied (unit) — the DenyPeer request/reply ack.
     ;; Status's tp-syms are `handle-tp-syms` (always non-empty — the transport marker),
     ;; so the binder splice here is unconditional (mirrors `handle-record` above).
     status-enum-def `(:wat::core::defenum ~status-ty-decl :- [~@handle-tp-syms] :wat::enum::Pure
                             :Started   [addr     <- ~addr-ty]
                             :Stopped     [resp     <- ~resp-ty]
                             :Hibernated [snapshot <- ~record-ty-ann]
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
     dispatch-admin-def `(:wat::core::defn ~dispatch-admin-name [ai <- ~admin-ty-ann] -> ~state-ty-ann
                            (:wat::core::match ai 
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
     ;; can extract the Address without naming per-service Status types.
     lu-sym     (:wat::core::symbol-node "lu")
     extract-addr-def `(:wat::core::defn ~extract-addr-name
                                  [lu <- ~status-ty-ann] -> ~addr-ty
                                  (:wat::core::match lu 
                                    ((~status-started-kw addr) addr)
                                    (_ (:wat::kernel::assertion-failed!
                                         "defservice extract-addr: unexpected Status variant (expected Started)"
                                         :wat::core::None
                                         :wat::core::None))))

     clauses       (:wat::core::ast->children ops)            ;; list of op-List nodes
     impl-clauses  (:wat::core::if satisfies?
                     
                     clauses
                     (:wat::core::Vector :wat::WatAST))

     ;; ── Arc 278 Stone 2-A (self-scheduling): the <service>::Op SUPERSET (Option A) ──────
     ;; The serve loop dispatches over :<fqdn>::Op = the surface's <proto>::Op variants
     ;; (field-for-field, so `retag-op'` embeds them) PLUS the service's internal leading-dash
     ;; ops (nullary). The WIRE stays <proto>::Op — a client can only construct surface ops; a
     ;; client op is RE-TAGGED into its <service>::Op counterpart at the Message arm. selectables
     ;; (the poll' set) is typed with the superset O; the O flows into `(Outcome :- [S R O])`/`(Alarm :- [O])`.
     service-op-str  (:wat::string::interpolate "{b}::Op" :b fqdn-base)
     service-op-kw   (:wat::core::keyword/from-string service-op-str)
     ;; Arc 278 the parametric protocol — the SUPERSET enum's DECLARED name carries the service's
     ;; own params (its variant fields name the surface's parametric messages, so the binders must
     ;; be in scope), and every TYPE-position reference below instantiates it at those params.
     ;; `service-op-str` stays BASE: it is the ctor/variant namespace, the `retag-op'` runtime
     ;; target, and the `derive` edge's child. Monomorphic ⇒ fqdn-tp-syms is empty ⇒ the two coincide.
     ;;
     ;; Arc 109 ③ — angle brackets are ILLEGAL for types; `service-op-ty-str`'s old `<…>`-suffixed
     ;; spelling is RETIRED. `service-op-ty-ann` mints the reference FORM off `fqdn-tp-syms`
     ;; structurally. `service-op-decl-kw-decl` is now the BARE name (siblings binder spliced at
     ;; `service-op-def`, below); the RUNTIME-ARG role (`retag-op`'s target-type argument) now
     ;; uses `service-op-ty-ann` too — `parse_peer_pair_type_arg` (src/check.rs) accepts the
     ;; reference form since Arc 109 ③ closed that checker-side gap (identity 2b's byte-identical
     ;; DECL-NAME/RUNTIME-ARG alias no longer holds; the two roles now want different spellings).
     service-op-ty-ann (:wat::core::if (:wat::core::empty? fqdn-tp-syms)
                         service-op-kw
                         `(~service-op-kw :- [~@fqdn-tp-syms]))
     service-op-decl-kw-decl service-op-kw
     service-op-decl-kw-runtime service-op-ty-ann
     ;; selectable-peer-ty: (Peer :- [proto::Reply service::Op]) (the poll' element — superset O).
     ;; Arc 109 ③ — reference FORM, structurally off `proto-reply-ty-ann` / `service-op-ty-ann`.
     selectable-peer-ty `(:wat::kernel::Peer :- [~proto-reply-ty-ann ~service-op-ty-ann])
     ;; selectable-vec-ty: (Vector :- [(Peer :- [proto::Reply service::Op])]) — the BARE peer vector, the
     ;; shape `:wat::kernel::poll`/`:wat::kernel::serve-dispatch-op` require (both are Rust
     ;; intrinsics that downcast every element to a real Peer opaque; neither can see through a
     ;; wrapper). Still used as the PROJECTED view built fresh each iteration (`peers-only-expr`
     ;; below) — never as `selectables`' own declared type anymore (see `selectable-entry-vec-ty`).
     ;; identity 2c: selectable-vec-ty is ANNOTATION-only (fold accumulator param + return type)
     ;; — mints the reference FORM directly.
     selectable-vec-ty `(:wat::core::Vector :- [~selectable-peer-ty])
     ;; ── arc 278 the call context: the caller id travels WITH its peer (STOP-2) ──────────────
     ;; selectable-entry-ty: (i64, (Peer :- [R O])). Arc 109 ③ — the OLD native tuple-STRING spelling
     ;; `:(T1,T2)` embedded `selectable-peer-ty-str`'s `<…>`, now illegal. `parse_type_form`'s
     ;; `raw_head == "wat::core::Tuple"` special-case (src/types.rs, ~5042) produces the
     ;; IDENTICAL `TypeExpr::Tuple` structurally off `(:wat::core::Tuple :- [args])` — so this
     ;; mints THAT instead; it unifies the same way the string spelling did (both collapse to
     ;; `TypeExpr::Tuple`, never `Parametric{head:"wat::core::Tuple",…}`). ONE element, id+peer
     ;; coupled by construction (never a second vector keyed by position: `remove-at` drops both
     ;; together, always, because there is only ever one vector). selectable-entry-vec-ty is
     ;; `selectables`' REAL declared type from here down; `selectable-vec-ty`/`selectable-peer-ty`
     ;; above survive only as the bare-peer PROJECTION poll'/serve-dispatch-op still need.
     ;;
     ;; identity 2c had split CTOR-ARG vs ANNOTATION into `-ctor`/`-ann` aliases of a `-raw`
     ;; base, because the OLD Keyword-only `Vector` ctor-arg check couldn't accept a reference
     ;; FORM. `infer_list_constructor` (src/check.rs) now accepts the List form too (arc 109
     ;; ②-iii widening) — one node serves both roles, so that split collapses back to one name.
     selectable-entry-ty `(:wat::core::Tuple :- [:wat::core::i64 ~selectable-peer-ty])
     ;; identity 2c: selectable-entry-vec-ty is ANNOTATION-only (arm-fold param/return type +
     ;; `serve-params`' `selectables` field) — mints the reference FORM directly.
     selectable-entry-vec-ty `(:wat::core::Vector :- [~selectable-entry-ty])
     ;; peers-only-expr — the BARE-peer projection `:wat::kernel::poll` / `:wat::kernel::
     ;; serve-dispatch-op` need (both are Rust intrinsics that downcast every Vector element to
     ;; a real Peer opaque; a Tuple wrapper is invisible to them). Built fresh, once per
     ;; serve-loop iteration, from the single canonical `selectables` — a DERIVED view, never an
     ;; independently-mutated structure, so it cannot desync (STOP-2 again: the risk that rule
     ;; guards against is two vectors someone forgets to update together; a projection
     ;; recomputed from the one source of truth has nothing to forget). Computed ONCE here
     ;; (outer macro scope) so `serve-body` below can splice it at both call sites.
     peers-acc-sym (:wat::core::symbol-node "pacc")
     peers-t-sym   (:wat::core::symbol-node "pt")
     peers-fold-fn `(:wat::core::fn [~peers-acc-sym <- ~selectable-vec-ty  ~peers-t-sym <- ~selectable-entry-ty]
                        -> ~selectable-vec-ty
                      (:wat::core::conj ~peers-acc-sym (:wat::core::second ~peers-t-sym)))
     peers-only-expr `(:wat::core::foldl ~peers-fold-fn (:wat::core::Vector ~selectable-peer-ty) selectables)
     ;; alarm-o-ty: (Alarm :- [service::Op]) — the arm-foldl binder type.
     ;; identity 2c: ANNOTATION-only (arm-fold's alarm param type) — mints the reference FORM,
     ;; structurally off `service-op-ty-ann` (Arc 109 ③ retired the angle-string concat).
     alarm-o-ty      `(:wat::service::Alarm :- [~service-op-ty-ann])
     ;; The superset variant items: a flat [variant-kw field-vec …] (Vector :- [WatAST]) spliced into
     ;; the defenum. A surface op → `:Pascal [req <- :<proto>::<Pascal>Request]` (mirrors the
     ;; surface Op variant's field, so `retag-op'` embeds field-for-field); an internal `-op` →
     ;; `:-Pascal []` (nullary). Dash preserved SCOPED here (strip `-`, kebab->pascal, re-prepend
     ;; `-`) — NOT the global `kebab_to_pascal_with_acronyms`.
     service-op-variant-items
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])  clause <- :wat::WatAST]
           -> (:wat::core::Vector :- [:wat::WatAST])
           (:wat::core::let
             [op-str      (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children clause)))
              is-internal (:wat::string::starts-with? op-str "-")
              variant-pascal (:wat::core::if is-internal
                               
                               (:wat::string::concat "-"
                                 (:wat::string::kebab->pascal-in surface-kw
                                   (:wat::string::subs op-str 1 (:wat::string::length op-str))))
                               (:wat::string::kebab->pascal-in surface-kw op-str))
              variant-kw-node (:wat::core::keyword-node
                                (:wat::string::interpolate ":{variant-pascal}" :variant-pascal variant-pascal))
              field-vec   (:wat::core::if is-internal
                            
                            empty-vec
                            (:wat::core::let
                              ;; Arc 278 the surface-minted op alias — NAME the alias Rust mints
                              ;; at the surface's registration site (`<Surface>::<op>/Request`,
                              ;; src/types.rs) instead of guessing the message's type name by
                              ;; concatenation. The alias's type ARGS re-attach (`p`), mirroring
                              ;; the surface Op variant's field EXACTLY (the `derive` edge +
                              ;; `retag-op'` both require field-for-field identity). Monomorphic
                              ;; ⇒ proto-args is `[]` ⇒ `(Head :- [])` IS the bare alias name.
                              ;; identity 2c: ANNOTATION-only (this variant's own field type) —
                              ;; mints the reference FORM. Arc 109 ③ — structurally off
                              ;; `proto-args` (the surface's own arg-node list), not the retired
                              ;; `proto-tp` angle-string suffix. STONE-exactly-one-call-position —
                              ;; UNCONDITIONAL (see the note beside `proto-op-ty-ann`, above):
                              ;; `(Head :- [])` IS `Head` now, so the monomorphic case needs no
                              ;; second branch.
                              [req-base-kw (:wat::core::keyword-node
                                             (:wat::string::concat ":"
                                               (:wat::string::interpolate "{b}::{op}/Request"
                                                 :b proto-base :op op-str)))
                               req-ty `(~req-base-kw :- [~@proto-args])]
                              `[req <- ~req-ty]))]
             (:wat::core::conj (:wat::core::conj acc variant-kw-node) field-vec)))
         (:wat::core::Vector :wat::WatAST)
         impl-clauses)
     ;; Arc 109 ③ — `service-op-decl-kw-decl` is now the BARE name; splice `:- [~@fqdn-tp-syms]`
     ;; as declaration siblings when the service is genuinely parametric.
     service-op-def (:wat::core::if (:wat::core::empty? fqdn-tp-syms)
                      `(:wat::core::defenum ~service-op-decl-kw-decl :wat::enum::Pure ~@service-op-variant-items)
                      `(:wat::core::defenum ~service-op-decl-kw-decl :- [~@fqdn-tp-syms] :wat::enum::Pure ~@service-op-variant-items))
     ;; ── Arc 278 reconciliation (b): surface-Op <: service-Op subtype edge ──────────────
     ;; The serve loop's `selectables` param is typed with the SUPERSET `<service>::Op`
     ;; (`selectable-peer-ty` above), but CLIENT peers speak the SURFACE `<proto>::Op` — a
     ;; caller-constructed `(Peer :- [proto::Reply proto::Op])` must be assignable into the
     ;; superset-O `(Vector :- [(Peer :- [proto::Reply service::Op])])`. `service::Op` is a genuine
     ;; superset (every surface variant embedded field-for-field + the internal `-op`s), and
     ;; `retag-op'` (wat/service.wat:1080) re-tags a client's surface op into its service-Op
     ;; counterpart at dispatch — so a surface-Op peer soundly satisfies a superset-Op slot
     ;; (covariant widening in Peer's received-Op position, one-directional). We register
     ;; the check-time edge via the ordinary `derive` mechanism: `assignable`'s per-arg
     ;; subtype-lattice flow (Arc 278 Stone 2, src/check.rs) then relaxes ONLY the Op slot
     ;; (Reply has no edge → stays exact). Guarded on `proto-str /= fqdn-str` so a
     ;; self-satisfying service (surface Op IS the service Op, one type) never emits a
     ;; reflexive self-edge (which `register_subtype` rejects as CyclicSubtype).
     service-op-derive-items
       (:wat::core::if (:wat::core::= proto-base fqdn-base)

         (:wat::core::Vector :wat::WatAST)
         (:wat::core::conj (:wat::core::Vector :wat::WatAST)
           `(:wat::core::derive ~enum-name ~service-op-kw)))
     ;; keyword-:op resolution data: for each INTERNAL op, the body keyword string (`:-tick`) and
     ;; the SOURCE TEXT of its <service>::Op variant constructor (`(:<fqdn>::Op::-Tick)`). A
     ;; handler body's `:op :-tick` is resolved to the variant via an ast->source → split/join →
     ;; read-string round-trip (in serve-op-arms) — <service>::Op never leaks to the author, and
     ;; the leading-dash marker makes `:-tick` an unambiguous token (never a substring of an fqdn).
     internal-op-kw-strs
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  clause <- :wat::WatAST]
           -> (:wat::core::Vector :- [:wat::core::String])
           (:wat::core::let
             [op-str (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children clause)))]
             (:wat::core::if (:wat::string::starts-with? op-str "-")
               
               (:wat::core::conj acc (:wat::string::interpolate ":{op-str}" :op-str op-str))
               acc)))
         (:wat::core::Vector :wat::core::String)
         impl-clauses)
     internal-op-repl-strs
       (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])  clause <- :wat::WatAST]
           -> (:wat::core::Vector :- [:wat::core::String])
           (:wat::core::let
             [op-str (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children clause)))]
             (:wat::core::if (:wat::string::starts-with? op-str "-")
               
               (:wat::core::let
                 [variant-pascal (:wat::string::concat "-"
                                   (:wat::string::kebab->pascal-in surface-kw
                                     (:wat::string::subs op-str 1 (:wat::string::length op-str))))]
                 (:wat::core::conj acc
                   (:wat::string::concat "(:"
                     (:wat::string::concat service-op-str
                       (:wat::string::concat "::"
                         (:wat::string::concat variant-pascal ")"))))))
               acc)))
         (:wat::core::Vector :wat::core::String)
         impl-clauses)
     has-internal-ops? (:wat::core::i64::> (:wat::core::length internal-op-kw-strs) 0)

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
                     (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                      clause <- :wat::WatAST]
                       -> (:wat::core::Vector :- [:wat::WatAST])
                       (:wat::core::let
                         [ch            (:wat::core::ast->children clause)
                          op-node       (:wat::core::first ch)
                          op-str        (:wat::core::ast-name op-node)
                          is-internal   (:wat::string::starts-with? op-str "-")
                          param-vec     (:wat::core::nth ch 1)
                          body0         (:wat::core::nth ch 2)
                          ;; keyword-:op RESOLUTION — rewrite each internal-op keyword (`:-tick`) in
                          ;; the handler body to its <service>::Op variant ctor via a source
                          ;; round-trip (ast->source → split/join → read-string). Skipped when the
                          ;; service declares no internal ops (keeps existing bodies' spans intact).
                          body          (:wat::core::if has-internal-ops?
                                          
                                          (:wat::core::first
                                            (:wat::core::ast->children
                                              (:wat::core::match (:wat::core::read-string
                                                (:wat::core::foldl
                                                  (:wat::core::fn [src <- :wat::core::String  i <- :wat::core::i64]
                                                    -> :wat::core::String
                                                    (:wat::string::join
                                                      (:wat::core::Option/expect (:wat::core::get internal-op-repl-strs i) "internal-op-repl")
                                                      (:wat::string::split src
                                                        (:wat::core::Option/expect (:wat::core::get internal-op-kw-strs i) "internal-op-kw"))))
                                                  (:wat::core::ast->source body0)
                                                  (:wat::core::range 0 (:wat::core::length internal-op-kw-strs)))) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::core::macro-error (:wat::string::concat "defservice: internal-op body did not re-parse: " (:wat::core::Error/message __cause)))))))
                                          body0)
                          param-ch      (:wat::core::ast->children param-vec)
                          arity         (:wat::core::length param-ch)
                          ;; arc 278 ctx-is-mandatory — DESIGN-STONE-mandatory-ctx-and-lifecycle-ops.md
                          ;; "THE SHAPE, RULED": the discriminator is the NAME (is-internal, computed
                          ;; above from the leading `-`); arity is NEVER an input to what an arm means
                          ;; (STOP-2) — it is consulted only here, to REFUSE a wrong shape, located and
                          ;; naming the op.
                          ;;   internal  -op [s ctx]     → arity 2, ctx : SelfInvocation
                          ;;   public     op [s ctx req] → arity 3, ctx : Invocation
                          _arity-ok     (:wat::core::if is-internal
                                          (:wat::core::if (:wat::core::= arity 2)
                                            nil
                                            (:wat::core::macro-error
                                              (:wat::string::concat "defservice: internal op '"
                                                (:wat::string::concat op-str
                                                  (:wat::string::concat "' must have shape [s ctx] (2 params); got "
                                                    (:wat::string::concat (:wat::core::i64::to-string arity) " params"))))))
                                          (:wat::core::if (:wat::core::= arity 3)
                                            nil
                                            (:wat::core::macro-error
                                              (:wat::string::concat "defservice: public op '"
                                                (:wat::string::concat op-str
                                                  (:wat::string::concat "' must have shape [s ctx req] (3 params); got "
                                                    (:wat::string::concat (:wat::core::i64::to-string arity) " params")))))))
                          s-binder      (:wat::core::first param-ch)
                          ;; ctx-binder — the SECOND param, ALWAYS: both valid shapes ([s ctx] and
                          ;; [s ctx req]) place ctx at index 1. Safe once _arity-ok has passed (every
                          ;; surviving arity is >= 2).
                          ctx-binder    (:wat::core::first (:wat::core::rest param-ch))
                          ;; dash-preserved variant pascal (SCOPED — strip `-`, kebab->pascal,
                          ;; re-prepend `-`; NOT the global kebab_to_pascal_with_acronyms).
                          variant-pascal (:wat::core::if is-internal

                                           (:wat::string::concat "-"
                                             (:wat::string::kebab->pascal-in surface-kw
                                               (:wat::string::subs op-str 1 (:wat::string::length op-str))))
                                           (:wat::string::kebab->pascal-in surface-kw op-str))
                          ;; op-variant-kw: the SERVICE superset variant — the arm PATTERN dispatches
                          ;; over <service>::Op (post-retag), NOT the surface <proto>::Op.
                          op-variant-kw (:wat::core::keyword/from-string
                                          (:wat::string::concat service-op-str
                                            (:wat::string::interpolate "::{variant-pascal}" :variant-pascal variant-pascal)))
                          ;; reply-variant-kw: the SURFACE reply variant (surface ops only wrap a reply).
                          reply-variant-kw (:wat::core::keyword/from-string
                                             (:wat::string::concat proto-base
                                               (:wat::string::interpolate "::Reply::{variant-pascal}" :variant-pascal variant-pascal)))
                          state-sym     (:wat::core::symbol-node "state")
                          ;; arc 278 ctx-is-mandatory — the ctx CONSTRUCTOR CALLS, built here at
                          ;; macro-expand time. `~fqdn-kw`/`~op-str` splice as LITERALS;
                          ;; `selectables`/`idx` are bare — literal identifiers in the GENERATED code,
                          ;; evaluated at RUNTIME inside the serve loop (the impure boundary: the live
                          ;; connection table, a fresh Uuid, a clock read) — never at macro-expand
                          ;; time. Both forms are always built (cheap AST data, never evaluated here);
                          ;; only the is-internal branch below picks which one is spliced.
                          self-ctx-ctor-expr `(:wat::service::SelfInvocation
                                                 :namespace      ~fqdn-kw
                                                 :operation      ~op-str
                                                 :invocation-id (:wat::uuid::v4)
                                                 :start-ns       (:wat::time::epoch-nanos (:wat::time::now)))
                          pub-ctx-ctor-expr  `(:wat::service::Invocation
                                                 :conn-id        (:wat::core::first (:wat::core::nth selectables idx))
                                                 :namespace      ~fqdn-kw
                                                 :operation      ~op-str
                                                 :invocation-id (:wat::uuid::v4)
                                                 :start-ns       (:wat::time::epoch-nanos (:wat::time::now)))
                          ;; let-bindings [s-binder state ctx-binder self-ctx-ctor-expr] — the
                          ;; INTERNAL arm's binding vector. STOP-0, FIXED: a SelfInvocation ctx is now
                          ;; bound (was silently dropped — `with-children` takes param-vec's SHAPE,
                          ;; never its CONTENTS, so the old 2-item binding-items list bound only
                          ;; [s-binder state] whatever the param vector held).
                          binding-items (:wat::core::conj
                                          (:wat::core::conj
                                            (:wat::core::conj
                                              (:wat::core::conj (:wat::core::Vector :wat::WatAST) s-binder)
                                              state-sym)
                                            ctx-binder)
                                          self-ctx-ctor-expr)
                          let-bindings  (:wat::core::with-children param-vec binding-items)
                          ;; the ARM fn — folds each Alarm into `selectables` as an `after` timer at
                          ;; the service's OWN tier (env-grab own-kind → both loci). alarm.op is a
                          ;; concrete <service>::Op value → the timer is (Peer :- [Never O]), joins poll'.
                          ;; arc 278 the call context — `selectables`' element is now
                          ;; (Tuple :- [i64 (Peer :- [R O])]) (STOP-2: id travels WITH its peer, one vector,
                          ;; never a second one keyed by position), so a timer needs an id slot
                          ;; too, purely to keep the vector's element type uniform. `-1` is a
                          ;; SENTINEL, never a real caller id (real ids are minted >= 0 in the
                          ;; Connection arm) — and it is never READ: a fired timer only ever
                          ;; dispatches through the INTERNAL (`[s ctx]`) arm, whose ctx is a
                          ;; `SelfInvocation` — a type with NO `conn-id` field at all (STOP-3: an
                          ;; internal op never gets a caller identity, STRUCTURALLY, not merely by
                          ;; convention) — so nothing ever asks this sentinel for one.
                          arm-acc-sym   (:wat::core::symbol-node "acc")
                          arm-alarm-sym (:wat::core::symbol-node "alarm")
                          ;; `after` is honestly typed `(Peer :- [Never O])` (it can never RECEIVE a
                          ;; Reply — arc 278 Stone 2's own comment: "after's honest uninhabited
                          ;; send-type"). `assignable`'s (Peer :- [Never _]) <: (Peer :- [Reply _]) widening
                          ;; (src/check.rs, the "SAME-head parametric" branch) fires for a BARE
                          ;; Peer-vs-Peer compare, but `unify` recurses into Tuple elements
                          ;; WITHOUT re-entering `assignable` — so `(i64,(Peer :- [Never O]))` does not
                          ;; widen to `(i64,(Peer :- [Reply O]))` once nested inside the tuple (proven by
                          ;; running `--check` and reading the TypeMismatch: touching that is a
                          ;; src/check.rs change, outside this strike's blast radius). Route the
                          ;; timer peer through a one-element `(Vector :- [(Peer :- [Reply O])])` first — THAT
                          ;; conj DOES hit the working bare-Peer widening — then `first` it back out;
                          ;; the checker now reads it at `(Peer :- [Reply O])` before it ever reaches the
                          ;; Tuple ctor. Values are unaffected: Peer's type params are erased at
                          ;; runtime (this file's own comment elsewhere: "params are erased in a
                          ;; runtime type_path") — this is a check-time-only detour.
                          arm-fn        `(:wat::core::fn [~arm-acc-sym <- ~selectable-entry-vec-ty  ~arm-alarm-sym <- ~alarm-o-ty]
                                             -> ~selectable-entry-vec-ty
                                           (:wat::core::conj ~arm-acc-sym
                                             (:wat::core::Tuple -1
                                               (:wat::core::first
                                                 (:wat::core::conj (:wat::core::Vector ~selectable-peer-ty)
                                                   (:wat::kernel::after
                                                     (:wat::program::Env/peer-kind (:wat::program::env))
                                                     (:wat::service::Alarm/after ~arm-alarm-sym)
                                                     (:wat::service::Alarm/op ~arm-alarm-sym)))))))]
                         (:wat::core::if is-internal
                           
                           ;; ── INTERNAL op arm (2-param [s ctx], ctx : SelfInvocation) ────────────
                           ;; No req-binder, no #16.2 guard, no reply variant. On fire → REMOVE the
                           ;; one-shot timer's idx, then arm any re-arms. Reply/Stop/ReplyAndArm are
                           ;; meaningless (no client) → a located assertion (never silently dropped).
                           (:wat::core::conj acc
                             `((~op-variant-kw)
                                (:wat::core::match (:wat::core::let ~let-bindings ~body) 
                                  ((:wat::service::Outcome::NoReply new-state)
                                    (~serve-name self l (:wat::std::list::remove-at selectables idx) next-id new-state))
                                  ((:wat::service::Outcome::NoReplyAndArm new-state arms)
                                    (~serve-name self l
                                      (:wat::core::foldl ~arm-fn (:wat::std::list::remove-at selectables idx) arms)
                                      next-id
                                      new-state))
                                  ((:wat::service::Outcome::Reply new-state resp)
                                    (:wat::kernel::assertion-failed!
                                      "defservice: an internal (-) op returned Outcome::Reply, but an internal op has no client to reply to (return NoReply / NoReplyAndArm)"
                                      :wat::core::None :wat::core::None))
                                  ((:wat::service::Outcome::Stop final-state resp)
                                    (:wat::kernel::assertion-failed!
                                      "defservice: an internal (-) op returned Outcome::Stop, but an internal op has no client to reply to"
                                      :wat::core::None :wat::core::None))
                                  ((:wat::service::Outcome::ReplyAndArm new-state resp arms)
                                    (:wat::kernel::assertion-failed!
                                      "defservice: an internal (-) op returned Outcome::ReplyAndArm, but an internal op has no client to reply to (return NoReplyAndArm)"
                                      :wat::core::None :wat::core::None)))))
                           ;; ── SURFACE op arm (3-param [s ctx req], ctx : Invocation) ─────────────
                           ;; #16.2 budget guard; wraps the op's reply variant; KEEPS its idx (a
                           ;; client persists even on a NoReply cast). …AndArm folds new timers in.
                           ;; arc 278 ctx-is-mandatory — ctx is UNCONDITIONAL: `_arity-ok` above has
                           ;; already refused any arm that is not exactly [s ctx req], so there is no
                           ;; longer a fallback shape to dispatch on here (STOP-2: arity is never an
                           ;; input to what an arm means).
                           (:wat::core::let
                             [req-binder    (:wat::core::first (:wat::core::rest (:wat::core::rest param-ch)))
                              ;; arm-let-bindings — [s-binder state ctx-binder pub-ctx-ctor-expr] —
                              ;; ctx is bound via LET (never a match-pattern field): it is synthesized
                              ;; locally, not part of the wire message (the Op variant still carries
                              ;; exactly one field, `req`).
                              arm-let-bindings (:wat::core::with-children param-vec
                                                  (:wat::core::conj
                                                    (:wat::core::conj
                                                      (:wat::core::conj
                                                        (:wat::core::conj (:wat::core::Vector :wat::WatAST) s-binder)
                                                        state-sym)
                                                      ctx-binder)
                                                    pub-ctx-ctor-expr))
                              op-upper      (:wat::string::to-uppercase op-str)
                              cap-const-kw  (:wat::core::keyword/from-string
                                              (:wat::string::concat proto-base
                                                (:wat::string::interpolate "::{op-upper}-MAX-REQUEST-BYTES" :op-upper op-upper)))
                              ;; arc 278 #74 — `<Op>Response` is LAW (builder ruling, 2026-08-05),
                              ;; checker-enforced at `defsurface` registration
                              ;; (`synthesize_surface_protocol`, src/types.rs): a serviceable op's
                              ;; response type is REQUIRED to be `<variant-pascal>Response`, so
                              ;; the concatenation below is guaranteed correct by construction —
                              ;; never a guess. These are LITERAL ctor keywords (built here, at
                              ;; macro-expand time, and spliced as `~rtl-ctor-kw`/`~rm-ctor-kw`
                              ;; below), not runtime String values read off a constant — the
                              ;; `guarded-arm`/`shape-guarded` bodies call them directly, exactly
                              ;; as `reply-variant-kw` is already called elsewhere in this file.
                              rtl-ctor-kw   (:wat::core::keyword/from-string
                                              (:wat::string::concat proto-base
                                                (:wat::string::interpolate "::{variant-pascal}Response::RequestTooLarge" :variant-pascal variant-pascal)))
                              rm-ctor-kw    (:wat::core::keyword/from-string
                                              (:wat::string::concat proto-base
                                                (:wat::string::interpolate "::{variant-pascal}Response::RequestMalformed" :variant-pascal variant-pascal)))
                              n-sym         (:wat::core::symbol-node "n")
                              outcome-match `(:wat::core::match
                                                  (:wat::core::let ~arm-let-bindings ~body)
                                                ;; arc 278 the send'-outcome wall — a reply to a gone
                                                ;; client is NOT a service error (the client left); every
                                                ;; arm's continuation is the SAME regardless of outcome.
                                                ;;
                                                ;; ★ arc 278 #73 — EXCEPT `Stopped`, and this is the arm the
                                                ;; variant exists for. `Closed`/`Lost` are facts about ONE
                                                ;; CLIENT, so the service keeps serving the others. `Stopped`
                                                ;; is a fact about THE WORLD: the substrate is going down, and
                                                ;; a serve loop that recurses on it would spin against a
                                                ;; stopping runtime while `/stop` waits on it to return.
                                                ;; Giving this arm the same body as `Closed` would be a live
                                                ;; bug, not a tidy uniformity — it is precisely the
                                                ;; identical-arms discard the stone forbids, at every service
                                                ;; this macro will ever generate.
                                                ((:wat::service::Outcome::Reply new-state resp)
                                                  (:wat::core::match (:wat::kernel::send (:wat::core::second (:wat::core::nth selectables idx)) (~reply-variant-kw resp))
                                                    (:wat::kernel::SendOutcome::Sent   (~serve-name self l selectables next-id new-state))
                                                    (:wat::kernel::SendOutcome::Closed (~serve-name self l selectables next-id new-state))   ;; client gone → keep serving
                                                    (:wat::kernel::SendOutcome::Stopped nil)                                          ;; the WORLD is stopping → return, do not recurse
                                                    ((:wat::kernel::SendOutcome::Lost _c) (~serve-name self l selectables next-id new-state))))
                                                ((:wat::service::Outcome::Stop final-state resp)
                                                  (:wat::core::match (:wat::kernel::send (:wat::core::second (:wat::core::nth selectables idx)) (~reply-variant-kw resp))
                                                    (:wat::kernel::SendOutcome::Sent   nil)
                                                    (:wat::kernel::SendOutcome::Closed nil)   ;; client gone → still stopping
                                                    ;; uniform here and the precondition is the whole reason:
                                                    ;; this handler ALREADY decided to stop, so a stop arriving
                                                    ;; mid-reply changes nothing. Same body, stated cause.
                                                    (:wat::kernel::SendOutcome::Stopped nil)
                                                    ((:wat::kernel::SendOutcome::Lost _c) nil)))
                                                ((:wat::service::Outcome::NoReply new-state)
                                                  (~serve-name self l selectables next-id new-state))
                                                ((:wat::service::Outcome::ReplyAndArm new-state resp arms)
                                                  (:wat::core::match (:wat::kernel::send (:wat::core::second (:wat::core::nth selectables idx)) (~reply-variant-kw resp))
                                                    (:wat::kernel::SendOutcome::Sent   (~serve-name self l (:wat::core::foldl ~arm-fn selectables arms) next-id new-state))
                                                    (:wat::kernel::SendOutcome::Closed (~serve-name self l (:wat::core::foldl ~arm-fn selectables arms) next-id new-state))   ;; client gone → keep serving
                                                    ;; the world is stopping → return WITHOUT arming: arming a
                                                    ;; new selectable on the way down would register work the
                                                    ;; loop is about to abandon.
                                                    (:wat::kernel::SendOutcome::Stopped nil)
                                                    ((:wat::kernel::SendOutcome::Lost _c) (~serve-name self l (:wat::core::foldl ~arm-fn selectables arms) next-id new-state))))
                                                ((:wat::service::Outcome::NoReplyAndArm new-state arms)
                                                  (~serve-name self l (:wat::core::foldl ~arm-fn selectables arms) next-id new-state)))
                              ;; ── arc 278 — the REQUEST-MALFORMED sanitization wall (UNCONDITIONAL) ──
                              ;; The SIZE guard's sibling, in the SAME slot and for the same reason.
                              ;; `:max-request-bytes` asks "is this request too BIG?"; this asks "is
                              ;; this request the SHAPE we declared we accept?" — the whitelist is the
                              ;; op's own `<Op>Request` record, already authored; nothing new is declared.
                              ;;
                              ;; WHY HERE and nowhere else: this is POST-DECODE, inside the generated
                              ;; dispatch arm, before the handler — so it covers BOTH TIERS. A Rust-side
                              ;; decode fix would miss the thread tier entirely: `ReactorClass::InMemory`
                              ;; (src/runtime.rs) passes the `Value` through crossbeam VERBATIM and never
                              ;; decodes at all. And the process tier's decode is TAG-driven, not
                              ;; target-driven (`reconstruct_record` uses the declared fields for names
                              ;; and order only; the declared `fty` is never compared to the decoded
                              ;; value). So `#dos.Bag/PutRequest {:items [1 2 3]}` against
                              ;; `items <- (Vector :- [String])` was accepted verbatim on BOTH tiers, the
                              ;; handler used the field at its declared type, and the service DIED FOR
                              ;; EVERYONE — a second, innocent client could not even `connect'`. That is
                              ;; a denial of service, and it is what this guard pulls out by the root:
                              ;; a bad caller, malicious or dumb, cannot crash anything.
                              ;;
                              ;; UNCONDITIONAL (arc 278 Stone 2). Stone 1 shipped this behind an opt-in
                              ;; clause to stage the corpus rollout and defaulted it OFF, which left the
                              ;; denial of service live for every service in the fleet — none opted in.
                              ;; The clause is deleted; there is no gate and no default. A service asks
                              ;; for nothing and gets this. `:RequestMalformed` is correspondingly
                              ;; MANDATORY on every serviceable op-Response, checker-forced with a
                              ;; located error in `synthesize_surface_protocol` (src/types.rs) — the
                              ;; exact standing `:RequestTooLarge` has under ruling A, arrived at in the
                              ;; same order: the fleet migration first (wat-scripts/fixes/
                              ;; mandate-request-malformed.wat), THEN the contract lock.
                              ;;
                              ;; (Historical note, so nobody re-derives it: gating generation on whether
                              ;; `<Op>Response` declares the variant is IMPOSSIBLE from here. This is a
                              ;; macro; freeze runs `expand_all` (step 4) BEFORE `register_types`
                              ;; (step 5, src/freeze.rs), so at expand time the type registry holds
                              ;; NOTHING from the program being loaded — not even a surface three forms
                              ;; up in the same file. That is why the lock lives in Rust, at synthesis
                              ;; time, and this generation is unconditional.)
                              ;;
                              ;; The WHITELIST: the op's declared request record (the S1 convention
                              ;; `<proto>::<Op>Request`, the same one `op-methods` builds its client
                              ;; method's `req` param from — one convention, two consumers).
                              ;;
                              ;; Arc 278 the surface-minted op alias — NAME the alias Rust mints
                              ;; (`<Surface>::<op>/Request`) instead of guessing the message's
                              ;; name by concatenation. Stays BARE (no `<p>` suffix), and
                              ;; deliberately: it is a RUNTIME argument (`:wat::edn::validate`
                              ;; reads it, evaluates the registry lookup, and walks the DECLARED
                              ;; field types), not a type position the checker reads; and inside a
                              ;; generic `serve :- [K V]` the params are erased, so re-attaching
                              ;; `:- [K V]` would hand the walker the letters `K` and `V` — no more
                              ;; information than the bare alias name already carries.
                              ;; `edn_to_typed_value` follows a `TypeDef::Alias` unconditionally
                              ;; (src/edn_shim.rs) and treats a type-VARIABLE position as opaque,
                              ;; enforcing every concrete field around it exactly. The measured
                              ;; boundary is pinned in wat-tests/service-parametric-messages.wat,
                              ;; probes (2) and (3).
                              req-ty-kw     (:wat::core::keyword/from-string
                                              (:wat::string::concat proto-base
                                                (:wat::string::interpolate "::{op-str}/Request" :op-str op-str)))
                              ;; symbol-node binders (mirrors n-sym above) — generated pattern binders,
                              ;; never caller-visible names.
                              mpath-sym     (:wat::core::symbol-node "mpath")
                              mexp-sym      (:wat::core::symbol-node "mexpected")
                              mgot-sym      (:wat::core::symbol-node "mgot")
                              shape-guarded `(:wat::core::match (:wat::edn::validate ~req-binder ~req-ty-kw)
                                               (:wat::edn::Validation::Valid ~outcome-match)
                                               ;; arc 278 the send'-outcome wall — refuse, then RECURSE
                                               ;; INTO SERVE with state UNCHANGED (the handler never ran).
                                               ;; A gone client is not fatal either; every arm keeps serving.
                                               ((:wat::edn::Validation::Invalid ~mpath-sym ~mexp-sym ~mgot-sym)
                                                 ;; arc 278 #74 — the TWIN of the RTL strike below:
                                                 ;; `RequestMalformed` is the SAME contract
                                                 ;; (`types.rs`, `RTL_VARIANT`/`RM_VARIANT` named
                                                 ;; together), so it builds off the identical
                                                 ;; guaranteed-correct literal ctor.
                                                 (:wat::core::match (:wat::kernel::send (:wat::core::second (:wat::core::nth selectables idx))
                                                     (~reply-variant-kw (~rm-ctor-kw ~mpath-sym ~mexp-sym ~mgot-sym)))
                                                   (:wat::kernel::SendOutcome::Sent   (~serve-name self l selectables next-id state))
                                                   (:wat::kernel::SendOutcome::Closed (~serve-name self l selectables next-id state))   ;; client gone → keep serving
                                                   (:wat::kernel::SendOutcome::Stopped nil)                                    ;; arc 278 #73 — the WORLD is stopping → return
                                                   ((:wat::kernel::SendOutcome::Lost _c) (~serve-name self l selectables next-id state)))))
                              guarded-arm   `(:wat::core::let
                                                 [~n-sym (:wat::string::length (:wat::edn::write ~req-binder))]
                                               (:wat::core::if (:wat::core::i64::> ~n-sym ~cap-const-kw)
                                                 ;; arc 278 the send'-outcome wall — a gone client here is
                                                 ;; not fatal either; every arm keeps serving the rest.
                                                 (:wat::core::match (:wat::kernel::send (:wat::core::second (:wat::core::nth selectables idx))
                                                     (~reply-variant-kw (~rtl-ctor-kw ~n-sym ~cap-const-kw)))
                                                   (:wat::kernel::SendOutcome::Sent   (~serve-name self l selectables next-id state))
                                                   (:wat::kernel::SendOutcome::Closed (~serve-name self l selectables next-id state))   ;; client gone → keep serving
                                                   (:wat::kernel::SendOutcome::Stopped nil)                                    ;; arc 278 #73 — the WORLD is stopping → return
                                                   ((:wat::kernel::SendOutcome::Lost _c) (~serve-name self l selectables next-id state)))
                                                 ~shape-guarded))]
                             (:wat::core::conj acc
                               `((~op-variant-kw ~req-binder) ~guarded-arm))))))
                     (:wat::core::Vector :wat::WatAST)
                     impl-clauses)

     ;; ── serve params argvec ───────────────────────────────────────────────────────
     ;; Template is a Vector node; checker does NOT recurse into Vector children.
     ;; self/l/clients/state in the Vector are fine as literal symbols.
     ;; arc 278 the call context — `next-id` is the NEW 5th param: the stable monotonic
     ;; conn-id counter, threaded as pure state (no clock, no entropy, no global — the
     ;; ONE pinned contract decision). `selectables` is now `selectable-entry-vec-ty`
     ;; ((Tuple :- [i64 Peer]) — the id travels WITH its peer, STOP-2), not the bare peer vector.
     serve-params `[self        <- ~lineage-peer-ty
                    l           <- ~listener-ty
                    selectables <- ~selectable-entry-vec-ty
                    next-id     <- :wat::core::i64
                    state       <- ~state-ty-ann]

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
     ;; arc 278 the call context — `poll'`/`serve-dispatch-op'` are Rust intrinsics that
     ;; downcast every Vector element to a real Peer opaque; `selectables` is now the
     ;; Tuple-wrapped canonical vector, so BOTH calls receive `~peers-only-expr` — the
     ;; bare-peer projection built fresh from `selectables` this iteration — never
     ;; `selectables` itself. The returned `idx` is a POSITION, valid identically against
     ;; either vector (the projection preserves order 1:1), so every other `idx` use below
     ;; (nth/remove-at against the CANONICAL `selectables`) is unaffected.
     serve-body   `(:wat::core::match (:wat::kernel::poll self l ~peers-only-expr)
                     (:wat::spawn::ServiceEvent::Shutdown nil)
                     ((:wat::spawn::ServiceEvent::Connection peer)
                       ;; mint THIS connection's id = the current next-id (pre-increment);
                       ;; pair it with the peer so it travels together from birth (STOP-2);
                       ;; the recursive call's OWN next-id is next-id+1, for the NEXT connect.
                       (~serve-name self l (:wat::core::conj selectables (:wat::core::Tuple next-id peer)) (:wat::core::i64::+ next-id 1) state))
                     ((:wat::spawn::ServiceEvent::Admin admin-msg)
                       (:wat::core::match admin-msg 
                         ;; arc 278 the send'-outcome wall — the owner's `recv'` (the `/stop`
                         ;; method's own recv') faces a gone-owner outcome on its side; this
                         ;; send' terminates the loop regardless (all arms → nil).
                         (~admin-stop-kw
                           (:wat::core::match (:wat::kernel::send self (~status-stopped-kw (~stop-project-name state)))
                             (:wat::kernel::SendOutcome::Sent   nil)
                             (:wat::kernel::SendOutcome::Closed nil)   ;; owner's recv' already faces this
                             (:wat::kernel::SendOutcome::Stopped nil)  ;; arc 278 #73 — same, and the owner's recv' faces the stop too
                             ((:wat::kernel::SendOutcome::Lost _c) nil)))
                         (~admin-hibernate-kw
                           (:wat::core::match (:wat::kernel::send self (~status-hibernated-kw (~hibernate-project-name state)))
                             (:wat::kernel::SendOutcome::Sent   nil)
                             (:wat::kernel::SendOutcome::Closed nil)   ;; owner's recv' already faces this
                             (:wat::kernel::SendOutcome::Stopped nil)  ;; arc 278 #73 — same, and the owner's recv' faces the stop too
                             ((:wat::kernel::SendOutcome::Lost _c) nil)))
                         ;; arc 278: AllowPeer[pids] — fold (allow' l pid) over the vec on the
                         ;; serve loop's OWN listener l (process-tier gate), ack PeersAllowed up
                         ;; the lineage peer (request/reply — owner blocks so grant-before-dial
                         ;; ordering holds), then CONTINUE serving (recur — no state change).
                         ((~admin-allow-peer-kw pids)
                           (:wat::core::do
                             (:wat::core::foldl
                               (:wat::core::fn [~allow-acc-sym <- :wat::core::nil
                                                ~allow-pid-sym <- :wat::core::i64] -> :wat::core::nil
                                 (:wat::kernel::allow l ~allow-pid-sym))
                               nil
                               pids)
                             ;; arc 278 the send'-outcome wall — the owner's `/grant` recv'
                             ;; faces a gone-owner outcome on its side; the serve loop always
                             ;; continues serving regardless of this ack's outcome.
                             (:wat::core::match (:wat::kernel::send self ~status-peers-allowed-kw)
                               (:wat::kernel::SendOutcome::Sent   (~serve-name self l selectables next-id state))
                               (:wat::kernel::SendOutcome::Closed (~serve-name self l selectables next-id state))   ;; owner's recv' already faces this
                               (:wat::kernel::SendOutcome::Stopped nil)                                     ;; arc 278 #73 — the WORLD is stopping → return
                               ((:wat::kernel::SendOutcome::Lost _c) (~serve-name self l selectables next-id state)))))
                         ;; arc 293: DenyPeer[pids] — mirror, fold (deny' l pid) over the vec on
                         ;; the serve loop's OWN listener l (process-tier gate), ack PeersDenied up
                         ;; the lineage peer (request/reply — owner blocks so revoke-before-return
                         ;; ordering holds), then CONTINUE serving (recur — no state change).
                         ((~admin-deny-peer-kw pids)
                           (:wat::core::do
                             (:wat::core::foldl
                               (:wat::core::fn [~deny-acc-sym <- :wat::core::nil
                                                ~deny-pid-sym <- :wat::core::i64] -> :wat::core::nil
                                 (:wat::kernel::deny l ~deny-pid-sym))
                               nil
                               pids)
                             ;; arc 278 the send'-outcome wall — the owner's `/revoke` recv'
                             ;; faces a gone-owner outcome on its side; the serve loop always
                             ;; continues serving regardless of this ack's outcome.
                             (:wat::core::match (:wat::kernel::send self ~status-peers-denied-kw)
                               (:wat::kernel::SendOutcome::Sent   (~serve-name self l selectables next-id state))
                               (:wat::kernel::SendOutcome::Closed (~serve-name self l selectables next-id state))   ;; owner's recv' already faces this
                               (:wat::kernel::SendOutcome::Stopped nil)                                     ;; arc 278 #73 — the WORLD is stopping → return
                               ((:wat::kernel::SendOutcome::Lost _c) (~serve-name self l selectables next-id state)))))
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
                     ;; arc 278 RST stone — the op-dispatch is wrapped in `serve-dispatch-op'`
                     ;; instead of a bare match: it is the ONE hook that can reach `clients`
                     ;; while a handler panic is caught (the interpreter's own catch_unwind,
                     ;; inserted around THIS form's evaluation, not the top-level one that has
                     ;; already lost `clients` by the time a panic reaches it). On a genuine
                     ;; handler panic it best-effort broadcasts PeerCrashed to `clients`, then
                     ;; resumes the SAME panic unchanged — the service still crashes exactly as
                     ;; before; this arm's own Reply/Stop behavior (the match body) is untouched.
                     ((:wat::spawn::ServiceEvent::Message idx op)
                       (:wat::kernel::serve-dispatch-op ~peers-only-expr
                         ;; Arc 278 the parametric protocol — TYPE-position spellings on both
                         ;; sides: `infer_retag_op` reads arg[2] as this form's RESULT TYPE, and
                         ;; the arms below dispatch over the instantiated `(<service>::Op :- [K V])`.
                         ;; `eval_retag_op` canonicalizes both to their base names (params are
                         ;; erased in a runtime `type_path`). Monomorphic ⇒ unchanged.
                         (:wat::core::match (:wat::kernel::retag-op op ~proto-op-ty-ann ~service-op-decl-kw-runtime)
                           ~@serve-op-arms)))
                     ((:wat::spawn::ServiceEvent::Closed idx)
                       (~serve-name self l (:wat::std::list::remove-at selectables idx) next-id state))
                     ;; arc 278 no-hidden-failures — a peer that broke abnormally is GONE:
                     ;; evict it. But its `cause` must NOT vanish (the old `_cause` silently
                     ;; swallowed the death reason — the exact masking this arc forbids). There
                     ;; is no reply target (the peer is dead) and the lineage peer is a
                     ;; request/reply admin channel we must not desync, so surface the reason on
                     ;; the honest loud sink (stderr) BEFORE evicting + continuing to serve. The
                     ;; RICHER client-facing recv'-EOF crash-reason surfacing is a separate
                     ;; follow-on strike; this arm's contract here is simply: do not DROP it.
                     ((:wat::spawn::ServiceEvent::Lost idx cause)
                       (:wat::core::do
                         (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None)
                         (~serve-name self l (:wat::std::list::remove-at selectables idx) next-id state)))
                     ;; arc 278 no-hidden-failures — a peer that sent an UNDECODABLE message is
                     ;; STILL ALIVE (a bad message is not a death). Reply the rich decode reason
                     ;; to THAT client as `Reply::Failed[cause]` (its generated method raises with
                     ;; the reason — the caller is never left blind), and KEEP THE CLIENT + keep
                     ;; serving (recur; do NOT remove-at — one client's garbage must never kill a
                     ;; shared service, the DoS this arc pulls out by the root).
                     ((:wat::spawn::ServiceEvent::Malformed idx cause)
                       ;; arc 278 the send'-outcome wall — a gone client here is not fatal
                       ;; either (same "reply to a gone client" doctrine); keep serving.
                       (:wat::core::match (:wat::kernel::send (:wat::core::second (:wat::core::nth selectables idx)) (~reply-failed-kw cause))
                         (:wat::kernel::SendOutcome::Sent   (~serve-name self l selectables next-id state))
                         (:wat::kernel::SendOutcome::Closed (~serve-name self l selectables next-id state))   ;; client gone → keep serving
                         (:wat::kernel::SendOutcome::Stopped nil)                                     ;; arc 278 #73 — the WORLD is stopping → return
                         ((:wat::kernel::SendOutcome::Lost _c) (~serve-name self l selectables next-id state))))
                     ;; arc 278 Stone 1a — a client sent an OVER-FOO frame (exceeded this
                     ;; service's declared max-frame-bytes). A bad request is a 400: TELL that
                     ;; client (reply `Reply::Failed[cause]` — its generated method raises with the
                     ;; reason, so the caller is never left blind), then EVICT (close) that ONE
                     ;; connection (discarding the un-read oversized residual that would otherwise
                     ;; desync the wire — this is why Malformed, which KEEPS the client, is wrong
                     ;; here), then KEEP SERVING everyone else. The reply is a NON-BLOCKING
                     ;; `try-send'` (the deadlock guard): a client blocked mid-send on an extreme
                     ;; oversized frame is not reading its reply side, so a blocking send' could
                     ;; wedge the serve loop — try-send' skips a non-draining client and we still
                     ;; evict (it learns via EPIPE on its own send). NO eprintln (that is wat's
                     ;; panic — a client-triggerable crash / DoS).
                     ((:wat::spawn::ServiceEvent::Rejected idx cause)
                       (:wat::core::do
                         (:wat::core::match (:wat::kernel::try-send (:wat::core::second (:wat::core::nth selectables idx)) (~reply-failed-kw cause))
                           (:wat::kernel::TrySendOutcome::Sent       nil)
                           (:wat::kernel::TrySendOutcome::WouldBlock nil)   ;; client not draining — evict anyway (it learns via EPIPE)
                           (:wat::kernel::TrySendOutcome::Closed     nil)
                           ((:wat::kernel::TrySendOutcome::Lost _c)  nil))
                         (~serve-name self l (:wat::std::list::remove-at selectables idx) next-id state))))

     ;; ── Arc 293 S2: client methods for :impls (over the surface's protocol) ─────────────
     ;; `(defn <fqdn>/<op> [c <- (Peer :- [S::Op S::Reply])  req <- <S>::<Op>Request] -> <S>::<Op>Response
     ;;    (let [_ (send' c (<S>::Op::<Op> req))  r (recv' c)]
     ;;      (match r ((<S>::Reply::<Op> resp) resp) …)))
     ;; The client fn is SERVICE-namespaced (<fqdn>/<op>) — the SURFACE-namespaced name <S>/<op>
     ;; is already the surface's method-dispatch stub (defsurface registers it; receiver = a Store
     ;; satisfier). The blind/uniform side is the shared Op/Reply protocol + (Address :- [S::Op S::Reply])
     ;; type; the surface method <S>/<op> becomes the blind entry once a satisfier extend-type wires
     ;; it to this concrete client fn (S4). Request/response records are the surface's own
     ;; (user-declared `<S>::<Op>Request` / `<S>::<Op>Response` — the S1/gRPC naming convention).
     op-methods    (:wat::core::foldl
                     (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                      clause <- :wat::WatAST]
                       -> (:wat::core::Vector :- [:wat::WatAST])
                       (:wat::core::let
                         [ch              (:wat::core::ast->children clause)
                          op-node         (:wat::core::first ch)
                          op-str          (:wat::core::ast-name op-node)
                          ;; Arc 278 Stone 2-A — an internal (leading-dash) op is NOT on the
                          ;; surface: NO client method (a client can only call surface ops).
                          is-internal     (:wat::string::starts-with? op-str "-")
                          op-pascal       (:wat::string::kebab->pascal-in surface-kw op-str)
                          ;; Arc 278 the parametric protocol — the client fn's SIGNATURE names the
                          ;; surface's parametric messages, so the DECLARATION carries the service's
                          ;; own binders, exactly as `/start`, `/stop`, `/grant` do. STONE-the-last-mint
                          ;; — `fqdn-tp` (the `<a,b>` angle-string mint) is RETIRED, mirroring
                          ;; `proto-tp`/`launch-head-kw`: `method-name` is now the BARE name; a
                          ;; non-empty `fqdn-tp-syms` splices `:- [~@fqdn-tp-syms]` as DECLARATION
                          ;; SIBLINGS in the emitted `defn`, below, the same shape `record-def`/
                          ;; `state-def`/`service-op-def` already use. A concrete service satisfying
                          ;; a surface at concrete args has an empty `fqdn-tp-syms` and a fully
                          ;; concrete signature — no binder to declare, nothing changes.
                          method-name     (:wat::core::keyword/from-string
                                            (:wat::string::interpolate "{b}/{op-str}"
                                              :b fqdn-base :op-str op-str))
                          ;; Arc 278 the surface-minted op alias — NAME the alias Rust mints
                          ;; (`<Surface>::<op>/Request` / `/Response`) instead of guessing the
                          ;; message's name by concatenation; `proto-args` re-attaches the alias's
                          ;; own type args structurally, exactly as `method-name`'s DECLARATION
                          ;; binder above re-attaches `fqdn-tp-syms`.
                          ;; identity 2c: req-ty (this scope) is ANNOTATION-only (client method's
                          ;; `req` param type) — mints the reference FORM. Arc 109 ③ —
                          ;; structurally off `proto-args`, not the retired `proto-tp` string.
                          ;; STONE-exactly-one-call-position — UNCONDITIONAL (see the note beside
                          ;; `proto-op-ty-ann` above): `(Head :- [])` IS `Head` now.
                          req-base-kw     (:wat::core::keyword-node
                                            (:wat::string::concat ":"
                                              (:wat::string::interpolate "{b}::{op-str}/Request"
                                                :b proto-base :op-str op-str)))
                          req-ty          `(~req-base-kw :- [~@proto-args])
                          ;; NOTE: named `client-resp-ty`, NOT `resp-ty` — the outer macro scope
                          ;; already binds `resp-ty` (the :stop projection's return type,
                          ;; `:wat::core::nth stop-fn-ch 3`, used well below); a same-named local
                          ;; here would shadow it silently inside this nested `let`.
                          client-resp-base-kw (:wat::core::keyword-node
                                            (:wat::string::concat ":"
                                              (:wat::string::interpolate "{b}::{op-str}/Response"
                                                :b proto-base :op-str op-str)))
                          client-resp-ty  `(~client-resp-base-kw :- [~@proto-args])
                          ;; arc 278 the recv'-outcome wall — the CLIENT-FACING return type is
                          ;; `(RecvOutcome :- [<Op>Response])` (a matchable value, never a raise).
                          ;; identity 2c: ANNOTATION-only (client method's return type) — mints
                          ;; the reference FORM, structurally off `client-resp-ty` above.
                          recv-ret-ty     `(:wat::kernel::RecvOutcome :- [~client-resp-ty])
                          op-variant-kw   (:wat::core::keyword/from-string
                                            (:wat::string::concat proto-base
                                              (:wat::string::interpolate "::Op::{op-pascal}" :op-pascal op-pascal)))
                          reply-variant-kw (:wat::core::keyword/from-string
                                             (:wat::string::concat proto-base
                                               (:wat::string::interpolate "::Reply::{op-pascal}" :op-pascal op-pascal)))
                          method-params   `[c <- ~client-peer-ty req <- ~req-ty]
                          discard-sym     (:wat::core::symbol-node "_")
                          r-sym           (:wat::core::symbol-node "r")
                          ;; DESIGN-STONE-the-client-validates-locally.md — the client checks the
                          ;; same surface-scoped budget constant `serve-op-arms` checks
                          ;; (`build_op_budget_constants`, src/types.rs:3041), built the IDENTICAL
                          ;; way (`<proto-base>::<OP-UPPER>-MAX-REQUEST-BYTES`) so the two never
                          ;; drift apart.
                          ;;
                          ;; arc 278 #74 — `<Op>Response` is LAW (builder ruling, 2026-08-05),
                          ;; checker-enforced at `defsurface` registration
                          ;; (`synthesize_surface_protocol`, src/types.rs): a serviceable op's
                          ;; response type is REQUIRED to be `<op-pascal>Response`, so
                          ;; `rtl-ctor-kw` below is a LITERAL ctor keyword (built here, at
                          ;; macro-expand time, guaranteed correct by construction) rather than a
                          ;; runtime String read off a constant — no EDN decode needed.
                          n-sym           (:wat::core::symbol-node "n")
                          op-upper        (:wat::string::to-uppercase op-str)
                          cap-const-kw    (:wat::core::keyword/from-string
                                            (:wat::string::concat proto-base
                                              (:wat::string::interpolate "::{op-upper}-MAX-REQUEST-BYTES" :op-upper op-upper)))
                          rtl-ctor-kw     (:wat::core::keyword/from-string
                                            (:wat::string::concat proto-base
                                              (:wat::string::interpolate "::{op-pascal}Response::RequestTooLarge" :op-pascal op-pascal)))
                          ;; arc 278 the recv'-outcome wall — recv' returns a matchable
                          ;; (RecvOutcome :- [Reply]), never a raise. This client method RE-WRAPS it into a
                          ;; `(RecvOutcome :- [<Op>Response])` the caller faces as a VALUE (we are ADT; no
                          ;; try/catch, no raise). CLIENT role: on ::Message, unwrap the reply variant
                          ;; to its Response and re-wrap as ::Message (the inner `_` arm stays for a
                          ;; GENUINE misroute — an off-protocol variant — which IS a real protocol
                          ;; violation, so it still raises). On ::Lost, map to a REASON-FREE ::Lost
                          ;; (arc-294 client = reason-free 500 — the client never gets the service's
                          ;; internal reason; the full cause is the owner's, on its crash channel), a
                          ;; fresh reason-free Failure (mirrors runtime::message_only_failure's shape).
                          ;; The reserved protocol-tier `Reply::Failed` (a decode failure) arrives as
                          ;; ::Lost too (recv' maps it), so it never reaches the inner reply match.
                          ;; On ::Closed, pass the reason-free terminal through.
                          ;;
                          ;; arc 278 the send'-outcome wall — a send-then-recv': the recv' right
                          ;; below faces Lost/Closed as a real outcome, so this send' just needs to
                          ;; proceed regardless (faced, not `_`-swallowed). UNDER budget (or no wire
                          ;; to measure against — STOP-3) reaches this form, and ONLY this form.
                          send-recv-form  `(:wat::core::let
                                             [~discard-sym (:wat::core::match (:wat::kernel::send c (~op-variant-kw req))
                                                             (:wat::kernel::SendOutcome::Sent   nil)
                                                             (:wat::kernel::SendOutcome::Closed nil)
                                                             ;; arc 278 #73 — uniform, and the precondition is
                                                             ;; the recv' on the very next line: a stop that
                                                             ;; interrupted this write is still in force when
                                                             ;; the read parks, so the read returns Stopped and
                                                             ;; the caller is told once, by the arm below.
                                                             ;; Deciding here would decide it twice.
                                                             (:wat::kernel::SendOutcome::Stopped nil)
                                                             ((:wat::kernel::SendOutcome::Lost _c) nil))
                                              ~r-sym (:wat::kernel::recv c)]
                                             (:wat::core::match ~r-sym
                                               ((:wat::kernel::RecvOutcome::Message recvd)
                                                 (:wat::kernel::RecvOutcome::Message
                                                   (:wat::core::match recvd
                                                     ((~reply-variant-kw resp) resp)
                                                     (_ (:wat::kernel::assertion-failed!
                                                          "defservice method: misrouted reply variant (protocol violation)"
                                                          :wat::core::None
                                                          :wat::core::None)))))
                                               ;; arc 278 the LociDiedError stone — forward the real
                                               ;; loci-agnostic death cause to the client's caller
                                               ;; (no-hidden-failures: never mask it with a generic
                                               ;; message-only Failure). RecvOutcome::Lost now carries
                                               ;; a :wat::kernel::LociDiedError, which `cause` already is.
                                               ((:wat::kernel::RecvOutcome::Lost cause)
                                                 (:wat::kernel::RecvOutcome::Lost cause))
                                               ;; ★ arc 278 #73 — THE CLIENT-FACING PAYOFF. This generated
                                               ;; method is what every caller of every service actually
                                               ;; holds, and until today a stop reached them through the
                                               ;; `Lost` arm above carrying `LociDiedError::Stopped`: the
                                               ;; caller matched "the service died" over a service that was
                                               ;; alive and merely being asked to stop. Forwarded faithfully
                                               ;; now, as itself.
                                               (:wat::kernel::RecvOutcome::Stopped
                                                 :wat::kernel::RecvOutcome::Stopped)
                                               (:wat::kernel::RecvOutcome::Closed
                                                 :wat::kernel::RecvOutcome::Closed)))
                          ;; DESIGN-STONE-the-client-validates-locally.md — THE STRIKE. Validation
                          ;; and dispatch are ONE operation (botocore validates before the HTTP call
                          ;; is ever made): over budget → the SAME RequestTooLarge{bytes,cap} a
                          ;; server would have sent, with NO send and therefore NO recv (STOP-2: the
                          ;; two arms reach DIFFERENT places, never a uniform fall-through). STOP-3:
                          ;; `peer-wire?` gates the measure+guard entirely behind "is there a wire" —
                          ;; a thread-tier `c` never reaches `:wat::edn::write` at all, so it pays
                          ;; nothing (not a redundant encode, ZERO encodes, exactly as today).
                          ;; STOP-4: the cap checked here is `:max-request-bytes` (the CONTRACT, both
                          ;; sides know it) — never `:max-frame-bytes` (FOO, the deployment's, unknown
                          ;; to a dialer and possibly stricter); a FOO violation is unreachable from
                          ;; here (it can only fire once a frame has actually left for the wire) and
                          ;; stays the server's own dismissal.
                          method-body     `(:wat::core::if (:wat::kernel::peer-wire? c)
                                             (:wat::core::let [~n-sym (:wat::string::length (:wat::edn::write req))]
                                               (:wat::core::if (:wat::core::i64::> ~n-sym ~cap-const-kw)
                                                 (:wat::kernel::RecvOutcome::Message (~rtl-ctor-kw ~n-sym ~cap-const-kw))
                                                 ~send-recv-form))
                                             ~send-recv-form)]
                         (:wat::core::if is-internal

                           acc
                           (:wat::core::conj acc
                             ;; STONE-the-last-mint — same siblings-binder splice as `record-def`/
                             ;; `state-def`/`service-op-def`, over `fqdn-tp-syms`.
                             (:wat::core::if (:wat::core::empty? fqdn-tp-syms)
                               `(:wat::core::defn ~method-name ~method-params -> ~recv-ret-ty ~method-body)
                               `(:wat::core::defn ~method-name :- [~@fqdn-tp-syms] ~method-params -> ~recv-ret-ty ~method-body))))))
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
                         (:wat::string::interpolate "{b}/stop" :b fqdn-base))
     handle-handle-acc (:wat::core::keyword/from-string
                         (:wat::string::interpolate "{b}::Handle/handle" :b fqdn-base))
     stop-method-params `[h <- ~handle-bare-name]
     stop-method-body  `(:wat::core::let
                          ;; arc 278 the send'-outcome wall — a send-then-recv': the recv' right
                          ;; below faces Lost/Closed; the send' just proceeds regardless.
                          [~stop-discard-sym (:wat::core::match (:wat::kernel::send (~handle-handle-acc h) ~admin-stop-kw)
                                               (:wat::kernel::SendOutcome::Sent   nil)
                                               (:wat::kernel::SendOutcome::Closed nil)
                                               (:wat::kernel::SendOutcome::Stopped nil)   ;; arc 278 #73 — the recv' below faces it
                                               ((:wat::kernel::SendOutcome::Lost _c) nil))
                           ~stop-r-sym       (:wat::kernel::recv (~handle-handle-acc h))]
                          (:wat::core::match ~stop-r-sym 
                            ((:wat::kernel::RecvOutcome::Message recvd)
                              (:wat::core::match recvd 
                                ((~status-stopped-kw resp) resp)
                                (_ (:wat::kernel::assertion-failed!
                                     "defservice stop: expected Status::Stopped"
                                     :wat::core::None
                                     :wat::core::None))))
                            ;; arc 278 the recv'-outcome wall — OWNER role: eprintln the cause
                            ;; (loud, terminal; the owner is the real final caller who does not
                            ;; recover — R51 eprintln IS the dying declaration), then terminate.
                            ((:wat::kernel::RecvOutcome::Lost cause)
                              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Stopped
                              (:wat::kernel::assertion-failed!
                                "defservice stop: stop requested while awaiting the reply — the service was ALIVE (arc 278 #73; this was reported as a peer close before the variant existed)"
                                :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Closed
                              (:wat::kernel::assertion-failed!
                                "defservice stop: service peer closed during stop"
                                :wat::core::None :wat::core::None))))
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
                             (:wat::string::interpolate "{b}/hibernate" :b fqdn-base))
     hibernate-method-params `[h <- ~handle-bare-name]
     hibernate-method-body  `(:wat::core::let
                               ;; arc 278 the send'-outcome wall — a send-then-recv': the recv'
                               ;; right below faces Lost/Closed; the send' just proceeds regardless.
                               [~hib-discard-sym (:wat::core::match (:wat::kernel::send (~handle-handle-acc h) ~admin-hibernate-kw)
                                                   (:wat::kernel::SendOutcome::Sent   nil)
                                                   (:wat::kernel::SendOutcome::Closed nil)
                                                   (:wat::kernel::SendOutcome::Stopped nil)   ;; arc 278 #73 — the recv' below faces it
                                                   ((:wat::kernel::SendOutcome::Lost _c) nil))
                                ~hib-r-sym       (:wat::kernel::recv (~handle-handle-acc h))]
                               (:wat::core::match ~hib-r-sym 
                                 ((:wat::kernel::RecvOutcome::Message recvd)
                                   (:wat::core::match recvd 
                                     ((~status-hibernated-kw snapshot) snapshot)
                                     (_ (:wat::kernel::assertion-failed!
                                          "defservice hibernate: expected Status::Hibernated"
                                          :wat::core::None
                                          :wat::core::None))))
                                 ((:wat::kernel::RecvOutcome::Lost cause)
                                   (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                                 (:wat::kernel::RecvOutcome::Stopped
                                   (:wat::kernel::assertion-failed!
                                     "defservice hibernate: stop requested while awaiting the reply — the service was ALIVE (arc 278 #73; this was reported as a peer close before the variant existed)"
                                     :wat::core::None :wat::core::None))
                                 (:wat::kernel::RecvOutcome::Closed
                                   (:wat::kernel::assertion-failed!
                                     "defservice hibernate: service peer closed during hibernate"
                                     :wat::core::None :wat::core::None))))
     hibernate-method  `(:wat::core::defn ~hibernate-method-name ~hibernate-method-params -> ~record-ty-ann ~hibernate-method-body)
     ;; Extend methods with the owner-only hibernate (stop + hibernate, not per-op).
     methods           (:wat::core::conj methods hibernate-method)

     ;; ── arc 278: owner-only grant method (mirror of stop) ────────────────────────
     ;; Method: (defn <fqdn>/grant [h <- Handle  pids <- (Vector i64)] -> nil ...)
     ;; Takes the Handle (unforgeable; never handed to clients — clients hold only a client
     ;; Peer, so a client has NO grant path). Sends Admin::AllowPeer[pids] down the lineage
     ;; peer; recv's Status::PeersAllowed → the grant is applied before this returns (so the
     ;; circuit builder's post-spawn grant lands before the caller dials). Callable any time,
     ;; repeatedly, mid-life. Uses symbol-node for `_`/`r` binders (hygiene: Unquote at def time).
     grant-discard-sym (:wat::core::symbol-node "_")
     grant-r-sym       (:wat::core::symbol-node "r")
     grant-method-name (:wat::core::keyword/from-string
                         (:wat::string::interpolate "{b}/grant" :b fqdn-base))
     ;; the BASE call name — the Capability/Dialable extend-type bodies invoke grant/revoke
     ;; with the receiver's own T already bound, so they name the bare fn (no turbofish).
     grant-call-name   (:wat::core::keyword/from-string
                         (:wat::string::interpolate "{b}/grant" :b fqdn-base))
     grant-method-params `[h <- ~handle-bare-name  pids <- (:wat::core::Vector :wat::core::i64)]
     ;; Grant is the process-tier accept-gate. Hinge is the existing
     ;; `peer-process` on the lineage handle (same un-erase stop/signal use).
     ;; Thread is shared memory: the handle IS the grant — no Admin::AllowPeer.
     grant-method-body `(:wat::core::match (:wat::kernel::peer-process (~handle-handle-acc h))
                          ((:wat::core::Some _)
                            (:wat::core::let
                          ;; arc 278 the send'-outcome wall — a send-then-recv': the recv' right
                          ;; below faces Lost/Closed; the send' just proceeds regardless.
                          [~grant-discard-sym (:wat::core::match (:wat::kernel::send (~handle-handle-acc h) (~admin-allow-peer-kw pids))
                                                (:wat::kernel::SendOutcome::Sent   nil)
                                                (:wat::kernel::SendOutcome::Closed nil)
                                                (:wat::kernel::SendOutcome::Stopped nil)   ;; arc 278 #73 — the recv' below faces it
                                                ((:wat::kernel::SendOutcome::Lost _c) nil))
                           ~grant-r-sym       (:wat::kernel::recv (~handle-handle-acc h))]
                          (:wat::core::match ~grant-r-sym 
                            ((:wat::kernel::RecvOutcome::Message recvd)
                              (:wat::core::match recvd 
                                (~status-peers-allowed-kw nil)
                                (_ (:wat::kernel::assertion-failed!
                                     "defservice grant: expected Status::PeersAllowed"
                                     :wat::core::None
                                     :wat::core::None))))
                            ((:wat::kernel::RecvOutcome::Lost cause)
                              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Stopped
                              (:wat::kernel::assertion-failed!
                                "defservice grant: stop requested while awaiting the reply — the service was ALIVE (arc 278 #73; this was reported as a peer close before the variant existed)"
                                :wat::core::None :wat::core::None))
                            (:wat::kernel::RecvOutcome::Closed
                              (:wat::kernel::assertion-failed!
                                "defservice grant: service peer closed during grant"
                                :wat::core::None :wat::core::None)))))
                          (:wat::core::None nil))
     grant-method      `(:wat::core::defn ~grant-method-name ~grant-method-params -> :wat::core::nil ~grant-method-body)
     ;; Extend methods with the owner-only grant (stop + hibernate + grant, not per-op).
     methods           (:wat::core::conj methods grant-method)

     ;; ── arc 293: owner-only revoke method (mirror of grant) ──────────────────────
     ;; Method: (defn <fqdn>/revoke [h <- Handle  pids <- (Vector i64)] -> nil ...)
     ;; Takes the Handle (unforgeable; never handed to clients — clients hold only a client
     ;; Peer, so a client has NO revoke path). Sends Admin::DenyPeer[pids] down the lineage
     ;; peer; recv's Status::PeersDenied → the revoke is applied before this returns. Callable
     ;; any time, repeatedly, mid-life. Uses symbol-node for `_`/`r` binders (hygiene: Unquote
     ;; at def time).
     revoke-discard-sym (:wat::core::symbol-node "_")
     revoke-r-sym       (:wat::core::symbol-node "r")
     revoke-method-name (:wat::core::keyword/from-string
                          (:wat::string::interpolate "{b}/revoke" :b fqdn-base))
     revoke-call-name   (:wat::core::keyword/from-string
                          (:wat::string::interpolate "{b}/revoke" :b fqdn-base))
     revoke-method-params `[h <- ~handle-bare-name  pids <- (:wat::core::Vector :wat::core::i64)]
     ;; Twin of grant: process-only via `peer-process`. Shared-memory lineage
     ;; has no pid set to revoke.
     revoke-method-body `(:wat::core::match (:wat::kernel::peer-process (~handle-handle-acc h))
                           ((:wat::core::Some _)
                             (:wat::core::let
                           ;; arc 278 the send'-outcome wall — a send-then-recv': the recv' right
                           ;; below faces Lost/Closed; the send' just proceeds regardless.
                           [~revoke-discard-sym (:wat::core::match (:wat::kernel::send (~handle-handle-acc h) (~admin-deny-peer-kw pids))
                                                  (:wat::kernel::SendOutcome::Sent   nil)
                                                  (:wat::kernel::SendOutcome::Closed nil)
                                                  (:wat::kernel::SendOutcome::Stopped nil)   ;; arc 278 #73 — the recv' below faces it
                                                  ((:wat::kernel::SendOutcome::Lost _c) nil))
                            ~revoke-r-sym       (:wat::kernel::recv (~handle-handle-acc h))]
                           (:wat::core::match ~revoke-r-sym 
                             ((:wat::kernel::RecvOutcome::Message recvd)
                               (:wat::core::match recvd 
                                 (~status-peers-denied-kw nil)
                                 (_ (:wat::kernel::assertion-failed!
                                      "defservice revoke: expected Status::PeersDenied"
                                      :wat::core::None
                                      :wat::core::None))))
                             ((:wat::kernel::RecvOutcome::Lost cause)
                               (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                             (:wat::kernel::RecvOutcome::Stopped
                               (:wat::kernel::assertion-failed!
                                 "defservice revoke: stop requested while awaiting the reply — the service was ALIVE (arc 278 #73; this was reported as a peer close before the variant existed)"
                                 :wat::core::None :wat::core::None))
                             (:wat::kernel::RecvOutcome::Closed
                               (:wat::kernel::assertion-failed!
                                 "defservice revoke: service peer closed during revoke"
                                 :wat::core::None :wat::core::None)))))
                           (:wat::core::None nil))
     revoke-method      `(:wat::core::defn ~revoke-method-name ~revoke-method-params -> :wat::core::nil ~revoke-method-body)
     ;; Extend methods with the owner-only revoke (stop + hibernate + grant + revoke, not per-op).
     methods           (:wat::core::conj methods revoke-method)

     ;; ── host-parity-4a: locus-agnostic start fn ──────────────────────────────────
     ;; (defn <fqdn>/start [locus <- :wat::spawn::Locus  state0 <- <state-ty>] -> <fqdn>::Handle
     ;;   (let [b    (listener' locus Op Reply)              ; listener' accepts an abstract :Locus
     ;;         l    (Bound/listener b)
     ;;         addr (Bound/address b)
     ;;         svc  (:wat::spawn::Locus/launch locus l (Vector (Peer :- [Reply Op])) state0
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
     ;;   `lr` is the let binder for the (Launched :- [S R]) value → symbol-node.
     ;;   `locus`, `state0` are value references (start's params) → fine as literals.
     ;; start-params `[locus <- :Locus  state0 <- ~state-ty]` → Vector inner → checker skips it.
     ;;
     ;; arc 272 6b-ii-β: listener-minting moved INTO Locus/launch (child-mints for process tier).
     ;; start calls `(Locus/launch :- [Op Reply …] …)` with EXPLICIT type-args (arc-232 dep) so
     ;; the impl's (listener' self :S :R) resolves S=Op, R=Reply. STONE-exactly-one-call-position —
     ;; the head is the bare keyword; the binder rides as a call-site sibling, not name-embedded.
     ;; launch returns (Launched :- [Op Reply]){handle,address}; start unwraps into Handle.
     lr-sym        (:wat::core::symbol-node "lr")
     ;; arc 170 closure #6 — binds `(:wat::kernel::call-site)` once in start/resume's own
     ;; body, so the ps label carries the CALLER's source position (which start brought this
     ;; process up), not this macro's fixed position in service.wat.
     origin-sym    (:wat::core::symbol-node "origin")
     ;; arc 291 kwargs-start: locus-sym minted once so start-params + start-body (and resume pair)
     ;; share the same scope node — avoids HygieneScopeDivergence when kwargs-defn rebuilds $impl.
     locus-sym     (:wat::core::symbol-node "locus")
     ;; arc 291 3a-ii-β: `(launch :- [Op Reply State Admin Status] …)` — Sh=Admin (ship), Lu=Status.
     ;; Arc 293 S2 — Op/Reply are the protocol's (proto-str); State/Admin/Status stay per-service.
     ;; STONE-exactly-one-call-position — LANDED: `Locus/launch` is a SURFACE-METHOD call
     ;; (`:S/method`, check.rs's `k.contains('/')` arm), which now peels a position-4 `:-`
     ;; binder from `args` exactly as the generic call arm (`69933d362`) does — the peel is
     ;; hoisted above BOTH arms' dispatch, so "the call position" is taught once. The head is
     ;; therefore the BARE keyword; `:- [...]` rides as call-site siblings (below, at each of
     ;; `launch-head-kw`'s call sites), never name-embedded.
     launch-head-kw :wat::spawn::Locus/launch
     launch-tp-ann  `[~proto-op-ty-ann ~proto-reply-ty-ann ~state-ty-ann ~admin-ty-ann ~status-ty-ann]

     ;; ── arc 272 6b-ii-β: transport-agnostic service-forms ────────────────────────
     ;; service-forms-kw must be defined before start-body (which splices ~service-forms-kw).
     ;; service-forms-kw: the keyword :<fqdn>::service-forms — the name of the emitted def.
     service-forms-kw (:wat::core::keyword/from-string
                        (:wat::string::interpolate "{b}::service-forms" :b fqdn-base))
     ;; The agnostic child :user::main: binds on :user::spawn::service-locus (a FREE
     ;; name — defservice does NOT define it). The ProcessOpts launch arm prepends
     ;; `(def :user::spawn::service-locus (process))` before spawning, so the child
     ;; universe resolves service-locus at startup to a ProcessOpts value. `:user::` is
     ;; the RENDEZVOUS space (bracket.wat's header), the same shape as
     ;; `:user::bracket::work-fn` — and it is REQUIRED, not stylistic: in the child these
     ;; forms are user residue, so a `:wat::`-tree def is a reserved-prefix violation.
     ;; self-peer S=addr-ty (child sends minted Address up), R=ship-ty (parent sends
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
     ;; arc 278 the recv'-outcome wall — hygienic binders for the child-main startup
     ;; recv' RecvOutcome match (a :user::main body → the ProgramBodyIntroducesName gate
     ;; forbids bare-Symbol binders; symbol-node + unquote appear as Unquote nodes).
     cm-shipmsg-sym   (:wat::core::symbol-node "shipmsg")
     cm-shipcause-sym (:wat::core::symbol-node "shipcause")
     cm-st-sym   (:wat::core::symbol-node "st")
     ;; arc 291 3a-ii-α: child-main-form uses the lineage protocol.
     ;; self-peer: (Peer :- [Status Admin])
     ;;   child sends Status::Started(addr) UP, receives Admin DOWN.
     ;; The send' wraps addr in Status::Started (was: raw addr).
     ;; The recv' gets Admin; dispatch-admin applies to it (was: init applied to raw ship).
     child-main-form `(:wat::core::defn :user::main [] -> :wat::core::nil
                        (:wat::core::let
                          ;; arc 278 startup-crash parity: recv ship → run :init → send Started
                          ;; (was: send Started → recv ship → run :init). :init now runs BEFORE
                          ;; Status::Started, so an :init crash dies before the send → the parent's
                          ;; crash-aware `recv' svc` (spawn.wat ProcessOpts, reordered to send-ship-
                          ;; then-recv-Started) RAISES the child's reason instead of /start
                          ;; succeeding and the owner's later connect' getting a bare ECONNREFUSED.
                          ;; Arc 278 the parametric protocol — the TYPE-position spellings.
                          ;; `listener'` types the `Bound`, whose `Address` flows into
                          ;; `Status::Started` — a `(Status :- [K V])` variant, so its addr slot is
                          ;; `(Address :- [(Op :- [K V]) (Reply :- [K V])])` and a BARE `(Address :- [Op Reply])` does
                          ;; not unify with it. In this generated child `:user::main` the params
                          ;; are FREE type vars (exactly as they already are in the sibling
                          ;; `(self-peer ~status-ty ~admin-ty)` below), which is what the child
                          ;; instance is: one erased instantiation. At RUNTIME the decode target
                          ;; a type var names is opaque (`edn_to_typed_value`'s var arm) — the
                          ;; concrete fields around it are still decoded and enforced exactly.
                          [~cm-b-sym    (:wat::kernel::listener :user::spawn::service-locus
                                            ~proto-op-ty-ann ~proto-reply-ty-ann ~max-frame-bytes-node)
                           ~cm-self-sym (:wat::program::self-peer ~status-ty-runtime ~admin-ty-runtime)
                           ~cm-ship-sym (:wat::core::match (:wat::kernel::recv ~cm-self-sym) 
                                            ((:wat::kernel::RecvOutcome::Message ~cm-shipmsg-sym) ~cm-shipmsg-sym)
                                            ;; arc 278 the recv'-outcome wall — the child lost/closed its
                                            ;; owner link before the startup ship arrived: eprintln is the
                                            ;; terminal dying declaration (loud, exits non-zero).
                                            ((:wat::kernel::RecvOutcome::Lost ~cm-shipcause-sym)
                                              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message ~cm-shipcause-sym) :wat::core::None :wat::core::None))
                                            ;; arc 278 #73 — the owner link did not close; a stop
                                            ;; arrived before the startup ship. Distinct fact, so a
                                            ;; distinct line: reporting "closed" here sent every
                                            ;; reader of this log after a link that was fine.
                                            (:wat::kernel::RecvOutcome::Stopped
                                              (:wat::kernel::eprintln "defservice child-main: stop requested before the startup ship — owner link was ALIVE"))
                                            (:wat::kernel::RecvOutcome::Closed
                                              (:wat::kernel::eprintln "defservice child-main: owner link closed before startup ship")))
                           ~cm-st-sym   (:wat::core::apply
                                            (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                            ~cm-ship-sym [])
                           ;; arc 278 the send'-outcome wall — the owner's crash-aware `recv' svc`
                           ;; (spawn.wat ProcessOpts) faces a gone-owner outcome on its side; this
                           ;; send' proceeds into serve regardless (faced, not `_`-swallowed).
                           ~cm-und-sym  (:wat::core::match (:wat::kernel::send ~cm-self-sym
                                            (~status-started-kw (:wat::spawn::Bound/address ~cm-b-sym)))
                                          (:wat::kernel::SendOutcome::Sent   nil)
                                          (:wat::kernel::SendOutcome::Closed nil)
                                          ;; arc 278 #73 — a stop arrived as the child announced
                                          ;; readiness. Same body, and the precondition is that the
                                          ;; child proceeds into `serve`, whose poll' faces the stop
                                          ;; as its own event; the owner's recv' faces it too.
                                          (:wat::kernel::SendOutcome::Stopped nil)
                                          ((:wat::kernel::SendOutcome::Lost _c) nil))]
                          ;; arc 278 the call context — the process-tier child's OWN initial
                          ;; `serve` call: the empty selectables vector is now Tuple-entry typed,
                          ;; and next-id starts at 0 (the first connection mints id 0).
                          (:wat::core::apply
                            (:wat::core::keyword/from-string ~serve-name-str) ~cm-self-sym
                            (:wat::spawn::Bound/listener ~cm-b-sym)
                            (:wat::core::Vector ~selectable-entry-ty)
                            0
                            ~cm-st-sym [])))
     ;; The transport-agnostic service-forms defn: Op/Reply/records/serve + agnostic child
     ;; main. Emitted as `(defn :<fqdn>::service-forms [] -> (Vector :- [WatAST]) (forms …))`.
     ;; A 0-arg fn so the checker can type-check call sites: `(:my::counter::service-forms)`
     ;; returns (Vector :- [WatAST]). Registered into sym.functions at step 6 via
     ;; preregister_fn_defs_in_do, so the checker sees it before checking start-fn.
     ;; The ProcessOpts launch arm receives the Vector value (the runtime evaluates the
     ;; call before dispatch, so it arrives as the actual Vec).
     ;; own-forms-call: the full service-forms body (this service's server internals + child main).
     ;; DESIGN-STONE the-child-needs-the-entry-not-the-library: a `:wat::`-rooted fqdn-base
     ;; can only be baked stdlib source (the reserved-prefix wall admits no other origin —
     ;; `src/resolve/reserved.rs:25-27`), so the child already has every internal below;
     ;; re-shipping them is what turned `resolve::gate -> Reserved` red in the child. The ONE
     ;; thing shipped unconditionally is `child-main-form` — generated per service, in no
     ;; bake, and the child's entry point; dropping it leaves the child with nothing to run.
     fqdn-is-wat-rooted? (:wat::string::starts-with? fqdn-base "wat::")
     own-forms-call  (:wat::core::if fqdn-is-wat-rooted?

                       `(:wat::core::forms ~child-main-form)
                       `(:wat::core::forms
                          ~record-def
                          ~state-def
                          ~service-op-def
                          ~@service-op-derive-items
                          (:wat::core::defn ~serve-name ~serve-params
                            -> :wat::core::nil ~serve-body)
                          ~init-def
                          ~stop-project-def
                          ~hibernate-project-def
                          ~admin-enum-def
                          ~status-enum-def
                          ~dispatch-admin-def
                          ~extract-addr-def
                          ~child-main-form))
     ;; ── Arc 278 S4c: the surface OWNS its protocol; SHIP it. ──────────────────────
     ;; The satisfied surface's `<S>::surface-forms` carrier (emitted by defsurface in Rust) is
     ;; a (Vector :- [WatAST]) of the surface's own forms (its :messages records/enums + the defsurface
     ;; that re-synthesizes ::Op/::Reply at the child's fresh startup). Concat it AHEAD of this
     ;; service's own forms so a forked child resolves the protocol its serve loop references.
     ;; proto-str = the surface fqdn (`:satisfies` is mandatory; `:ops` is retired), so the carrier
     ;; name is `<surface>::surface-forms`.
     surface-forms-kw (:wat::core::keyword/from-string
                        (:wat::string::interpolate "{proto-base}::surface-forms" :proto-base proto-base))
     ;; Arc 278 S4d: concat the OWN surface's forms + every :peers surface's forms + own internals.
     ;; `concat` is strictly binary, so we build a LEFT-nested chain (order-preserving):
     ;;   (concat (concat … (concat (OwnSurface::surface-forms) (S1::surface-forms)) …) own-forms-call)
     ;; peers-forms-node folds each `(:Si::surface-forms)` onto the own-surface call; empty :peers
     ;; → peers-forms-node is just `(OwnSurface::surface-forms)` (identical to the pre-S4d concat).
     ;; DESIGN-STONE the-child-needs-the-entry-not-the-library: the OWN surface's
     ;; `<S>::surface-forms` call is the base of the fold. A `:wat::`-rooted proto-base
     ;; can only be a baked stdlib surface, so the child already has it — the base becomes
     ;; the empty `(:wat::core::forms)` rather than the call, and peer-forms-calls (already
     ;; filtered above) folds onto that instead.
     proto-is-wat-rooted? (:wat::string::starts-with? proto-base "wat::")
     own-surface-forms-node (:wat::core::if proto-is-wat-rooted?

                               `(:wat::core::forms)
                               `(~surface-forms-kw))
     peers-forms-node (:wat::core::foldl
                        (:wat::core::fn [acc       <- :wat::WatAST
                                         call-node <- :wat::WatAST]
                          -> :wat::WatAST
                          `(:wat::core::concat ~acc ~call-node))
                        own-surface-forms-node
                        peer-forms-calls)
     service-forms-def `(:wat::core::defn ~service-forms-kw
                          [] -> (:wat::core::Vector :- [:wat::WatAST])
                          (:wat::core::concat ~peers-forms-node ~own-forms-call))

     ;; 293.W.2f — `/start` must not erase T. Native kwargs+defclause is unexpressible
     ;; (`& [… ]` is a defn-macro idiom; defclause's argspec rejects a vector after `&`;
     ;; a generated defclause inside this `do` is also invisible to the top-level
     ;; defclause preregister). Public `start` stays a kwargs macro. Three positional
     ;; impls (ThreadOpts → (Handle :- [Shared]), ProcessOpts → (Handle :- [Wire]), Locus residual)
     ;; so K,V infer from init args; the macro picks the impl from the `:locus` AST.
     ;; Abstract-locus (a symbol / `Locus`-typed value) is the residual — T stays unknown.
     start-impl-name (:wat::core::keyword/from-string
                       (:wat::string::interpolate "{b}/start$impl" :b fqdn-base))
     start-impl-call (:wat::core::keyword/from-string
                       (:wat::string::interpolate "{b}/start$impl" :b fqdn-base))
     start-impl-thread-name (:wat::core::keyword/from-string
                              (:wat::string::interpolate "{b}/start$impl-thread" :b fqdn-base))
     start-impl-thread-call (:wat::core::keyword/from-string
                              (:wat::string::interpolate "{b}/start$impl-thread" :b fqdn-base))
     start-impl-process-name (:wat::core::keyword/from-string
                               (:wat::string::interpolate "{b}/start$impl-process" :b fqdn-base))
     start-impl-process-call (:wat::core::keyword/from-string
                               (:wat::string::interpolate "{b}/start$impl-process" :b fqdn-base))
     start-macro-name (:wat::core::keyword/from-string
                        (:wat::string::interpolate "{b}/start" :b fqdn-base))
     resume-impl-name (:wat::core::keyword/from-string
                        (:wat::string::interpolate "{b}/resume$impl" :b fqdn-base))
     resume-impl-call (:wat::core::keyword/from-string
                        (:wat::string::interpolate "{b}/resume$impl" :b fqdn-base))
     resume-impl-thread-name (:wat::core::keyword/from-string
                               (:wat::string::interpolate "{b}/resume$impl-thread" :b fqdn-base))
     resume-impl-thread-call (:wat::core::keyword/from-string
                               (:wat::string::interpolate "{b}/resume$impl-thread" :b fqdn-base))
     resume-impl-process-name (:wat::core::keyword/from-string
                                (:wat::string::interpolate "{b}/resume$impl-process" :b fqdn-base))
     resume-impl-process-call (:wat::core::keyword/from-string
                                (:wat::string::interpolate "{b}/resume$impl-process" :b fqdn-base))
     resume-macro-name (:wat::core::keyword/from-string
                         (:wat::string::interpolate "{b}/resume" :b fqdn-base))
     start-call-args-sym (:wat::core::symbol-node "call-args")
     start-fname-nodes (:wat::core::foldl
                         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                          n   <- :wat::WatAST]
                           -> (:wat::core::Vector :- [:wat::WatAST])
                           (:wat::core::conj acc n))
                         (:wat::core::conj (:wat::core::Vector :wat::WatAST) locus-sym)
                         init-arg-names)
     start-fnames-ast (:wat::core::with-children init-params-vec start-fname-nodes)
     start-impl-params `[~locus-sym <- :wat::spawn::Locus ~@init-param]
     start-impl-thread-params `[~locus-sym <- :wat::spawn::ThreadOpts ~@init-param]
     start-impl-process-params `[~locus-sym <- :wat::spawn::ProcessOpts ~@init-param]
     start-handle-expr `(~handle-new-kw (:wat::spawn::Launched/handle ~lr-sym)
                                        (:wat::spawn::Launched/address ~lr-sym))
     start-body    `(:wat::core::let
                      [~origin-sym (:wat::kernel::call-site)
                       ~lr-sym (~launch-head-kw :- ~launch-tp-ann
                                 ;; arc 170 closure #6 — label this service's process locus
                                 ;; with its own fqdn + the START CALL SITE (the
                                 ;; `:wat::process::Service` identity, wat/process.wat); a
                                 ;; no-op for a thread locus (with-label's ThreadOpts arm).
                                 ;; `fqdn-base` is the params-stripped BASE name — a
                                 ;; runtime-name-string keyword, same convention as
                                 ;; `::Handle{p}`'s sibling names. The name says WHICH
                                 ;; service; the origin says which of possibly several
                                 ;; starts brought THIS process up.
                                 (:wat::spawn::with-label ~locus-sym
                                   (:wat::process::Service
                                     :name (:wat::core::keyword/from-string ~fqdn-base)
                                     :file (:wat::kernel::Frame/file ~origin-sym)
                                     :line (:wat::kernel::Frame/line ~origin-sym)))
                                 (~admin-init-kw ~@init-arg-names)
                                 (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                 (:wat::core::keyword/from-string ~serve-name-str)
                                 (~service-forms-kw)
                                 (:wat::core::keyword/from-string ~extract-addr-name-str)
                                 ;; arc 278: lu-mk-kw = the Status::Started ctor (thread tier's
                                 ;; generic serve closure uses it to send Started after :init).
                                 (:wat::core::keyword/from-string ~status-started-str))]
                      ~start-handle-expr)
     start-body-thread `(:wat::core::let
                          [~origin-sym (:wat::kernel::call-site)
                           ~lr-sym (~launch-head-kw :- ~launch-tp-ann
                                     (:wat::spawn::with-label ~locus-sym
                                       (:wat::process::Service
                                         :name (:wat::core::keyword/from-string ~fqdn-base)
                                         :file (:wat::kernel::Frame/file ~origin-sym)
                                         :line (:wat::kernel::Frame/line ~origin-sym)))
                                     (~admin-init-kw ~@init-arg-names)
                                     (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                     (:wat::core::keyword/from-string ~serve-name-str)
                                     (~service-forms-kw)
                                     (:wat::core::keyword/from-string ~extract-addr-name-str)
                                     (:wat::core::keyword/from-string ~status-started-str))]
                          (:wat::core::ann-form ~start-handle-expr ~handle-shared-name))
     start-body-process `(:wat::core::let
                           [~origin-sym (:wat::kernel::call-site)
                            ~lr-sym (~launch-head-kw :- ~launch-tp-ann
                                      (:wat::spawn::with-label ~locus-sym
                                        (:wat::process::Service
                                          :name (:wat::core::keyword/from-string ~fqdn-base)
                                          :file (:wat::kernel::Frame/file ~origin-sym)
                                          :line (:wat::kernel::Frame/line ~origin-sym)))
                                      (~admin-init-kw ~@init-arg-names)
                                      (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                      (:wat::core::keyword/from-string ~serve-name-str)
                                      (~service-forms-kw)
                                      (:wat::core::keyword/from-string ~extract-addr-name-str)
                                      (:wat::core::keyword/from-string ~status-started-str))]
                           (:wat::core::ann-form ~start-handle-expr ~handle-wire-name))
     start-impl-fn `(:wat::core::defn ~start-impl-name ~start-impl-params -> ~handle-name-ann ~start-body)
     start-impl-thread-fn `(:wat::core::defn ~start-impl-thread-name ~start-impl-thread-params -> ~handle-shared-name ~start-body-thread)
     start-impl-process-fn `(:wat::core::defn ~start-impl-process-name ~start-impl-process-params -> ~handle-wire-name ~start-body-process)
     start-fn      `(:wat::core::do
                      ~start-impl-fn
                      ~start-impl-thread-fn
                      ~start-impl-process-fn
                      (:wat::core::defmacro ~start-macro-name
                        [& ~start-call-args-sym <- (:wat::core::Vector :- [:wat::WatAST])]
                        -> :wat::WatAST
                        (:wat::core::let
                          [~(:wat::core::symbol-node "flat")
                           (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::length call-args) 1)
                                               (:wat::core::= (:wat::core::ast-kind (:wat::core::first call-args)) "map")
                                               false)
                             (:wat::core::ast->children (:wat::core::first call-args))
                             call-args)
                           ~(:wat::core::symbol-node "found")
                           (:wat::core::foldl
                             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                              i   <- :wat::core::i64]
                               -> (:wat::core::Vector :- [:wat::WatAST])
                               (:wat::core::if (:wat::core::not (:wat::core::empty? acc))
                                 acc
                                 (:wat::core::let
                                   [k (:wat::core::Option/expect
                                        (:wat::core::get flat (:wat::core::i64::* i 2))
                                        "start kwargs: locus key")
                                    v (:wat::core::Option/expect
                                        (:wat::core::get flat (:wat::core::i64::+ (:wat::core::i64::* i 2) 1))
                                        "start kwargs: locus val")]
                                   (:wat::core::if (:wat::core::= (:wat::core::ast-name k) ":locus")
                                     (:wat::core::conj acc v)
                                     acc))))
                             (:wat::core::Vector :wat::WatAST)
                             (:wat::core::range 0 (:wat::core::i64::/ (:wat::core::length flat) 2)))
                           ~(:wat::core::symbol-node "locus-ast")
                           (:wat::core::if (:wat::core::empty? found) :wat::core::nil (:wat::core::first found))
                           ~(:wat::core::symbol-node "head-nm")
                           (:wat::core::if (:wat::core::empty? found)
                             ""
                             (:wat::core::if (:wat::core::= (:wat::core::ast-kind locus-ast) "list")
                               (:wat::core::let [ch (:wat::core::ast->children locus-ast)]
                                 (:wat::core::if (:wat::core::empty? ch) "" (:wat::core::ast-name (:wat::core::first ch))))
                               ""))
                           ~(:wat::core::symbol-node "inner-nm")
                           (:wat::core::if (:wat::core::= head-nm ":wat::spawn::with-label")
                             (:wat::core::let [ch (:wat::core::ast->children locus-ast)]
                               (:wat::core::if (:wat::core::empty? (:wat::core::rest ch))
                                 ""
                                 (:wat::core::let [inner (:wat::core::first (:wat::core::rest ch))]
                                   (:wat::core::if (:wat::core::= (:wat::core::ast-kind inner) "list")
                                     (:wat::core::let [ich (:wat::core::ast->children inner)]
                                       (:wat::core::if (:wat::core::empty? ich) "" (:wat::core::ast-name (:wat::core::first ich))))
                                     ""))))
                             head-nm)
                           ~(:wat::core::symbol-node "ctor-nm")
                           (:wat::core::if (:wat::core::= head-nm ":wat::spawn::with-label") inner-nm head-nm)
                           ~(:wat::core::symbol-node "impl")
                           (:wat::core::if (:wat::string::starts-with? ctor-nm ":wat::spawn::process")
                             ~start-impl-process-call
                             (:wat::core::if (:wat::string::starts-with? ctor-nm ":wat::spawn::thread")
                               ~start-impl-thread-call
                               ~start-impl-call))
                           ~(:wat::core::symbol-node "kty")  (:wat::core::keyword-node ":wat::core::agg-positional")
                           ~(:wat::core::symbol-node "fvec") (:wat::core::quote ~start-fnames-ast)
                           ~(:wat::core::symbol-node "ns")   (:wat::core::keyword-node (:wat::string::concat ":" ~fqdn-base))]
                          `(:wat::core::kwargs-lower ~impl ~kty ~fvec 0 ~ns ~@call-args))))

     ;; ── arc 291 4b-ii: resume fn (mirror of start, ships Admin::Resume instead of Admin::Init) ──
     ;; (defn <fqdn>/resume [locus <- :wat::spawn::Locus  snapshot <- ~record-ty] -> ~handle-name
     ;;   (let [lr (launch :- [Op Reply State Admin Status] locus (Admin::Resume snapshot) dispatch-admin serve service-forms lu-addr)]
     ;;     (Handle (Launched/handle lr) (Launched/address lr))))
     ;; dispatch-admin routes Admin::Resume → (init snapshot) to rebuild the struct.
     ;; launch is UNCHANGED — resume reuses the same machinery.
     ;; `snapshot` param binder: use a symbol-node (hygiene: Unquote at def time).
     ;; 293.W.2f — resume is the same T-stamp as start (kwargs UX + impl + ann-form).
     resume-body    `(:wat::core::let
                       [~origin-sym (:wat::kernel::call-site)
                        ~lr-sym (~launch-head-kw :- ~launch-tp-ann
                                  ;; arc 170 closure #6 — see start-body's identical wrap.
                                  (:wat::spawn::with-label ~locus-sym
                                    (:wat::process::Service
                                      :name (:wat::core::keyword/from-string ~fqdn-base)
                                      :file (:wat::kernel::Frame/file ~origin-sym)
                                      :line (:wat::kernel::Frame/line ~origin-sym)))
                                  (~admin-resume-kw ~@init-arg-names)
                                  (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                  (:wat::core::keyword/from-string ~serve-name-str)
                                  (~service-forms-kw)
                                  (:wat::core::keyword/from-string ~extract-addr-name-str)
                                  ;; arc 278: lu-mk-kw = the Status::Started ctor (see start-body).
                                  (:wat::core::keyword/from-string ~status-started-str))]
                       ~start-handle-expr)
     resume-body-thread `(:wat::core::let
                           [~origin-sym (:wat::kernel::call-site)
                            ~lr-sym (~launch-head-kw :- ~launch-tp-ann
                                      (:wat::spawn::with-label ~locus-sym
                                        (:wat::process::Service
                                          :name (:wat::core::keyword/from-string ~fqdn-base)
                                          :file (:wat::kernel::Frame/file ~origin-sym)
                                          :line (:wat::kernel::Frame/line ~origin-sym)))
                                      (~admin-resume-kw ~@init-arg-names)
                                      (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                      (:wat::core::keyword/from-string ~serve-name-str)
                                      (~service-forms-kw)
                                      (:wat::core::keyword/from-string ~extract-addr-name-str)
                                      (:wat::core::keyword/from-string ~status-started-str))]
                           (:wat::core::ann-form ~start-handle-expr ~handle-shared-name))
     resume-body-process `(:wat::core::let
                            [~origin-sym (:wat::kernel::call-site)
                             ~lr-sym (~launch-head-kw :- ~launch-tp-ann
                                       (:wat::spawn::with-label ~locus-sym
                                         (:wat::process::Service
                                           :name (:wat::core::keyword/from-string ~fqdn-base)
                                           :file (:wat::kernel::Frame/file ~origin-sym)
                                           :line (:wat::kernel::Frame/line ~origin-sym)))
                                       (~admin-resume-kw ~@init-arg-names)
                                       (:wat::core::keyword/from-string ~dispatch-admin-name-str)
                                       (:wat::core::keyword/from-string ~serve-name-str)
                                       (~service-forms-kw)
                                       (:wat::core::keyword/from-string ~extract-addr-name-str)
                                       (:wat::core::keyword/from-string ~status-started-str))]
                            (:wat::core::ann-form ~start-handle-expr ~handle-wire-name))
     resume-impl-fn `(:wat::core::defn ~resume-impl-name ~start-impl-params -> ~handle-name-ann ~resume-body)
     resume-impl-thread-fn `(:wat::core::defn ~resume-impl-thread-name ~start-impl-thread-params -> ~handle-shared-name ~resume-body-thread)
     resume-impl-process-fn `(:wat::core::defn ~resume-impl-process-name ~start-impl-process-params -> ~handle-wire-name ~resume-body-process)
     resume-fn      `(:wat::core::do
                       ~resume-impl-fn
                       ~resume-impl-thread-fn
                       ~resume-impl-process-fn
                       (:wat::core::defmacro ~resume-macro-name
                         [& ~start-call-args-sym <- (:wat::core::Vector :- [:wat::WatAST])]
                         -> :wat::WatAST
                         (:wat::core::let
                           [~(:wat::core::symbol-node "flat")
                            (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::length call-args) 1)
                                                (:wat::core::= (:wat::core::ast-kind (:wat::core::first call-args)) "map")
                                                false)
                              (:wat::core::ast->children (:wat::core::first call-args))
                              call-args)
                            ~(:wat::core::symbol-node "found")
                            (:wat::core::foldl
                              (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST])
                                               i   <- :wat::core::i64]
                                -> (:wat::core::Vector :- [:wat::WatAST])
                                (:wat::core::if (:wat::core::not (:wat::core::empty? acc))
                                  acc
                                  (:wat::core::let
                                    [k (:wat::core::Option/expect
                                         (:wat::core::get flat (:wat::core::i64::* i 2))
                                         "resume kwargs: locus key")
                                     v (:wat::core::Option/expect
                                         (:wat::core::get flat (:wat::core::i64::+ (:wat::core::i64::* i 2) 1))
                                         "resume kwargs: locus val")]
                                    (:wat::core::if (:wat::core::= (:wat::core::ast-name k) ":locus")
                                      (:wat::core::conj acc v)
                                      acc))))
                              (:wat::core::Vector :wat::WatAST)
                              (:wat::core::range 0 (:wat::core::i64::/ (:wat::core::length flat) 2)))
                            ~(:wat::core::symbol-node "locus-ast")
                            (:wat::core::if (:wat::core::empty? found) :wat::core::nil (:wat::core::first found))
                            ~(:wat::core::symbol-node "head-nm")
                            (:wat::core::if (:wat::core::empty? found)
                              ""
                              (:wat::core::if (:wat::core::= (:wat::core::ast-kind locus-ast) "list")
                                (:wat::core::let [ch (:wat::core::ast->children locus-ast)]
                                  (:wat::core::if (:wat::core::empty? ch) "" (:wat::core::ast-name (:wat::core::first ch))))
                                ""))
                            ~(:wat::core::symbol-node "inner-nm")
                            (:wat::core::if (:wat::core::= head-nm ":wat::spawn::with-label")
                              (:wat::core::let [ch (:wat::core::ast->children locus-ast)]
                                (:wat::core::if (:wat::core::empty? (:wat::core::rest ch))
                                  ""
                                  (:wat::core::let [inner (:wat::core::first (:wat::core::rest ch))]
                                    (:wat::core::if (:wat::core::= (:wat::core::ast-kind inner) "list")
                                      (:wat::core::let [ich (:wat::core::ast->children inner)]
                                        (:wat::core::if (:wat::core::empty? ich) "" (:wat::core::ast-name (:wat::core::first ich))))
                                      ""))))
                              head-nm)
                            ~(:wat::core::symbol-node "ctor-nm")
                            (:wat::core::if (:wat::core::= head-nm ":wat::spawn::with-label") inner-nm head-nm)
                            ~(:wat::core::symbol-node "impl")
                            (:wat::core::if (:wat::string::starts-with? ctor-nm ":wat::spawn::process")
                              ~resume-impl-process-call
                              (:wat::core::if (:wat::string::starts-with? ctor-nm ":wat::spawn::thread")
                                ~resume-impl-thread-call
                                ~resume-impl-call))
                            ~(:wat::core::symbol-node "kty")  (:wat::core::keyword-node ":wat::core::agg-positional")
                            ~(:wat::core::symbol-node "fvec") (:wat::core::quote ~start-fnames-ast)
                            ~(:wat::core::symbol-node "ns")   (:wat::core::keyword-node (:wat::string::concat ":" ~fqdn-base))]
                           `(:wat::core::kwargs-lower ~impl ~kty ~fvec 0 ~ns ~@call-args))))

     ;; ── C.3: Handle STRUCT ───────────────────────────────────────────────────────
     ;; (defstruct <fqdn>::Handle
     ;;   [handle <- (Peer :- [Admin Status])
     ;;    addr   <- (:wat::kernel::Address :- [fqdn::Op fqdn::Reply])])
     ;; arc 291 3a-ii-β: handle is the owner-only lineage peer (admin channel).
     ;; (Peer :- [Admin Status]) — owner sends Admin (down), receives Status (up).
     ;; (Thread :- [Admin Status]) and (Process :- [Admin Status]) both satisfy this field
     ;; (send'/recv' intrinsics accept Thread|Process|Peer uniformly).
     ;; addr carries the typed (Address :- [Op Reply]) for client connect'.
     ;;
     ;; ★ A STRUCT, NOT A RECORD — arc 278 2026-08-03, builder-ruled: "they are
     ;; resources - they are not pure." BOTH fields are live: `handle` is a peer
     ;; (crossbeam tx/rx or an fd pair) and `addr` is an Address RustOpaque. A
     ;; record is GUARANTEED pure data; a thing that holds a resource is a struct
     ;; ([[reference_struct_holds_resources_record_is_pure_data]]). Its own parent
     ;; `Launched` (wat/spawn.wat:265) holds these SAME two fields and has always
     ;; been a defstruct, for this exact reason, in its own comment — this
     ;; generated child was the one that wasn't.
     ;;
     ;; It was a `defrecord` until the §7 purity wall was corrected to cover
     ;; `Peer`/`Thread`/`Process` (check.rs `is_pure_type`; only `ThreadSelfPeer`
     ;; had been listed). Arming that wall lit 26 Handles and 2697 tests — the
     ;; corrected law naming every violator (R52 QVOD LEX ACCENDIT). This one
     ;; token is the whole fix: a Handle is an owner-side CAPABILITY, never data,
     ;; and only ADDRESSES cross (293.W).
     ;; identity 2c: handle-peer-ty is ANNOTATION-only (Handle struct field) — mints the
     ;; reference FORM, structurally off `admin-ty-ann`/`status-ty-ann` (both already
     ;; reference-form nodes — Arc 109 ③ retired the angle-string concat this used).
     handle-peer-ty `(:wat::kernel::Peer :- [~admin-ty-ann ~status-ty-ann])
     handle-fields `[handle <- ~handle-peer-ty addr <- ~addr-ty]
     ;; Arc 109 ③ — `handle-tp-syms` always carries at least the transport marker, so the
     ;; binder splice is unconditional here (unlike `record-def`/`state-def`, which may be
     ;; genuinely monomorphic).
     handle-record `(:wat::core::defstruct ~handle-name-decl :- [~@handle-tp-syms] ~handle-fields)

     ;; ── arc 170: auto-emit the Capability extend-type ─────────────────────────────
     ;; Every <fqdn>::Handle uniformly satisfies :wat::capability::Capability (relocated to
     ;; wat/capability.wat, stone 2; renamed from Grantable, stone A), routing grant/revoke to
     ;; the already-landed <fqdn>/grant & <fqdn>/revoke methods, and `coordinate` to an
     ;; up-cast of the handle's own typed addr field to the surface's bare Address — so a
     ;; single handle carries both grant AND dial. This is a TOP-LEVEL form (NOT a method —
     ;; methods are client fns; extend-type is not). Mirrors the hand-written extend-type
     ;; proven in scratchpad/probe-coordinate-on-surface.wat. self/pids binders use
     ;; symbol-node for hygiene (Unquote at def time).
     grantable-self-sym (:wat::core::symbol-node "self")
     grantable-pids-sym (:wat::core::symbol-node "pids")
     handle-addr-name (:wat::core::keyword/from-string
                         (:wat::string::interpolate "{b}::Handle/addr" :b fqdn-base))
     grantable-extend `(:wat::core::extend-type ~handle-bare-name :wat::capability::Capability
                         (grant  [~grantable-self-sym ~grantable-pids-sym] (~grant-call-name  ~grantable-self-sym ~grantable-pids-sym))
                         (revoke [~grantable-self-sym ~grantable-pids-sym] (~revoke-call-name ~grantable-self-sym ~grantable-pids-sym))
                         (coordinate [~grantable-self-sym]
                           (:wat::core::ann-form (~handle-addr-name ~grantable-self-sym) :wat::kernel::Address)))

     ;; ── arc 170 W1: auto-emit the Dialable :- [S R] extend-type ───────────────────
     ;; A SECOND, PARAMETRIC surface (wat/capability.wat) every <fqdn>::Handle also
     ;; satisfies, beside the flat Capability above. Where Capability/coordinate up-casts
     ;; to a bare Address (service-erased, for uniform grant/revoke), Dialable/coord
     ;; returns the handle's own TYPED addr field directly — (Address :- [proto::Op proto::
     ;; Reply]) — so a wrong-service dial is a compile-time discrimination error. Mirrors
     ;; the hand-proven extend-type in scratchpad/probe-c2-typed-coordinate.wat. proto-str
     ;; (not fqdn-str) matches addr-ty's own Op/Reply namespace (arc 293 S2, line ~472).
     dialable-ty `(:wat::capability::Dialable :- [~proto-op-ty-ann ~proto-reply-ty-ann])
     dialable-extend `(:wat::core::extend-type ~handle-bare-name ~dialable-ty
                         (coord [~grantable-self-sym] (~handle-addr-name ~grantable-self-sym)))

     ;; ── arc 170 C2 D: auto-emit the THIRD, BODILESS TypedCapability :- [S R] extend-type ─
     ;; Registers the satisfaction EDGE only — no method bodies (that's the whole point: a
     ;; third re-declaration of coord/grant/revoke here would collide with grantable-extend/
     ;; dialable-extend's own bodies on the flat `<Type>/<method>` key, DuplicateDefine).
     ;; Runtime dispatch serves TypedCapability/coord|grant|revoke off THIS SAME Handle's
     ;; Capability+Dialable bodies above via that flat key. Mirrors dialable-ty's Op/Reply
     ;; wiring exactly (proto-str namespace, not fqdn-str — same reasoning as line ~1192).
     ;; Shape proven scratchpad/probe-v-bodiless.wat / probe-v-swap.wat / probe-v-run.wat.
     typedcap-ty `(:wat::capability::TypedCapability :- [~proto-op-ty-ann ~proto-reply-ty-ann])
     typedcap-extend `(:wat::core::extend-type ~handle-bare-name ~typedcap-ty)]

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
       ~service-op-def
       ~@service-op-derive-items
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
       ~handle-record
       ~grantable-extend
       ~dialable-extend
       ~typedcap-extend)))
