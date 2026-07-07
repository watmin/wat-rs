# DESIGN — 293: services-as-surfaces (a `defservice` satisfies a surface — the AWS service model, decomplected)

> **Status: CONTRACT SETTLED (2026-07-05 — the operation model: `RequestRecord → ResponseRecord`, errors as `Reply`
> variants, both width-evolvable + checker-walled). ERROR SHAPE RESOLVED (2026-07-06, open item #4) — every op returns a
> per-op `*Result` enum (`:Success` first + only that op's error variants; reusable error records; exhaustive-match-walled),
> and the surface is `:nature :wat::kernel::Peer'` (post `Holder→Nature`). See § The `*Result` shape. The CODEGEN
> re-pointing + intueri names + a feasibility probe are what remain to draw (see § The mechanism + § Open).** This is 293's
> **unfinished third face.** 293 fused struct+record into the aggregate and made `defsurface` a set-of-accessors
> satisfied by **attrs** (a struct's fields) and **methods** (an `extend-type`). The one citizen that never got folded
> in is the **service** — it predates surfaces (arc 209/291), still mints its own private `::Op`/`::Reply`, and
> satisfies nothing. This delivers the third way a surface is satisfied: **by a service, at the wire.** 293 was already
> open (closure-gated on the aggregate audit; closes with 294; 291 blocked on it) — this is a stone within it, not a
> new arc. **278/T1b (the blind telemetry sink) is blocked on this** and resumes by inheritance once it lands.

## How we got here (the arc-170 unfolding — "started with 'can we add argv to main'")

A small honest question, pulled on until it became the real thing. The thread, on the record:
1. **T1b:** how does the telemetry sink write to a store *without naming mem vs sqlite*?
2. **The wall (293.W, checker-taught):** you cannot hand a service a live satisfier — a `Store` struct is impure; a
   start operating-input lands in the pure `resume::Kwargs`. Only **addresses** cross (a peer is crossbeam tx/rx or a
   unix pipe pair — process-local; a process must *dial* its peer). 293.W was right; I handed it the wrong thing.
3. **The reframe:** the sink depends on the `Store` **surface**, not on a store. An enum is closed (a user brings their
   own store); a surface is open. The sink asks *"is this a store?"*, never *"which."*
4. **Remembering what we knew:** loci are already **unbounded** — `Locus` (a legacy `defprotocol`, doctrinally a
   surface) is open, `start [locus <- :Locus]` and `connect'` are locus-agnostic, *"a new transport joins as one
   `extend-type`, zero edit."* Transport-agnosticism is **inherited**, not built.
5. **The gap, grounded:** `defservice` mints `::Op`/`::Reply` **per-service** (`{fqdn}::Op`), and `:calls` names
   *concrete* services. So `mem-store'::Op ≠ sqlite-store'::Op` — the sink can't dial both by one address. *Not* assembly
   this time: a real weld is missing.
6. **The recognition (builder):** this is **AWS API-as-JSON** — one interface spec a service *implements*, and clients
   *generated from the same spec* to build requests + handle replies.

## The thesis

**A `defservice` can `:satisfies` a surface. The surface SOURCES the service's wire-protocol** — the shared
`Op`/`Reply` enums and per-op request/response records come *from* the surface's methods, so **every service that
satisfies the same surface speaks the identical protocol.** The service supplies only its state
(`:durable`/`:ephemeral`/`:init`) and the op **bodies**. From one surface fall out **two sides**: the **server** (the
service's wire-protocol + bodies) and the **client** (request constructors, reply accessors, the dial-and-RPC methods).
A consumer `:calls [<surface>]` (the *surface*, not a concrete service) and dials `Address'<Surface::Op,
Surface::Reply>` — a *uniform* type across every backend — so it talks to any satisfier **blind**.

## The design (the shape agreed)

```clojure
;; the surface is the ONE contract / interface spec (exists today):
(:wat::core::defsurface :wat::query::Store :holder :wat::core::Struct
  :features [(put  [self rows] -> :wat::core::Result<..nil,..Error>)
             (scan [self q]    -> :wat::core::Result<..Page,..Error>) …])

;; SERVER side — a service SATISFIES the surface (generative): its wire-protocol is SOURCED from Store's methods.
;; The shared types are :wat::query::Store::Op / ::Reply + per-op request/response records — ONE set, not per-service.
(:wat::service::defservice :wat::query::mem-store'  :satisfies :wat::query::Store
  :durable [rows <- …] :ephemeral [] :init …
  :impls [(put  [s rows] …pv body…)  (scan [s q] …pv body…) …])
(:wat::service::defservice :wat::query::sqlite-store'  :satisfies :wat::query::Store
  :durable […] :ephemeral [conn …] :init …
  :impls [(put  [s rows] …sql body…) (scan [s q] …sql body…) …])
;;   ↑ BOTH speak :wat::query::Store::Op — one protocol, two backends.

;; CLIENT side — a consumer :calls the SURFACE (not a concrete service) + dials any Store address, blind:
(:wat::service::defservice :wat::telemetry'::TelemetryService'  :calls [:wat::query::Store]
  :ephemeral [store <- (peer dialed from a given Address'<Store::Op, Store::Reply>)]  …)
```

### The load-bearing rulings (settled tonight)

1. **`:satisfies` is GENERATIVE, not a redundant label.** The builder's earlier 293 concern (*"explicit `:satisfies`
   feels wrong… ambient 'you do or you don't'"*) was aimed at **structs** — a struct satisfies structurally (it *has*
   the shape), so declaring is noise. A **service** is the opposite: `:satisfies` *sources* the protocol from the
   surface. It does work. It is honest here.
2. **Exactly ONE surface per service.** A struct's surface is **read** (structural match → many, passively). A
   service's surface is **generated-from** (the protocol *is* the surface → one source). Two surfaces = two `Op` enums
   unioned into one wire-face = two services fused = decomplect them (293's own thesis). Exactly-one is what makes the
   address type `Address'<Store::Op, Store::Reply>` **unambiguous** — and that clean type is the *entire reason* the
   client can dial blind. Open in the dimension that matters (any number of services satisfy `Store`), closed in the
   one that would rot (one surface per service).
3. **The honest asymmetry, named:** structs satisfy *many* surfaces; a service satisfies *one*. Not a spurious split —
   a genuine difference in *kind* of satisfaction: **data is matched (many); a protocol is generated (one).** Same
   surface, two relationships — read-from vs sourced-from.
4. **Decomplected from transport.** The **surface** is the API (the ops); the **`Locus`** is the wire (thread / process
   / UDS / mTLS / a wat-native L4 — unbounded, one `extend-type` each). *Orthogonal.* The same generated client dials a
   store over shared memory OR a socket OR remote mTLS with zero client edits. (AWS welds API to HTTP; wat does not.)
5. **Homoiconic + typed.** The surface *is* a wat form in the same language (not a separate JSON/Smithy IDL); the
   checker enforces **both** ends — the service must implement the surface (or it won't compile) and the client must
   use it correctly (or it won't compile). No external codegen, no spec↔impl drift, no "SDK a version behind."

## The recognition — the AWS service model, derived and decomplected

This *is* the AWS service model, which the builder ran for years (Kinesis KCL, API Gateway, Shield): one model
(`service-2.json` / Coral / Smithy) → the framework generates **the server skeleton AND every client SDK** from the
one spec (build request → serialize → send → parse response). `RATIONE NON MIRACVLO` (R19) / R15 — not invented here,
**derived to where the greats stand**, because it's the correct shape and he'd already found it once. What makes it
*ours*, not a copy — the three decomplections above: **API split from transport** (locus-agnostic), **spec brought
into the language** (homoiconic), **codegen replaced by the type system** (enforced). `300 R7 VIRTVTE PARES` — a
dialect that fields what they field (one-model-two-ends) *and* what they can't (transport-blind, drift-free).

**The scope this reveals:** `defservice :satisfies surface` is not a telemetry convenience — it is **wat growing its
own AWS-grade service framework**, which fell out of "how does the sink dial a store." That is the method (arc 170
began "can we add argv to main"); the size is the point, not an accident.

## The mechanism — the OPERATION model (SETTLED 2026-07-05, the T1b co-design)

The whole contract, symmetric and grounded — reasoned to over the T1b design conversation.

**Every operation is `RequestRecord → ResponseRecord`. Both are NAMED records — structured units, not loose args.**
The surface declares them all:

```clojure
(:wat::core::defsurface :wat::query::Store :holder :wat::core::Struct
  :features [(put  [self PutRequest]  -> PutResponse)      ;; PutRequest [rows <- Vector<StoredRow>]  — a unit
             (scan [self ScanRequest] -> ScanResponse) …]) ;; ScanRequest already a record; ScanResponse now too
```

- This is the **AWS operation**, exactly: an Input shape → an Output shape, both records, nothing loose.
- It fixes a real inconsistency the current `Store` surface *already* shows — the smell that proves the point: `scan`
  takes a `ScanRequest` **record**, but `put` takes a bare `Vector<StoredRow>` **collection**. Same surface, two shapes
  for "the input" — the loose-args model leaking. Uniform: `put` takes `PutRequest`.

**Errors are `Reply` variants.** If everything on the wire is a named shape, an error is *also* a named shape. The
`Op` enum carries the request shapes; the `Reply` enum carries **success (the response record) + one variant per
modeled error** (each an error record). AWS models errors as first-class shapes; `Store` already models `Error` as a
recovery-axis enum — this is that, on the wire. (Supersedes the doc's earlier `-> Result<T,Error>` sketch: the Result
becomes the `Reply` enum's success-vs-error split.)

**Both records evolve by WIDTH and are CHECKER-WALLED — on BOTH ends:**
- *Evolvable:* add an optional field to `PutRequest` (a `:consistency` mode) or to `ScanResponse` (a `:scanned-count`)
  and every existing client keeps compiling — a *wider* record still satisfies the surface (293's structural
  width-subtyping). A loose arg list can grow *neither* direction without a breaking change; a record grows *both*.
  **This is the capability the AWS model existed for — evolve a service without versioned breakage — delivered by the
  subtyping 293 already built.**
- *Walled:* the client constructs a `PutRequest` (named — cannot misshape it); the server returns a `PutResponse`
  (named — checked against the surface, exactly the `extend-type` honesty strike `fa8bbcb9` / R29). The wrong shape is
  **uncompilable on both ends of the wire**, not just the response. Safety by wall, not author-discipline.

**The two selves.** A surface method `(put [self PutRequest] -> PutResponse)` — `self` is the *client's* view (the store
you call `Store/put` **on**, the dial target). Inside the service, that same position is the *server's* `s <- :State`.
One method, two ends: the client dials `self`, the server binds `s`. The macro owns the mapping. (This is the
client/server duality of a service wearing a surface.)

**The clause shape.** `:satisfies <one surface>` replaces `:ops` with `:impls` — **bodies only**; the sigs ARE the
surface, and re-listing them is the duplication `derive-is-the-wall` forbids (like a body-only `extend-type`). Each
impl constructs the named response record; the wire (`Op`/`Reply`, the request/response *wrapping*) is DERIVED from the
surface, invisible to the author:

```clojure
(:wat::service::defservice :wat::query::mem-store'  :satisfies :wat::query::Store
  :durable [rows <- …] :ephemeral [] :init …
  :impls   ;; (<surface-method> [s req] body). s = State (the server's self); req = the named request record.
  [(put  [s req] (:let [merged …] (:wat::service::Outcome::Reply (State merged) (:wat::query::PutResponse …))))
   (scan [s req] (:let [page …]   (:wat::service::Outcome::Reply s          (:wat::query::ScanResponse …))))])
```

**One consistent story.** Everything on the wire is a named shape. A service is a set of operations; each operation is
`RequestRecord → ResponseRecord` (errors are `Reply` variants); the surface declares every shape; the server
implements them (`:impls`) and the client is generated from them (`:calls [surface]`); both records evolve by width and
are enforced by the checker. The AWS service model, with the two things AWS never had — **structural evolvability
instead of versioned breakage, and the type system instead of codegen.**

## The `*Result` shape — RESOLVED (2026-07-06: the error channel, reasoned to over the S4 co-design)

Open item #4 is closed. The abstract *"errors are `Reply` variants (success + per-error records)"* resolves to one
concrete, uniform shape — the **per-operation `*Result` enum**, completing the `*Request` / `*Response` trio.

**Every operation returns a per-op `*Result` enum** — NOT a generic `Result<T, SharedError>`. A shared error enum
over-approximates: it lets *every* op carry *every* error, which is a lie about the op's real contract (`scan` cannot
raise a uniqueness `Constraint`; `put` cannot return "no such index"). Each op names its own outcome enum:

```clojure
(:wat::core::defenum :wat::query::PutResult :wat::enum::Pure
  :Success    [ret <- :wat::query::PutResponse]   ;; ALWAYS the first variant; carries the op's *Response
  :Constraint [err <- :wat::query::Constraint]    ;; then ONLY the errors PUT can actually emit
  :Transient  [err <- :wat::query::Transient]
  :Fatal      [err <- :wat::query::Fatal])

(put [self <- :wat::query::Store  req <- :wat::query::PutRequest] -> :wat::query::PutResult)
```

Three rules, all load-bearing:
- **`:Success` is always the first variant**, carrying that op's `*Response` record — a uniform head across every
  `*Result`, so the caller always knows the success arm.
- **Error records are a shared, reusable vocabulary**, defined ONCE (`Transient` / `Constraint` / `Fatal`, each carrying
  a `Fault`). They communicate the *shape of the thing the caller must handle*; each op's `*Result` selects the subset
  it can produce. The old recovery-axis `Error` *enum* dissolves into these standalone records.
- **Per-op error selection is the AWS/Smithy operation-error-set model, wat-native** — but enforced by the type system,
  not docs.

**The wall — the caller can never be surprised (`MVNIRE` on error-handling).** Because `*Result` is an enum, the caller
must `match` it, and wat's match is *exhaustive*: a modeled error left unhandled is a **compile error** — an unhandled
error has no representable form. Strictly stronger than a generic `Result` (whose single `Err` a caller can wave away):
every error an op can produce is a branch the caller is *forced* to write.

**Authoring conventions folded in here:**
- **`:satisfies` is authored FIRST** — always the leading clause of a `:satisfies` defservice: identity (the surface it
  *is*) before state (`:durable`/`:ephemeral`) before impl (`:impls`). The macro's clause-map is order-free; this is a
  presentation law, not a mechanism constraint.
- **The surface is `:nature :wat::kernel::Peer'`** (post `Holder→Nature`, `4b9a6d7f` + R32 `QVANTVMVIS PROCVL IDEM
  NEXVS`) — a service is a surface at a coordinate; the peer dispatches intrinsically (Path B, `823b20ac`). The
  `:holder :Struct` in this doc's other examples predates the rename; migrating `:wat::query::Store` →
  `:nature :Peer'` **is** stone S4a.

### The reshaped `:wat::query::Store` contract (the S4a target — the worked example)

```clojure
;; ── shared error-record vocabulary — defined ONCE, reused across ops (the recovery axis) ──
(:wat::core::defrecord :wat::query::Fault
  [op <- :wat::core::keyword  code <- :wat::core::i64
   diagnostic <- :wat::core::String  message <- :wat::core::String])
(:wat::core::defrecord :wat::query::Transient  [fault <- :wat::query::Fault])   ;; retry — momentarily unavailable
(:wat::core::defrecord :wat::query::Constraint [fault <- :wat::query::Fault])   ;; surface — schema/uniqueness violation
(:wat::core::defrecord :wat::query::Fatal      [fault <- :wat::query::Fault])   ;; abort — unrecoverable

;; ── per-op request / response records ──
(:wat::core::defrecord :wat::query::EnsureSchemaRequest
  [table <- :wat::query::TableSchema  indexes <- (:wat::core::Vector :wat::query::IndexSchema)])
(:wat::core::defrecord :wat::query::EnsureSchemaResponse [])
(:wat::core::defrecord :wat::query::PutRequest  [rows <- (:wat::core::Vector :wat::query::StoredRow)])
(:wat::core::defrecord :wat::query::PutResponse [])
;; ScanRequest / IndexScanRequest already conform; the page-bearing responses:
(:wat::core::defrecord :wat::query::ScanResponse
  [rows <- (:wat::core::Vector :wat::query::Row)       next-cursor <- (:wat::core::Option :wat::core::String)])
(:wat::core::defrecord :wat::query::IndexScanResponse
  [rows <- (:wat::core::Vector :wat::query::IndexRow)  next-cursor <- (:wat::core::Option :wat::core::String)])

;; ── per-op RESULT enums — :Success first, then only that op's errors ──
(:wat::core::defenum :wat::query::EnsureSchemaResult :wat::enum::Pure
  :Success [ret <- :wat::query::EnsureSchemaResponse]  :Constraint [err <- :wat::query::Constraint]  :Fatal [err <- :wat::query::Fatal])
(:wat::core::defenum :wat::query::PutResult :wat::enum::Pure
  :Success [ret <- :wat::query::PutResponse]  :Constraint [err <- :wat::query::Constraint]  :Transient [err <- :wat::query::Transient]  :Fatal [err <- :wat::query::Fatal])
(:wat::core::defenum :wat::query::ScanResult :wat::enum::Pure
  :Success [ret <- :wat::query::ScanResponse]  :Transient [err <- :wat::query::Transient]  :Fatal [err <- :wat::query::Fatal])
(:wat::core::defenum :wat::query::ScanIndexResult :wat::enum::Pure
  :Success [ret <- :wat::query::IndexScanResponse]  :Transient [err <- :wat::query::Transient]  :Fatal [err <- :wat::query::Fatal])

;; ── the surface — :satisfies-first authoring, :nature :Peer', success-only sigs; the *Result IS the return ──
(:wat::core::defsurface :wat::query::Store :nature :wat::kernel::Peer'
  :features
  [(ensure-schema [self <- :wat::query::Store  req <- :wat::query::EnsureSchemaRequest] -> :wat::query::EnsureSchemaResult)
   (put           [self <- :wat::query::Store  req <- :wat::query::PutRequest]          -> :wat::query::PutResult)
   (scan          [self <- :wat::query::Store  req <- :wat::query::ScanRequest]         -> :wat::query::ScanResult)
   (scan-index    [self <- :wat::query::Store  req <- :wat::query::IndexScanRequest]    -> :wat::query::ScanIndexResult)])
;; synthesized wire:  Op::Put [req <- PutRequest]  +  Reply::Put [result <- PutResult]
;; client-facing:     (:wat::query::Store/put peer req) -> PutResult  ← caller MUST match :Success/:Constraint/:Transient/:Fatal
```

**Per-op error sets (PROPOSED — modeling, pending final builder confirm):** reads (`scan`/`scan-index`) →
`Transient`/`Fatal` (a read cannot violate a constraint); `put` → `Constraint`/`Transient`/`Fatal`; `ensure-schema`
(DDL) → `Constraint`/`Fatal`.

**Provenance (path-of-voices):** the per-op `*Result` shape, `:Success`-always-first, the reusable-error-records
insight, and the `:satisfies`-first law are the **builder's** — reasoned to over the S4 co-design (*"not all ops can
toss all errors"*; *"the err records can be reused… they communicate the shape of the thing the user needs to
handle"*; *"forces the user to always handle all errors — they can never be surprised"*). The materialization (this
contract), the four-questions grounding, and the `MVNIRE`/Smithy framing are the apparatus's.

### The error CONTEXT — RESOLVED (2026-07-06, later: supersedes the `Fault` sketch above)

The `Fault [op, code, diagnostic, message]` sketched above **leaks the backend** — `code <- i64` + a SQL-flavored
`diagnostic` is sqlite's error struct wearing an agnostic contract's clothes (mysql/mongo/redis/ddb/es have their own
shapes). Reasoned to, over the S4 co-design + a live `check.rs` measurement, the resolution:

- **The recovery-class variant is the agnostic, walled part** — `:Constraint` / `:Transient` / `:Fatal` (the `*Result`
  variants), exhaustive-matched. Every backend maps its failures onto retry / surface / abort; that mapping is the
  satisfier's job and the contract's promise.
- **The backend-specific detail is an OPEN `Reason` surface** — `(defsurface :wat::query::Reason :nature
  :wat::core::Record :features [])`, the *exact* pattern `:wat::telemetry'::LogMessage` already ships (`telemetry.wat:56`).
  Any pure record satisfies it **structurally** (no `extend-type`), so a backend just defines its own concrete record —
  `(defrecord :wat::sqlite'::Reason [code <- i64  sql <- String])` — and it *is* a `Reason`, zero ceremony. Each
  `*Result` variant carries `reason <- :wat::query::Reason`.
- **Discrimination is a plain `defclause` on concrete satisfier types** — no `as?`, no `match-type`, no surface-fallback
  (a `[r <- Reason]` clause is runtime-dead — dispatch is on exact concrete class). A backend-aware consumer writes
  concrete clauses (`[r <- :wat::sqlite'::Reason]`); an agnostic consumer never discriminates (it matched the recovery
  *class* already) and logs the `Reason`'s EDN.

**The one substrate enabler (measured this session, floor `4121/1-known/0-new`):** a value typed as an open surface
(`Reason`, out of the agnostic `*Result` field) is rejected by the checker into concrete-satisfier `defclause` clauses,
even though the **runtime already dispatches on concrete class** (arc-237 `value_matches_type_by_name`). The fix is one
condition at `src/check.rs:6104` — the clause-match `assignable(arg, param)` becomes bidirectional, so a clause whose
concrete param **satisfies** the open-surface arg is permitted, deferring the pick to the runtime. This is the
**check-time half of R7's down-narrowing**, general to every open surface (not error-specific). Proven small + safe by
measurement; the **production strike still owes**: (1) **return-type unification** across multiple statically-matching
clauses (first-match-wins is unsound if their return types differ), (2) a proper open-surface **guard** + a **test** in
the tree. Probes: `scratchpad/probe-defclause-{real-shape,open-arg,discriminate}.wat`. Realization: **278 R34 `CAEDOR
ERGO RESEROR`**.

## Open — what's left to scout + draw (the CONTRACT above is settled)

1. **Re-point the codegen.** `defservice`'s existing request/response/variant/`Op`/`Reply` codegen (grounded:
   `wat/service.wat` §C.3, `client-forms-def`) re-sourced FROM the surface's methods, emitted ONCE under the surface
   namespace (`Store::Op`, `Store::PutRequest`, …) instead of per-service `{fqdn}::Op`. Lifecycle
   (`Admin`/`Status`/`Handle`/`serve`/`start`) stays per-service.
2. **`:satisfies`/`:impls` validation** — the `:impls` must cover exactly the surface's methods, and each impl's
   returned response record is checked against the surface's declared response type (mirror the extend-type body-check,
   the honesty strike).
3. **`:calls [surface]`** — extend `:calls` (today: concrete service keywords) to accept a surface + ship the
   surface-sourced client-forms; the consumer dials `Address'<Store::Op, Store::Reply>` (uniform, given as a pure
   operating-input, dialed in `:init`).
4. **The `Reply`-as-error-union shape** — ✅ **RESOLVED 2026-07-06** (see § The `*Result` shape): each op returns a
   per-op `*Result` enum (`:Success` first + only that op's error variants; reusable error records; exhaustive-match-
   walled). `Reply::<Op>` carries the `*Result`; the client method returns it; the caller must exhaustively match it.
5. **Naming (intueri)** — `:satisfies` / `:impls` clause names; where the shared protocol types live (the surface
   namespace); the Request/Response record naming convention.
6. **A disconfirming probe** — can `defservice`'s request/response codegen be re-pointed at a surface's methods and
   emit ONE shared protocol two services reference? (S1's feasibility, before a shadowdancer.)

## S1 — DRAWN (2026-07-05: intueri weighed, feasibility grounded, A/derived-purity decided, reference verified)

**Names (intueri-cast + weighed):** `:satisfies` (the defservice clause; `:serves` rejected — collides with the
`{fqdn}::serve` loop fn) · `:impls` (bodies-only, over keep-`:ops` — the grammar genuinely changes) · `<Op>Request` /
`<Op>Response` (the existing wat/gRPC convention; not AWS `Input`/`Output`) · protocol under the **surface** namespace.

**Gating decision (four-questions; the builder corrected my marker-framing):** NOT a declared marker (that's the footgun —
build threaded, forget the flag, can't network in prod). **A service is loci-agnostic by nature** — thread-hosting is a
novelty of location — so it is *always* dialable. The gate is **DERIVED PURITY**, self-applying: a surface emits its
protocol **iff its method sigs are pure (EDN-crossable)**. Impure sigs (a method taking/returning a live `Peer'`/
`Connection`) can't cross → 293.W already rejects → not a service, honestly. No new branch; the wire wall applied once more.

**Locus (grounded):** `defsurface` is **Rust-handled** (`src/types/surface.rs` `parse_defsurface`, routed
`src/types.rs:1941`) — it registers a `TypeDef::Surface`, it is NOT a wat macro. So S1 is a **Rust strike**: in the
surface-registration path, when the method sigs are pure, **synthesize + register `Surface::Op` and `Surface::Reply`
`EnumDef`s** from the surface's `SurfaceMember::Method` list (each method → one variant per enum). The request/response
records are **user-declared** (the method members reference them); S1 emits only the two enums.

**The map (per method):** `(put [self <- :S  req <- :S::PutRequest] -> :S::PutResponse)` ⟶
`:S::Op::Put [req <- :S::PutRequest]` + `:S::Reply::Put [resp <- :S::PutResponse]` (PascalCase variant per method).
Error variants in `Reply` are a downstream concern (the `Reply`-as-error-union item below).

**Reference target (VERIFIED — `cargo wat` prints "S1 reference target type-checks"; the codegen is graded against it):**

```clojure
;; user-declared request/response records + surface (methods reference the records; pure sigs → serviceable):
(:wat::core::defrecord :probe::Kv::PutRequest  [key <- :wat::core::String  val <- :wat::core::String])
(:wat::core::defrecord :probe::Kv::PutResponse [ok  <- :wat::core::bool])
(:wat::core::defrecord :probe::Kv::GetRequest  [key <- :wat::core::String])
(:wat::core::defrecord :probe::Kv::GetResponse [val <- (:wat::core::Option :wat::core::String)])
(:wat::core::defsurface :probe::Kv :holder :wat::core::Struct
  :features [(put [self <- :probe::Kv  req <- :probe::Kv::PutRequest] -> :probe::Kv::PutResponse)
             (get [self <- :probe::Kv  req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse)])
;; S1 SYNTHESIZES (from the method members) — the target shape:
(:wat::core::defenum :probe::Kv::Op :wat::enum::Pure
  :Put [req <- :probe::Kv::PutRequest]  :Get [req <- :probe::Kv::GetRequest])
(:wat::core::defenum :probe::Kv::Reply :wat::enum::Pure
  :Put [resp <- :probe::Kv::PutResponse]  :Get [resp <- :probe::Kv::GetResponse])
```
(full file: `scratchpad/s1-reference-target.wat`.)

**S1 gate:** a pure-sig surface's synthesized `::Op`/`::Reply` exist + are constructible/matchable (the reference target,
now produced by the codegen rather than hand-written); an impure-sig surface synthesizes nothing (no regression); whole
floor 0-new-failures.

## Blast radius (for the eventual strike)

`wat/service.wat` (the `defservice` macro — the `:satisfies` clause + surface-sourced protocol emission) · the surface
machinery in `src/check.rs` (service-satisfies-surface checking, mirroring extend-type) · `:calls` accepting a surface
· the store services (`mem-store'`/`sqlite-store'` gain `:satisfies :wat::query::Store`, their per-service `::Op` retired
for the shared `Store::Op`) · then **278/T1b** (the sink `:calls [:wat::query::Store]`, dials blind — now assembly).

## Four-questions verdict (the shape)

**Obvious** YES (a service serves one protocol; the surface is its spec). **Simple** YES (one service = one surface =
one protocol, un-braided; the asymmetry with structs is named, not hidden). **Honest** YES (`:satisfies` is generative
here, not the redundant label it would be on a struct; the spec/server/client can't drift because they're one typed
object). **Good UX** YES (the client dials `Address'<Store::Op>` blind; a new backend is one `defservice :satisfies`,
a new transport is one `Locus` `extend-type`).

## S3 — DRAWN (2026-07-06): the client side, the Nature model, and "a service is a surface at a coordinate"

**The crystallization (278 R32 `QVANTVMVIS PROCVL, IDEM NEXVS`, the builder's synthesis):**
**every service is a *surface* whose *nature* is `:Peer`, living at a *coordinate*.** From the caller's side there is
nothing else — two things: the **surface** (*what* to say — build the request, handle the reply) and the **coordinate**
(its `Address'` — *where* to dial). The Locus (transport) is orthogonal + unbounded. Because a service is *nothing but*
`(surface, coordinate)`, the relationship is **distance-invariant** — dial → peer → speak the surface, identical
in-thread (`ThreadSelfPeer'`) or across the world (`Process'`/mTLS). Distance became a coordinate VALUE, not a wall.

### The client-side names (intueri-cast + weighed + ratified)
- **peer-as-satisfier — NO NAME.** A dialed `Peer'<S::Op,S::Reply>` **directly satisfies** the surface `:S` (a generated
  `extend-type` whose bodies `send'`/`recv'` the Op/Reply). A wrapper type re-complects R31 — `Client`/`Remote`/`Proxy`/
  `Stub` all rejected (role-name / locus-lie / codegen-jargon).
- **client entry — `:S/<op>`, UNIFORM.** The surface method itself, dispatched through whatever satisfies `:S` (a local
  struct OR a dialed peer). The wire is invisible in the name; RPC-vs-local is the decomplection, not a distinction.
  (`:S/<op>!` lies — a bang asserts the wire; `:S::call/<op>` mumbles the mechanism.)
- **dial — `connect'`.** No surface-specific dial; the `Address'` is already protocol-typed to `:S`.
- **return — `Result<Response, OpError>`** at the face, **uniform across every satisfier** (the surface method's return
  becomes this too — else local returns bare `Response`, dialed returns `Result`, and the wire LEAKS through the return
  type). `Reply` stays the **pure per-op demux** (which op's reply is this); the **outcome** (success vs error) is the
  `Result`. **This supersedes the earlier "errors are `Reply` variants" sketch** (§ The mechanism) — one axis per enum.

### The Nature model (`Holder` → `Nature`; ratified)
A `Peer'` **cannot satisfy a holder-bound surface**: the holder (`:Struct`/`:Record`/`:HolonRecord`) is an **aggregate**
floor, and a peer is no aggregate (the foreign-satisfaction path requires `holder.is_none()`, but `:holder` is mandatory
— arc 293 K0a). Grounding *why* (`AGGREGATE-AUDIT.md` — the holder is the **capability trit**; `DESIGN-293.W` — mandatory
because **a default masks intent**) flipped the naive "just relax it" lean: the capability must be **declared, not
defaulted**. So intueri was cast: **`Holder` LIES** — a peer *holds nothing* (a category pun, the same 293.W cut for
enums) → **`Nature`**, the satisfier's intrinsic character, from which the boundary trit is *derived*. ONE axis, four
mutually-exclusive variants (a satisfier is a backed-value **or** a peer, never both):

```clojure
(:wat::core::defenum :wat::types::Nature :wat::enum::Portable
  :Struct         ;; stays home — may hold live resources, cannot cross
  :Record         ;; travels by copy — pure data, crosses as EDN
  :HolonRecord    ;; travels with VSA — pure data + a hologram
  :Peer)          ;; IS the door everything else travels through — a live channel endpoint,
                  ;;   reached only by its coordinate (Address'); never moves. (:Remote LIES — may be same-process.)
(:wat::core::defsurface :wat::query::Store :nature :Peer   ;; declared, never defaulted (293.W)
  :features [(put [self <- :Store  req <- :Store::PutRequest] -> :Store::PutResponse) …])
```
A `:nature :Peer` surface is **peer-satisfied only** — local = an in-thread peer; there is no coexisting bare-struct
satisfier (R31's one act). `:Value`/"Any" was explicitly REJECTED (a non-assertion; the builder: *"i'm very hesitant to
accept an Any"*) in favor of the positive nature `:Peer`.

### The decomposition (updated)
- **S3a — DONE (`93e936b3`).** A parametric `extend-type` self decomposes to `Parametric` (not a flat `Path`), so
  `send'`/`recv'` accept a `Peer'` extend-type self (`runtime.rs:709` → `parse_type_expr`). Closed `project_peer_io`;
  0-new-failures. (Exposed Gap B below.)
- **S3-Nature — the substrate stone (NEXT).** Rename `Holder` → `Nature` (the `Holder` enum + `:holder`→`:nature` clause
  + the ~99 `Holder::…` branches + `AGGREGATE-AUDIT.md`); add the `:Peer` nature; **teach the checker that a
  `Peer'<S::Op,S::Reply>` satisfies a `:nature :Peer` surface** — which *is* the fix for S3's **Gap B** (the call-site
  `:S/<op>` receiver check rejecting the `Peer'` today).
- **S3b — the feature (assembly after S3-Nature).** `:calls [surface]` (**surface-only** — ratified; zero current
  consumers, born surface-only, no retirement sweep; the client mirror of the server-side `:ops`→`:impls` endpoint).
  Generate the peer-as-satisfier `extend-type` + the surface-sourced `:S/<op>` client-forms returning `Result`; the
  caller dials `Address'<S::Op,S::Reply>` **blind**. → 278/T1b (the blind sink) is then assembly.
